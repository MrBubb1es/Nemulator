use std::rc::Rc;

use bitfield_struct::bitfield;

use crate::cartridge::mapper::Mapper;

use super::instructions::{AddressingMode, Instruction, OpcodeData, INSTRUCTION_TABLE, DEFAULT_ILLEGAL_OP};

use super::mem::Memory;
use super::ppu::PpuRegisters;

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

/// Representation of the NES 6502 CPU. Thankfully, the good gentelmen down at
/// the lab have already done extensive research and documentation of this
/// particular device, so if you ever have questions about why things are the
/// way they are, check this wiki: https://www.nesdev.org/wiki/CPU
pub struct CPU {
    acc: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    pub status: CpuStatus,

    sys_ram: Memory,
    prg_rom: Memory,
    ppu_regs: Rc<PpuRegisters>,
    mapper: Rc<dyn Mapper>,

    clocks: usize,

    current_instr: Instruction,
    instr_data: OpcodeData,
}

impl CPU {
    /// Make a new CPU with access to given program memory and PPU Registers and a given mapper
    pub fn new(prg_rom: Memory, ppu_regs: Rc<PpuRegisters>, mapper: Rc<dyn Mapper>) -> Self {
        let mut new_cpu = CPU{
            acc: 0,
            x: 0,
            y: 0,
            sp: 0, // will be set to 0xFD in reset due to wrapping sub
            pc: 0,
            status: CpuStatus::from_bits(0x20), // start w/ unused flag on cuz why not ig (fixes nesdev tests)

            sys_ram: Memory::new(0x800), // NES has 2KiB of internal memory that only the CPU can access
            prg_rom: prg_rom,
            ppu_regs: Rc::clone(&ppu_regs),
            mapper: Rc::clone(&mapper),

            clocks: 0,
    
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
    /// the fetch, decode, and execute stages of the CPU. Returns the total
    /// number of clock cycles taken for the instruction run.
    pub fn cycle(&mut self) {
        let opcode = self.read(self.pc);

        // fetch - get the opcode we are running
        let instr = &INSTRUCTION_TABLE[opcode as usize];

        // decode - retrieve the neccesary data for the instruction
        let (opcode_data, fetch_cycles) = (instr.addr_func)(self);

        // Increment pc before instruction execution
        self.pc += instr.bytes as u16;

        // execute - run the instruction, updating memory and processor status
        //           as defined by the instruction
        let execute_cycles = (instr.func)(self, opcode_data);

        // store the instruction just executed (for debugging)
        self.current_instr = instr.clone();
        self.instr_data = opcode_data;

        self.clocks += execute_cycles + instr.base_clocks;

        if instr.has_extra_fetch_cycles {
            self.clocks += fetch_cycles;
        }
    }

    // GETTER/SETTER FUNCTIONS

    /// Get the current number of clock cycles since turn-on
    pub fn clocks(&self) -> usize {
        self.clocks
    }

    /// Manually set the number of clock cycles since turn-on
    pub fn set_clocks(&mut self, clks: usize) {
        self.clocks = clks;
    }
    
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
                self.ppu_regs.read(address & 0x0007)
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
        match address {
            0x0000..=0x1FFF => {
                // First 2KiB of memory (0x0800) are mirrored until 0x2000
                self.sys_ram.write(address & 0x07FF, data);
            },
            0x2000..=0x3FFF => {
                // PPU Registers mirrored over 8KiB
                self.ppu_regs.write(address & 0x0007, data);
            }
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

        self.clocks += 7;
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
            self.clocks += 7;
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
        self.clocks += 7;
    }

    /// Get the instruction just executed as a string of 6502 assembly for
    /// debugging purposes.
    pub fn current_instr_str(&self) -> String {
        let mut out_str = format!("0x{:02X}", self.current_instr.opcode_num);
        out_str.push(' ');
        out_str.push_str(self.current_instr.name);

        out_str.push(' ');
        let temp = if self.current_instr.name == "???" {
            if self.current_instr.opcode_num == 0x00 {
                String::from("none : [---]")
            } else {
                format!(" : [illegal op #{:02X}]", self.current_instr.opcode_num)
            }
        } else {
            match self.current_instr.addr_mode {
                AddressingMode::Accumulator => String::from("A : [acc]"),

                AddressingMode::Implied => String::from(": [imp]"),

                AddressingMode::Immediate => {
                    format!("#${:02X} : [imm]", self.instr_data.data.unwrap())
                }

                AddressingMode::Absolute => {
                    format!("${:04X} : [abs]", self.instr_data.address.unwrap())
                }
                AddressingMode::AbsoluteX => {
                    format!("${:04X},X : [abs x]", self.instr_data.address.unwrap())
                }
                AddressingMode::AbsoluteY => {
                    format!("${:04X},Y : [abs y]", self.instr_data.address.unwrap())
                }

                AddressingMode::ZeroPage => {
                    format!("${:02X} : [zpage]", self.instr_data.address.unwrap())
                }
                AddressingMode::ZeroPageX => {
                    format!("${:02X},X : [zpage x]", self.instr_data.address.unwrap())
                }
                AddressingMode::ZeroPageY => {
                    format!("${:02X},Y : [zpage y]", self.instr_data.address.unwrap())
                }

                AddressingMode::Indirect => {
                    format!("$(??) : [ind, abs_addr = 0x{:04X}]", self.instr_data.address.unwrap())
                }
                AddressingMode::IndirectX => {
                    format!("$(??),X : [ind x, abs_addr = 0x{:04X}]", self.instr_data.address.unwrap())
                }
                AddressingMode::IndirectY => {
                    format!("$(??),Y : [ind y, abs_addr = 0x{:04X}]", self.instr_data.address.unwrap())
                }

                AddressingMode::Relative => format!(
                    "${:02X} : [rel, offset = {}]",
                    self.instr_data.offset.unwrap() as u8,
                    self.instr_data.offset.unwrap()
                ),
            }
        };
        out_str.push_str(&temp);

        if self.current_instr.is_illegal {
            out_str.push_str(" <ILLEGAL>");
        }

        out_str
    }

    pub fn print_state(&self) {
        println!("CPU State:");
        println!("  A: 0x{:02X}, X: 0x{:02X}, Y: 0x{:02X}", self.acc, self.x, self.y);
        println!("  SP: 0x{:02X}, PC: 0x{:04X}", self.sp, self.pc);
        println!("  Status (NVUBDIZC): {:08b}", self.get_status());
        println!("  Last Instr: {}", self.current_instr_str());
        println!("  Total Clks: {}", self.clocks);
    }
}

#[cfg(test)]
mod tests {
    // use std::fs;
    // use std::io::Read;

    // use super::CPU;

    // fn load_raw_mem_to_cpu(cpu: &CPU, path_str: &str) {
    //     let mut mem_file = fs::File::open(path_str).unwrap();
    //     let mut data: Vec<u8> = Vec::new();
    //     mem_file.read_to_end(&mut data).unwrap();

    //     data.into_iter()
    //         .enumerate()
    //         .for_each(|(addr, byte)| cpu.write(addr as u16, byte));
    // }

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

