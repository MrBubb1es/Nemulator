use std::rc::Rc;
use std::cell::{Cell, RefCell};

use bitfield_struct::bitfield;

use crate::cartridge::mapper::Mapper;

use super::apu::Apu2A03;
use super::controller::{ControllerReadState, NesController};
use super::instructions::{AddressingMode, Instruction, OpcodeData, INSTRUCTION_TABLE, DEFAULT_ILLEGAL_OP};

use super::ppu::Ppu2C02;

// NES has 2KiB of internal memory that only the CPU can access
const SYS_RAM_SIZE: usize = 0x800;

// NVUBDIZC
#[bitfield(u8)]
pub struct CpuStatus {
    pub carry: bool,
    pub zero: bool,
    pub interrupt: bool,
    pub decimal: bool,
    pub b: bool,
    pub unused: bool,
    pub overflow: bool,
    pub negative: bool,
}

#[derive(Default)]
pub struct CpuState {
    pub acc: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub status: CpuStatus,
    pub cycles_remaining: usize,
    pub total_clocks: u64,
}

/// Representation of the NES 6502 CPU. Thankfully, the good gentelmen down at
/// the lab have already done extensive research and documentation of this
/// particular device, so if you ever have questions about why things are the
/// way they are, check this wiki: https://www.nesdev.org/wiki/CPU
pub struct Cpu6502 {
    acc: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    pub status: CpuStatus,

    // Flag used to keep track of when the PPU triggers an NMI
    nmi_flag: bool,
    // Flag used to keep track of when the APU triggers an IRQ
    irq_flag: bool,

    // Memory accessable only by the CPU
    sys_ram: [u8; SYS_RAM_SIZE],

    // Last polled states of each controller
    polled_p1_controller: NesController,
    polled_p2_controller: NesController,
    // Current reading status of each controller
    p1_read_state: Cell<ControllerReadState>,
    p2_read_state: Cell<ControllerReadState>,
    // Flags dictating whether to update the polled controller values
    poll_p1: Cell<bool>,
    poll_p2: Cell<bool>,

    // References to the cartridge mapper and PPU are required so the CPU can
    // map addresses & read/write data to and from the PPU
    mapper: Rc<RefCell<dyn Mapper>>,
    ppu: Rc<RefCell<Ppu2C02>>,
    apu: Rc<RefCell<Apu2A03>>,

    oam_data: u8,
    oam_address: u16,
    dma_in_progress: bool,

    cycles_remaining: usize, // Number of CPU clocks before next instruction
    total_clocks: u64, // Total number of clocks since CPU started running

    current_instr: Instruction,
    instr_data: OpcodeData,
}

impl Cpu6502 {
    /// Make a new CPU with access to given program memory and PPU Registers and a given mapper
    pub fn new(ppu: Rc<RefCell<Ppu2C02>>, 
            apu: Rc<RefCell<Apu2A03>>, 
            mapper: Rc<RefCell<dyn Mapper>>
        ) -> Self {

        let mut new_cpu = Cpu6502{
            acc: 0,
            x: 0,
            y: 0,
            sp: 0, // will be set to 0xFD in reset due to wrapping sub
            pc: 0,
            status: CpuStatus::from_bits(0x20), // start w/ unused flag on cuz why not ig (fixes nesdev tests)

            nmi_flag: false,
            irq_flag: false,

            sys_ram: [0; SYS_RAM_SIZE],

            polled_p1_controller: NesController::default(),
            polled_p2_controller: NesController::default(),
            p1_read_state: Cell::new(ControllerReadState::new()),
            p2_read_state: Cell::new(ControllerReadState::new()),
            poll_p1: Cell::new(true),
            poll_p2: Cell::new(true),

            mapper,
            ppu,
            apu,

            oam_data: 0,
            oam_address: 0,
            dma_in_progress: false,

            cycles_remaining: 0,
            total_clocks: 0,
    
            current_instr: DEFAULT_ILLEGAL_OP,
            instr_data: OpcodeData {
                data: None,
                address: None,
                offset: None,
            },
        };

        new_cpu.reset();

        new_cpu
    }

    /// Cycles the CPU through one whole instruction, taking as many clock
    /// cycles as that instruction requires. This function encapsulates all of
    /// the fetch, decode, and execute stages of the CPU. Returns a bool 
    /// reporting whether an instruction was excecuted.
    pub fn cycle(&mut self, p1_controller_state: NesController, p2_controller_state: NesController) -> bool {
        let mut excecuted = false;

        // Update controllers
        if self.poll_p1.get() { self.polled_p1_controller = p1_controller_state; }
        if self.poll_p2.get() { self.polled_p2_controller = p2_controller_state; }

        if self.cycles_remaining == 0 {
            if self.nmi_flag {
                self.nmi();
                self.nmi_flag = false;

                return false;
            }

            if self.irq_flag {
                self.irq();
                self.irq_flag = false;

                return false;
            }

            excecuted = true;

            let opcode = self.read(self.pc);
    
            // fetch - get the opcode we are running
            let instr = &INSTRUCTION_TABLE[opcode as usize];
    
            // decode - retrieve the neccesary data for the instruction
            let (opcode_data, fetch_cycles) = (instr.addr_func)(self);

            // store the instruction (for debugging)
            self.current_instr = instr.clone();
    
            // Increment pc before instruction execution
            self.pc += instr.bytes as u16;
            
            // execute - run the instruction, updating memory and processor status
            //           as defined by the instruction
            let execute_cycles = (instr.func)(self, opcode_data);
    
            self.cycles_remaining += execute_cycles + instr.base_clocks;
    
            if instr.has_extra_fetch_cycles {
                self.cycles_remaining += fetch_cycles;
            }
        }

        self.cycles_remaining -= 1;
        self.total_clocks += 1;
        
        excecuted
    }
}

// Internal & Helper functionality
impl Cpu6502 {
    // HELPER FUNCTIONS

    /// Read a single byte from a given address off the bus
    pub fn read(&self, address: u16) -> u8 {
        if let Some(data) = self.mapper.borrow_mut().cpu_cart_read(address) {
            return data;
        }

        match address {
            0x0000..=0x1FFF => {
                // First 2KiB of memory (0x0800) are mirrored until 0x2000
                self.sys_ram[(address & 0x07FF) as usize]
            },
            0x2000..=0x3FFF => {
                // PPU Registers mirrored over 8KiB
                self.ppu.as_ref().borrow_mut().cpu_read(address)
            }
            0x4016 => {
                // Player 1 controller port
                let data = self.polled_p1_controller.read_button(self.p1_read_state.get());

                if !self.poll_p1.get() {
                    self.p1_read_state.set( self.p1_read_state.get().next() );
                }

                data
            }
            0x4017 => {
                // Player 2 controller port
                let data = self.polled_p2_controller.read_button(self.p2_read_state.get());

                if !self.poll_p2.get() {
                    self.p2_read_state.set( self.p2_read_state.get().next() );
                }

                data
            }
            0x4000..=0x401F => { 0xEE },
            // 0x4000..=0x401F => {
            //     // APU or I/O Reads
            //     println!("APU READ OCCURED");
            //     0xEE
            // },
            // 0x4020..=0xFFFF => {
            //     // Read to program ROM through mapper
            //     if let Some(mapped_addr) = self.mapper.borrow_mut().get_cpu_read_addr(address) {
            //         self.prg_rom[mapped_addr as usize]
            //     } else {
            //         0
            //     }
            // },
            _ => 0x00,
        }
    }
    /// Write a single byte to the bus at a given address
    pub fn write(&mut self, address: u16, data: u8) {
        if self.mapper.borrow_mut().cpu_cart_write(address, data) {
            return;
        }

        // println!("CPU Write to ${address:04X} w/ 0x{data:02X}");
        match address {
            // CPU RAM
            0x0000..=0x1FFF => {
                // First 2KiB of memory (0x0800) are mirrored until 0x2000
                self.sys_ram[(address & 0x07FF) as usize] = data;
            },

            // PPU Internal Registers
            0x2000..=0x3FFF => {
                // PPU Registers mirrored over 8KiB
                self.ppu.as_ref().borrow_mut().cpu_write(address, data);
            },

            // APU Addresses
            0x4000..=0x4013 => {
                self.apu.as_ref().borrow_mut().cpu_write(address, data);
            },

            // PPU OAM DMA Register
            0x4014 => {
                // self.oam_address = (data as u16) << 8;
                // self.dma_in_progress = true;
                let source_addr = ((data as u16) << 8) as usize;
                let oam_dma_source = &self.sys_ram[source_addr..source_addr+256];
                self.ppu.as_ref().borrow_mut().full_oam_dma_transfer(oam_dma_source);
                // self.dma_in_progress = true;
                self.cycles_remaining += (513 + self.total_clocks & 1) as usize;
            },

            0x4015 => {
                self.apu.as_ref().borrow_mut().cpu_write(address, data);
            }

            // Player 1 Controller Port
            0x4016 => {
                self.poll_p1.set(data & 1 == 1);
                self.p1_read_state.set(ControllerReadState::new());
            },

            // Player 2 Controller Port & APU Register
            0x4017 => {
                self.poll_p2.set(data & 1 == 1);
                self.p2_read_state.set(ControllerReadState::new());

                self.apu.as_ref().borrow_mut().cpu_write(address, data);
            },

            0x4018 | 0x4019 => {},
            
            // Program ROM
            // 0x4020..=0xFFFF => {
            //     // Write to program ROM through mapper
            //     if let Some(mapped_addr) = self.mapper.borrow_mut().get_cpu_write_addr(address, data) {
            //         self.prg_rom[mapped_addr as usize] = data;
            //     }
            // },

            _ => {},
        };
    }
    /// Read a 2 byte value starting at the given address in LLHH (little-endian) form
    pub fn read_word(&self, address: u16) -> u16 {
        let lo = self.read(address) as u16;
        let hi = self.read(address + 1) as u16;
        (hi << 8) | lo
    }
    /// Reads the next OAM DMA byte into an internal register to be fetched a few
    /// NES cycles down the line.
    pub fn read_next_oam_data(&mut self) {
        self.oam_data = self.read(self.oam_address);
        
        self.oam_address = self.oam_address.wrapping_add(1);

        // OAM DMA ends when address wraps back to $XX00, so as long as the low
        // byte of the OAM address isn't 0, the DMA is still going on.
        if self.oam_address & 0xFF == 0 {
            self.dma_in_progress = false;
        } else {
            self.dma_in_progress = true;
        }
    }
    
    /// Write a 2 byte value to a given address in LLHH form
    pub fn write_word(&mut self, address: u16, data: u16) {
        let lo = data as u8;
        let hi = (data >> 8) as u8;
        self.write(address, lo);
        self.write(address + 1, hi);
    }
    /// Read a 2 byte value from the zero-page of memory. If the address being read from is 0xFF,
    /// then the high byte will be taken from address 0x00 (wrap around zero-page)
    pub fn read_zpage_word(&self, zpage_address: u8) -> u16 {
        let lo = self.read(zpage_address as u16) as u16;
        let hi = self.read(zpage_address.wrapping_add(1) as u16) as u16;
        (hi << 8) | lo
    }
    /// Write a 2 byte value to the z-page in memory at given address. If address is 0xFF, the
    /// second byte will be written to 0x00 (wrapping z-page).
    pub fn write_zpage_word(&mut self, zpage_address: u8, data: u16) {
        let lo = data as u8;
        let hi = (data >> 8) as u8;
        self.write(zpage_address as u16, lo);
        self.write(zpage_address.wrapping_add(1) as u16, hi);
    }
    /// Push a byte to the stack in main memory. The stack is the first page of
    /// memory (i.e. 0x0100-0x01FF, right after the zero page). Decrements the
    /// sp after the value is pushed.
    pub fn push_to_stack(&mut self, data: u8) {
        
        // println!("Pushing 0x{:02X} to stk", data);

        let stk_address = 0x0100 | self.sp as u16;
        self.write(stk_address, data);
        self.sp = self.sp.wrapping_sub(1);
    }
    /// Pop or 'pull' a value from the stack.
    pub fn pop_from_stack(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let stk_address = 0x0100 | self.sp as u16;
        let data = self.read(stk_address);

        // println!("Popping 0x{:02X} from stk", data);

        data
    }

    // RESET FUNCTION

    /// Runs the defined reset sequence of the 6502, detailed here:
    /// https://www.nesdev.org/wiki/CPU_power_up_state
    pub fn reset(&mut self) {
        const RESET_PC_VECTOR: u16 = 0xFFFC;

        // self.acc = 0;
        // self.x = 0;
        // self.y = 0;
        self.sp = self.sp.wrapping_sub(3);

        // Interrupt flag set and unused flag unchanged, set the rest to 0
        self.status.set_carry(false);
        self.status.set_zero(false);
        self.status.set_interrupt(true);
        self.status.set_decimal(false);
        self.status.set_b(false);
        // leave unused flag alone
        self.status.set_overflow(false);
        self.status.set_negative(false);

        self.pc = self.read_word(RESET_PC_VECTOR);

        self.cycles_remaining += 7;
    }

    // INTERRUPTS

    /// Make an interrupt request to the CPU. Only interrupts if the interrupt
    /// disable flag is set to 0. The interrupt sequence is detailed here:
    /// https://www.nesdev.org/wiki/CPU_interrupts
    pub fn irq(&mut self) {
        // Check interrupt disable flag
        if !self.status.interrupt() {
            const IRQ_PC_VECTOR: u16 = 0xFFFE;

            // Store PC
            let lo = self.pc as u8;
            let hi = (self.pc >> 8) as u8;
            self.push_to_stack(hi);
            self.push_to_stack(lo);

            // Set flags and store status
            self.status.set_b(true);
            self.status.set_unused(true);
            self.push_to_stack(self.get_status());
            
            self.status.set_interrupt(true);

            // Set PC to whatever is at addr 0xFFFE
            self.pc = self.read_word(IRQ_PC_VECTOR);

            // Interrupts take 7 clock cycles
            self.cycles_remaining += 7;
        }
    }
    /// Send a non-maskable interrupt to the CPU, which executes the defined
    /// 6502 interrupt sequence regardless of the state of the interrupt disable
    /// flag. The interrupt sequence is detailed here:
    /// https://www.nesdev.org/wiki/CPU_interrupts
    pub fn nmi(&mut self) {
        const NMI_PC_VECTOR: u16 = 0xFFFA;

        // Store PC
        let lo = self.pc as u8;
        let hi = (self.pc >> 8) as u8;
        self.push_to_stack(hi);
        self.push_to_stack(lo);

        // Set flags and store status
        self.status.set_b(true);
        self.status.set_unused(true);
        self.push_to_stack(self.get_status());

        self.status.set_interrupt(true);

        // Set PC to whatever is at addr 0xFFFE
        self.pc = self.read_word(NMI_PC_VECTOR);

        // Interrupts take 7 clock cycles
        self.cycles_remaining += 7;
    }
}

// Public functionality
impl Cpu6502 {
    // GETTER/SETTER FUNCTIONS

    /// Get the current number of clock cycles since turn-on
    pub fn total_clocks(&self) -> u64 {
        self.total_clocks
    }

    /// Manually set the number of clock cycles since turn-on
    pub fn set_total_clocks(&mut self, clks: u64) {
        self.total_clocks = clks;
    }

    /// Get the number of cycles until the next instruction
    pub fn get_remaining_cycles(&self) -> usize {
        self.cycles_remaining
    }

    /// Manually set the number of remaining cycles before next instruction
    pub fn set_remaining_cycles(&mut self, cycles: usize) {
        self.cycles_remaining = cycles;
    }
    
    /// Get the CPU Status byte
    pub fn get_status(&self) -> u8 {
        self.status.0
    }
    
    /// Set the whole processor status byte
    pub fn set_status(&mut self, val: u8) {
        self.status = CpuStatus::from_bits(val);
    }

    /// Get the current value of the accumulator
    pub fn get_acc(&self) -> u8 {
        self.acc
    }
    /// Get the current value of the X register
    pub fn get_x_reg(&self) -> u8 {
        self.x
    }
    /// Get the current value of the Y register
    pub fn get_y_reg(&self) -> u8 {
        self.y
    }
    /// Get the current value of the stack pointer
    pub fn get_sp(&self) -> u8 {
        self.sp
    }
    /// Get the current value of the program counter
    pub fn get_pc(&self) -> u16 {
        self.pc
    }
    /// Get the last read byte from the OAM DMA process
    pub fn get_oam_data(&self) -> u8 {
        self.oam_data
    }
    /// Get the 8-bit address last used to read data from the page being accessed for the OAM DMA process
    pub fn get_oam_addr(&self) -> u8 {
        ((self.oam_address.wrapping_sub(1)) & 0xFF) as u8
    }

    pub fn dma_in_progress(&self) -> bool {
        self.dma_in_progress
    }

    /// Set the value of the accumulator
    pub fn set_acc(&mut self, val: u8) {
        self.acc = val;
    }
    /// Set the value of the X register
    pub fn set_x_reg(&mut self, val: u8) {
        self.x = val;
    }
    /// Set the value of the Y register
    pub fn set_y_reg(&mut self, val: u8) {
        self.y = val;
    }
    /// Set the value of the stack pointer
    pub fn set_sp(&mut self, val: u8) {
        self.sp = val;
    }
    /// Set the value of the program counter
    pub fn set_pc(&mut self, val: u16) {
        self.pc = val;
    }

    pub fn trigger_ppu_nmi(&mut self) {
        self.nmi_flag = true;
    }

    pub fn trigger_apu_irq(&mut self) {
        self.irq_flag = true;
    }

    pub fn increment_clock(&mut self) {
        self.total_clocks += 1;
    }


    /// Get the instruction just executed as a string of 6502 assembly for
    /// debugging purposes.
    pub fn current_instr_str(&self) -> String {
        let mut out_str = format!("0x{:02X}", self.current_instr.opcode_num);
        out_str.push(' ');
        out_str.push_str(self.current_instr.name);

        out_str.push(' ');
        let temp = match self.current_instr.addr_mode {
            AddressingMode::Accumulator => String::from(": [acc]"),

            AddressingMode::Implied => String::from(": [imp]"),

            AddressingMode::Immediate => String::from(": [imm]"),

            AddressingMode::Absolute => String::from(": [abs]"),
            AddressingMode::AbsoluteX => String::from(": [abs x]"),
            AddressingMode::AbsoluteY => String::from(": [abs y]"),

            AddressingMode::ZeroPage => String::from(": [zpage]"),
            AddressingMode::ZeroPageX => String::from(": [zpage x]"),
            AddressingMode::ZeroPageY => String::from(": [zpage y]"),

            // Don't know the original data from the instruction, it's somewhere in memory
            AddressingMode::Indirect => String::from(": [ind]"),
            AddressingMode::IndirectX => String::from(": [ind x]"),
            AddressingMode::IndirectY => String::from(": [ind y]"),

            AddressingMode::Relative => String::from(": [rel]"),
        };
        out_str.push_str(&temp);

        if self.current_instr.is_illegal {
            out_str.push_str(" <ILLEGAL>");
        }

        out_str
    }

    pub fn get_state(&self) -> CpuState {
        CpuState {
            acc: self.acc,
            x: self.x,
            y: self.y,
            sp: self.sp,
            pc: self.pc,
            status: self.status,
            cycles_remaining: self.cycles_remaining,
            total_clocks: self.total_clocks,
        }
    }

    pub fn state_str(&self) -> String {
        let mut text = String::with_capacity(200);
        text.push_str(&format!("  A: 0x{:02X}, X: 0x{:02X}, Y: 0x{:02X}", self.acc, self.x, self.y));
        text.push_str(&format!("  SP: 0x{:02X}, PC: 0x{:04X}", self.sp, self.pc));
        text.push_str(&format!("  Status (NVUBDIZC): {:08b}", self.get_status()));
        text.push_str(&format!("  Last Instr: {}", self.current_instr_str()));
        text.push_str(&format!("  Total Clks: {}", self.total_clocks));
        text
    }

    pub fn print_state(&self) {
        println!("CPU State:");
        println!("{}", self.state_str());
    }
}

