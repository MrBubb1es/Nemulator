use std::borrow::Borrow;

use super::instructions::{self, AddressingMode, Instruction, OpcodeData, INSTRUCTION_TABLE};

use super::bus::Bus;

pub struct CPU<'a> {
    acc: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    flags: u8,

    bus: &'a Bus,

    current_instr: Instruction,
    instr_data: OpcodeData,
}
impl<'a> CPU<'a> {
    pub fn new(bus: &'a Bus) -> Self {
        CPU {
            acc: 0,
            x: 0,
            y: 0,
            sp: 0,
            pc: 0,
            flags: 0,
            bus: bus,
            current_instr: instructions::DEFAULT_ILLEGAL_OP,
            instr_data: instructions::OpcodeData{data: None, address: None, offset: None},
        }
    }

    pub fn cycle(&mut self, opcode: u8) -> usize {
        let instr = &instructions::INSTRUCTION_TABLE[opcode as usize];

        let (opcode_data, fetch_cycles) = (instr.addr_func)(self);
        let execute_cycles = (instr.func)(self, opcode_data);
        self.pc += instr.bytes as u16;

        self.current_instr = instr.clone();
        self.instr_data = opcode_data;

        execute_cycles + fetch_cycles + instr.base_clocks
    }

    // GETTER/SETTER FUNCTIONS
    pub fn get_carry_flag(&self) -> u8 {
        self.flags & 0x01
    }
    pub fn get_zero_flag(&self) -> u8 {
        (self.flags & 0x02) >> 1
    }
    pub fn get_interrupt_flag(&self) -> u8 {
        (self.flags & 0x04) >> 2
    }
    pub fn get_decimal_flag(&self) -> u8 {
        (self.flags & 0x08) >> 3
    }
    pub fn get_b_flag(&self) -> u8 {
        (self.flags & 0x10) >> 4
    }
    pub fn get_unused_flag(&self) -> u8 {
        (self.flags & 0x20) >> 5
    }
    pub fn get_overflow_flag(&self) -> u8 {
        (self.flags & 0x40) >> 6
    }
    pub fn get_negative_flag(&self) -> u8 {
        (self.flags & 0x80) >> 7
    }
    pub fn get_flags(&self) -> u8 {
        self.flags
    }

    pub fn set_carry_flag(&mut self, val: u8) {
        self.flags = (self.flags & 0xFE) | (val << 0);
    }
    pub fn set_zero_flag(&mut self, val: u8) {
        self.flags = (self.flags & 0xFD) | (val << 1);
    }
    pub fn set_interrupt_flag(&mut self, val: u8) {
        self.flags = (self.flags & 0xFB) | (val << 2);
    }
    pub fn set_decimal_flag(&mut self, val: u8) {
        self.flags = (self.flags & 0xF7) | (val << 3);
    }
    pub fn set_b_flag(&mut self, val: u8) {
        self.flags = (self.flags & 0xEF) | (val << 4);
    }
    pub fn set_unused_flag(&mut self, val: u8) {
        self.flags = (self.flags & 0xDF) | (val << 5);
    }
    pub fn set_overflow_flag(&mut self, val: u8) {
        self.flags = (self.flags & 0xBF) | (val << 6);
    }
    pub fn set_negative_flag(&mut self, val: u8) {
        self.flags = (self.flags & 0x7F) | (val << 7);
    }
    pub fn set_flags(&mut self, val: u8) {
        self.flags = val;
    }

    pub fn get_acc(&self) -> u8 {
        self.acc
    }
    pub fn get_x_reg(&self) -> u8 {
        self.x
    }
    pub fn get_y_reg(&self) -> u8 {
        self.y
    }
    pub fn get_sp(&self) -> u8 {
        self.sp
    }
    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn set_acc(&mut self, val: u8) {
        self.acc = val;
    }
    pub fn set_x_reg(&mut self, val: u8) {
        self.x = val;
    }
    pub fn set_y_reg(&mut self, val: u8) {
        self.y = val;
    }
    pub fn set_sp(&mut self, val: u8) {
        self.sp = val;
    }
    pub fn set_pc(&mut self, val: u16) {
        self.pc = val;
    }

    // HELPER FUNCTIONS
    pub fn read(&self, address: u16) -> u8 {
        self.bus.read(address)
    }
    pub fn write(&self, address: u16, data: u8) {
        self.bus.write(address, data)
    }
    // Read a 2 byte value starting at address in LLHH (little-endian) form
    pub fn read_word(&self, address: u16) -> u16 {
        let lo = self.bus.read(address) as u16;
        let hi = self.bus.read(address + 1) as u16;
        (hi << 8) | lo
    }
    // Write a 2 byte value to a given memory address in LLHH form
    pub fn write_word(&self, address: u16, data: u16) {
        let lo = data as u8;
        let hi = (data >> 8) as u8;
        self.bus.write(address, lo);
        self.bus.write(address + 1, hi);
    }
    // Read a 2 byte value from the zero-page of memory. If the address being read from is 0xFF,
    // then the high byte will be taken from address 0x00 (wrap around zero-page)
    pub fn read_zpage_word(&self, zpage_address: u8) -> u16 {
        let lo = self.bus.read(zpage_address as u16) as u16;
        let hi = self.bus.read(zpage_address.wrapping_add(1) as u16) as u16;
        (hi << 8) | lo
    }
    // Write a 2 byte value to the z-page in memory at given address. If address is 0xFF, the
    // second byte will be written to 0x00 (wrapping z-page).
    pub fn write_zpage_word(&self, zpage_address: u8, data: u16) {
        let lo = data as u8;
        let hi = (data >> 8) as u8;
        self.bus.write(zpage_address as u16, lo);
        self.bus.write(zpage_address.wrapping_add(1) as u16, hi);
    }
    pub fn push_to_stack(&mut self, data: u8) {
        let stk_address = 0x0100 | self.sp as u16;
        self.bus.write(stk_address, data);
        self.sp = self.sp.wrapping_sub(1);
    }
    pub fn pop_from_stack(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let stk_address = 0x0100 | self.sp as u16;
        self.bus.read(stk_address)
    }

    // RESET FUNCTION
    pub fn reset(&mut self) {
        const STK_RESET: u8 = 0xFD;
        const PC_RESET_ADDR: u16 = 0xFFFC;

        self.acc = 0;
        self.x = 0;
        self.y = 0;
        self.sp = STK_RESET;

        self.pc = self.read_word(PC_RESET_ADDR);
    }

    // Interrupt Request
    pub fn irq(&mut self) {
        // Check interrupt disable flag
        if self.get_interrupt_flag() == 0 {
            const IRQ_PC_VECTOR: u16 = 0xFFFE;

            // Store PC
            let lo = self.pc as u8;
            let hi = (self.pc >> 8) as u8;
            self.push_to_stack(lo);
            self.push_to_stack(hi);

            // Set flags and store status
            self.set_b_flag(0);
            self.set_unused_flag(1);
            self.set_interrupt_flag(1);
            self.push_to_stack(self.flags);

            // Set PC to whatever is at addr 0xFFFE
            self.pc = self.read_word(IRQ_PC_VECTOR);
        }
    }
    // Non-Maskable Interrupt
    pub fn nmi(&mut self) {
        const NMI_PC_VECTOR: u16 = 0xFFFA;

        // Store PC
        let lo = self.pc as u8;
        let hi = (self.pc >> 8) as u8;
        self.push_to_stack(lo);
        self.push_to_stack(hi);

        // Set flags and store status
        self.set_b_flag(0);
        self.set_unused_flag(1);
        self.set_interrupt_flag(1);
        self.push_to_stack(self.flags);

        // Set PC to whatever is at addr 0xFFFE
        self.pc = self.read_word(NMI_PC_VECTOR);
    }

    pub fn current_instr_str(&self) -> String {
        let mut out_str = String::from(self.current_instr.name);

        // Accumulator,
        // Implied,
        // Immediate,
        // Absolute,
        // AbsoluteX,
        // AbsoluteY,
        // ZeroPage,
        // ZeroPageX,
        // ZeroPageY,
        // Indirect,
        // IndirectX,
        // IndirectY,
        // Relative,
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
                
                AddressingMode::Implied => String::from(" : [imp]"),
                
                AddressingMode::Immediate => format!("#${:02X} : [imm]", self.instr_data.data.unwrap()),
                
                AddressingMode::Absolute => format!("${:04X} : [abs]", self.instr_data.address.unwrap()),
                AddressingMode::AbsoluteX => format!("${:04X},X : [abs x]", self.instr_data.address.unwrap()),
                AddressingMode::AbsoluteY => format!("${:04X},Y : [abs y]", self.instr_data.address.unwrap()),

                AddressingMode::ZeroPage => format!("${:02X} : [zpage]", self.instr_data.address.unwrap()), 
                AddressingMode::ZeroPageX => format!("${:02X},X : [zpage x]", self.instr_data.address.unwrap()),
                AddressingMode::ZeroPageY => format!("${:02X},Y : [zpage y]", self.instr_data.address.unwrap()),

                AddressingMode::Indirect => format!("$({:02X}) : [ind]", self.instr_data.address.unwrap()), 
                | AddressingMode::IndirectX => format!("$({:02X}),X : [ind x]", self.instr_data.address.unwrap()),
                | AddressingMode::IndirectY => format!("$({:02X}),Y : [ind y]", self.instr_data.address.unwrap()),

                AddressingMode::Relative => format!("${:02X} : [rel, offset = {}]", self.instr_data.offset.unwrap() as u8, self.instr_data.offset.unwrap()),
            }
        };
        out_str.push_str(&temp);

        out_str
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::fs;

    use super::Bus;
    use super::CPU;

    fn load_raw_mem_to_cpu(cpu: &CPU, path_str: &str) {
        let mut mem_file = fs::File::open(path_str).unwrap();
        let mut data: Vec<u8> = Vec::new();
        mem_file.read_to_end(&mut data).unwrap();

        data.into_iter().enumerate()
            .for_each(|(addr, byte)|
                cpu.write(addr as u16, byte)
        );
    }

    #[test]
    // Test program that multiplies 3 and 10 and stores the result in Accumulator
    fn test_multiply() {
        let test_file = "prg_tests/cpu_tests/test_multiply.bin";
        let test_bus = Bus::new();
        let mut test_cpu = CPU::new(&test_bus);

        load_raw_mem_to_cpu(&test_cpu, &test_file);

        // Just make sure the program loaded correctly
        // (only checking a couple bytes, one at the start and one near the end)
        // assert_eq!(test_cpu.read(0x0000), 0xA9);
        // assert_eq!(test_cpu.read(0x0010), 0x00);

        // test_cpu.reset();

        crate::run_debug(&mut test_cpu, &test_bus);
    }
}