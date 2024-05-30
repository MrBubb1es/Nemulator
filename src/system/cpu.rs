use std::rc::Rc;
use std::cell::RefCell;

use bitfield_struct::bitfield;

use crate::cartridge::mapper::Mapper;

use super::instructions::{AddressingMode, Instruction, OpcodeData, INSTRUCTION_TABLE, DEFAULT_ILLEGAL_OP};

use super::mem::Memory;
use super::ppu::Ppu2C02;

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
    pub total_clocks: usize,
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

    // Memory accessable only by the CPU
    sys_ram: Memory,
    prg_rom: Memory,

    // References to the cartridge mapper and PPU are required so the CPU can
    // map addresses & read/write data to and from the PPU
    mapper: Rc<dyn Mapper>,
    ppu: Rc<RefCell<Ppu2C02>>,

    cycles_remaining: usize, // Number of CPU clocks before next instruction
    total_clocks: usize, // Total number of clocks since CPU started running

    current_instr: Instruction,
    instr_data: OpcodeData,
}

impl Cpu6502 {
    /// Make a new CPU with access to given program memory and PPU Registers and a given mapper
    pub fn new(prg_rom: Memory, ppu: Rc<RefCell<Ppu2C02>>, mapper: Rc<dyn Mapper>) -> Self {
        let mut new_cpu = Cpu6502{
            acc: 0,
            x: 0,
            y: 0,
            sp: 0, // will be set to 0xFD in reset due to wrapping sub
            pc: 0,
            status: CpuStatus::from_bits(0x20), // start w/ unused flag on cuz why not ig (fixes nesdev tests)

            nmi_flag: false,

            sys_ram: Memory::new(0x800), // NES has 2KiB of internal memory that only the CPU can access
            prg_rom: prg_rom,
            ppu: Rc::clone(&ppu),
            mapper: Rc::clone(&mapper),

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
    pub fn cycle(&mut self) -> bool {
        let mut excecuted = false;

        if self.cycles_remaining == 0 {
            if self.nmi_flag {
                self.nmi();
                self.nmi_flag = false;
                
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

            self.total_clocks += self.cycles_remaining;
        }

        self.cycles_remaining -= 1;

        excecuted
    }
}

// Internal & Helper functionality
impl Cpu6502 {
    // HELPER FUNCTIONS

    /// Read a single byte from a given address off the bus
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => {
                // First 2KiB of memory (0x0800) are mirrored until 0x2000
                self.sys_ram.read(address & 0x07FF)
            },
            0x2000..=0x3FFF => {
                // PPU Registers mirrored over 8KiB
                self.ppu.as_ref().borrow_mut().cpu_read(address & 0x0007)
            }
            // 0x4000..=0x401F => {
            //     // APU or I/O Reads
            //     println!("APU READ OCCURED");
            //     0xEE
            // },
            0x4000..=0xFFFF => {
                // Read to program ROM through mapper
                if let Some(mapped_addr) = self.mapper.get_cpu_read_addr(address) {
                    self.prg_rom.read(mapped_addr)
                } else {
                    0
                }
            },
        }
    }
    /// Write a single byte to the bus at a given address
    pub fn write(&self, address: u16, data: u8) {
        // println!("CPU Write to ${address:04X} w/ 0x{data:02X}");
        match address {
            0x0000..=0x1FFF => {
                // First 2KiB of memory (0x0800) are mirrored until 0x2000
                self.sys_ram.write(address & 0x07FF, data);
            },
            0x2000..=0x3FFF => {
                // PPU Registers mirrored over 8KiB

                self.ppu.as_ref().borrow_mut().cpu_write(address & 0x0007, data);
            },
            // 0x4000..=0x401F => {
            //     // APU or I/O Writes
            //     println!("APU WRITE OCCURED");
            // },
            0x4000..=0xFFFF => {
                // Read to program ROM through mapper
                if let Some(mapped_addr) = self.mapper.get_cpu_write_addr(address, data) {
                    self.prg_rom.write(mapped_addr, data);
                }
            },
        };
    }
    /// Read a 2 byte value starting at the given address in LLHH (little-endian) form
    pub fn read_word(&self, address: u16) -> u16 {
        let lo = self.read(address) as u16;
        let hi = self.read(address + 1) as u16;
        (hi << 8) | lo
    }
    /// Write a 2 byte value to a given address in LLHH form
    pub fn write_word(&self, address: u16, data: u16) {
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
    pub fn write_zpage_word(&self, zpage_address: u8, data: u16) {
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

        self.total_clocks += 7;
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
            self.status.set_b(false);
            self.status.set_unused(true);
            self.status.set_interrupt(true);
            self.push_to_stack(self.get_status());

            // Set PC to whatever is at addr 0xFFFE
            self.pc = self.read_word(IRQ_PC_VECTOR);

            // Interrupts take 7 clock cycles
            self.total_clocks += 7;
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
        self.status.set_b(false);
        self.status.set_unused(true);
        self.status.set_interrupt(true);
        self.push_to_stack(self.get_status());

        // Set PC to whatever is at addr 0xFFFE
        self.pc = self.read_word(NMI_PC_VECTOR);

        // Interrupts take 7 clock cycles
        self.total_clocks += 7;
    }
}

// Public functionality
impl Cpu6502 {
    // GETTER/SETTER FUNCTIONS

    /// Get the current number of clock cycles since turn-on
    pub fn total_clocks(&self) -> usize {
        self.total_clocks
    }

    /// Manually set the number of clock cycles since turn-on
    pub fn set_total_clocks(&mut self, clks: usize) {
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


    /// Get the instruction just executed as a string of 6502 assembly for
    /// debugging purposes.
    // pub fn current_instr_str(&self) -> String {
    //     let mut out_str = format!("0x{:02X}", self.current_instr.opcode_num);
    //     out_str.push(' ');
    //     out_str.push_str(self.current_instr.name);

    //     out_str.push(' ');
    //     let temp = match self.current_instr.addr_mode {
    //         AddressingMode::Accumulator => String::from("A : [acc]"),

    //         AddressingMode::Implied => String::from(": [imp]"),

    //         AddressingMode::Immediate => {
    //             format!("#${:02X} : [imm]", self.instr_data.data.unwrap())
    //         }

    //         AddressingMode::Absolute => {
    //             format!("${:04X} : [abs]", self.instr_data.address.unwrap())
    //         }
    //         AddressingMode::AbsoluteX => {
    //             format!("${:04X},X : [abs x]", self.instr_data.address.unwrap())
    //         }
    //         AddressingMode::AbsoluteY => {
    //             format!("${:04X},Y : [abs y]", self.instr_data.address.unwrap())
    //         }

    //         AddressingMode::ZeroPage => {
    //             format!("${:02X} : [zpage]", self.instr_data.address.unwrap())
    //         }
    //         AddressingMode::ZeroPageX => {
    //             format!("${:02X},X : [zpage x]", self.instr_data.address.unwrap())
    //         }
    //         AddressingMode::ZeroPageY => {
    //             format!("${:02X},Y : [zpage y]", self.instr_data.address.unwrap())
    //         }

    //         // Don't know the original data from the instruction, it's somewhere in memory
    //         AddressingMode::Indirect => {
    //             String::from("$(??) : [ind]")
    //         }
    //         AddressingMode::IndirectX => {
    //             String::from("$(??),X : [ind x]")
    //         }
    //         AddressingMode::IndirectY => {
    //             String::from("$(??),Y : [ind y]")
    //         }

    //         AddressingMode::Relative => format!(
    //             "${:02X} : [rel, offset = {}]",
    //             self.instr_data.offset.unwrap() as u8,
    //             self.instr_data.offset.unwrap()
    //         ),
    //     };
    //     out_str.push_str(&temp);

    //     if self.current_instr.is_illegal {
    //         out_str.push_str(" <ILLEGAL>");
    //     }

    //     out_str
    // }

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
        // text.push_str(&format!("  Last Instr: {}", self.current_instr_str()));
        text.push_str(&format!("  Total Clks: {}", self.total_clocks));
        text
    }

    pub fn print_state(&self) {
        println!("CPU State:");
        println!("{}", self.state_str());
    }
}


#[cfg(test)]
mod tests {
    // use std::fs;
    // use std::io::Read;

    // use super::CPU;

    #[test]
    // Test program that multiplies 3 and 10 and stores the result in Accumulator
    fn test_multiply() {
        // let test_file = "prg_tests/cpu_tests/test_multiply.bin";
        // let test_bus = Bus::new();
        // let mut test_cpu = CPU::new(&test_bus);

        // load_raw_mem_to_cpu(&test_cpu, &test_file);

        // // Just make sure the program loaded correctly
        // // (only checking a couple bytes, one at the start and one near the end)
        // // assert_eq!(test_cpu.read(0x0000), 0xA9);
        // // assert_eq!(test_cpu.read(0x0010), 0x00);

        // // test_cpu.reset();

        // crate::run_debug(&mut test_cpu, &test_bus);
    }
}

