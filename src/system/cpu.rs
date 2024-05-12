use super::bus::Bus;

pub struct CPU<'a> {
    acc: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    flags: u8,

    bus: &'a Bus,
}
impl CPU<'_> {
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
    pub fn get_blank_flag(&self) -> u8 {
        (self.flags & 0x20) >> 5
    }
    pub fn get_overflow_flag(&self) -> u8 {
        (self.flags & 0x40) >> 6
    }
    pub fn get_negative_flag(&self) -> u8 {
        (self.flags & 0x80) >> 7
    }

    fn set_carry_flag(&mut self, val: u8) {
        self.flags |= val
    }
    fn set_zero_flag(&mut self, val: u8) {
        self.flags |= val << 1
    }
    fn set_interrupt_flag(&mut self, val: u8) {
        self.flags |= val << 2
    }
    fn set_decimal_flag(&mut self, val: u8) {
        self.flags |= val << 3
    }
    fn set_b_flag(&mut self, val: u8) {
        self.flags |= val << 4
    }
    fn set_blank_flag(&mut self, val: u8) {
        self.flags |= val << 5
    }
    fn set_overflow_flag(&mut self, val: u8) {
        self.flags |= val << 6
    }
    fn set_negative_flag(&mut self, val: u8) {
        self.flags |= val << 7
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

    // fn set_acc(&self, val: u8) { self.acc = val }
    // fn set_x_reg(&self, val: u8) { self.x = val }
    // fn set_y_reg(&self, val: u8) { self.y = val }
    // fn set_sp(&self, val: u8) { self.sp = val }
    // fn set_pc(&self, val: u16) { self.pc = val }

    // HELPER FUNCTIONS
    // Read a 2 byte value starting at address in LLHH (little-endian) form
    fn read_word(&self, address: u16) -> u16 {
        let lo = self.bus.read(address) as u16;
        let hi = self.bus.read(address + 1) as u16;
        (hi << 8) | lo
    }
    // Write a 2 byte value to a given memory address in LLHH form
    fn write_word(&self, address: u16, data: u16) {
        let lo = data as u8;
        let hi = (data >> 8) as u8;
        self.bus.write(address, lo);
        self.bus.write(address + 1, hi);
    }
    // Read a 2 byte value from the zero-page of memory. If the address being read from is 0xFF,
    // then the high byte will be taken from address 0x00 (wrap around zero-page)
    fn read_zpage_word(&self, zpage_address: u8) -> u16 {
        let lo = self.bus.read(zpage_address as u16) as u16;
        let hi = self.bus.read(zpage_address.wrapping_add(1) as u16) as u16;
        (hi << 8) | lo
    }
    // Write a 2 byte value to the z-page in memory at given address. If address is 0xFF, the
    // second byte will be written to 0x00 (wrapping z-page).
    fn write_zpage_word(&self, zpage_address: u8, data: u16) {
        let lo = data as u8;
        let hi = (data >> 8) as u8;
        self.bus.write(zpage_address as u16, lo);
        self.bus.write(zpage_address.wrapping_add(1) as u16, hi);
    }
    fn push_to_stack(&mut self, data: u8) {
        let stk_address = 0x0100 | self.sp as u16;
        self.bus.write(stk_address, data);
        self.sp = self.sp.wrapping_sub(1);
    }
    fn pop_from_stack(&mut self) -> u8 {
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

    // ADDRESSING MODES - Fetches data from the bus
    // Returns: Tuple of (data, address, cycles) where:
    //  - data: Single byte read from memory
    //  - address: 2 byte address that data was read from
    //  - cycles: Number of clock cycles taken to read the data from memory
    //
    // Note: Implied addressing mode has no return type because no data is needed for instructions
    // with implied addressing mode.

    // Implied - no extra data needed for this instruction, read no extra bytes
    fn implied(&self) {}
    // Immediate - data immediatly follows instruction, always 2 bytes
    fn immediate(&self) -> u16 {
        self.read_word(self.pc + 1)
    }
    // Absolute - The next 2 bytes are the address in memory of the data to retrieve
    fn absolute(&self) -> u8 {
        let abs_address = self.read_word(self.pc + 1);
        self.bus.read(abs_address)
    }
    // Zero Page - Like absolute, but uses only 1 byte for address & uses 0x00 for the high byte of the address
    fn zero_page(&self) -> u8 {
        let address_lo = self.bus.read(self.pc + 1) as u16;
        let zpage_address = address_lo;
        self.bus.read(zpage_address)
    }
    // Indexed Addressing (X) - Like Immediate, but adds the x register to the abs address to get
    // the "effective address," and uses that to fetch data from memory
    fn indexed_x(&self) -> u8 {
        let abs_address = self.read_word(self.pc + 1);
        let effective_address = abs_address + self.x as u16;
        self.bus.read(effective_address)
    }
    // Indexed Addressing (Y) - Like same as Indexed x, but used the y register instead
    fn indexed_y(&self) -> u8 {
        let abs_address = self.read_word(self.pc + 1);
        let effective_address = abs_address + self.y as u16;
        self.bus.read(effective_address)
    }
    // Indexed Addressing Zero-Page (X) - Like Indexed x, but uses only the single next byte as the
    // low order byte of the absolute address and fills the top byte w/ 0x00. Then adds x to get
    // the effective address. Note that the effective address will never go off the zero-page, if
    // the address exceeds 0x00FF, it will loop back around to 0x0000.
    fn indexed_zpage_x(&self) -> u8 {
        let zpage_address = self.bus.read(self.pc + 1);
        let effective_zpage_address = zpage_address.wrapping_add(self.x) as u16;
        self.bus.read(effective_zpage_address)
    }
    // Indexed Addressing Zero-Page (Y) - Like Indexed Z-Page x, but uses the y register instead
    fn indexed_zpage_y(&self) -> u8 {
        let zpage_address = self.bus.read(self.pc + 1);
        let effective_zpage_address = zpage_address.wrapping_add(self.y) as u16;
        self.bus.read(effective_zpage_address)
    }
    // Indirect Addressing - Uses the next 2 bytes as the abs address, then reads the byte that
    // points to in memory and the one after (in LLHH order) and uses those as the effective
    // address where the data will be read from
    fn indirect(&self) -> u8 {
        let abs_address = self.read_word(self.pc + 1);
        let effective_address = self.read_word(abs_address);
        self.bus.read(effective_address)
    }
    // Pre-Indexed Indirect Zero-Page (X) - Like Indexed Z-Page x, but as in Indirect addressing, another
    // address is read from memory instead of the data itself. The address read is the effective
    // address of the actual data. Note that if the z-page address is 0x00FF, then the bytes at
    // 0x00FF and 0x0000 are read and used as low and high bytes of the effective address, respectively
    fn preindexed_zpage_x(&self) -> u8 {
        let zpage_address = self.bus.read(self.pc + 1).wrapping_add(self.x);
        let effective_address = self.read_zpage_word(zpage_address);
        self.bus.read(effective_address)
    }
    // Post-Indexed Indirect Zero-Page (Y) - Like Indirect Indexed Z-Page x, but with two major differences:
    // First, the y register is used instead of x. Second, the register is added to the address
    // retrieved from the z-page, not the address used to access the z-page.
    fn postindexed_zpage_y(&self) -> u8 {
        let zpage_address = self.bus.read(self.pc + 1);
        let effective_address = self.read_zpage_word(zpage_address) + self.y as u16;
        self.bus.read(effective_address)
    }
    // Relative Addressing - Gets the next bytes as a signed byte to be added to the pc for branch
    // instructions.
    fn relative(&self) -> i8 {
        self.bus.read(self.pc + 1) as i8
    }

    // OPCODES - all the cpu instructions
    // Args:
    //  - data: Single byte of data read from memory. Where data comes from is determined by the
    //          addressing mode of the instruction.
    //  - address: Some instructions also need to write to memory, so the address that data was
    //             taken from can also be included as an argument.
    //
    // Returns:
    //  - Number of clock cycles taken by the instruction

    // ADC - Add Memory to Accumulator with Carry
    fn adc(&mut self, data: u8) {
        let result = (data as u16) + (self.acc as u16) + (self.get_carry_flag() as u16);
        self.set_carry_flag(if result & 0xFF00 > 0 { 1 } else { 0 });
        self.set_zero_flag(if result & 0xFF == 0 { 1 } else { 0 });
        self.acc = result as u8;
        // NOTE: come back and do the overflow (V) flag once you know how it works
        // NOTE: and the negative flag bc I can't be bothered to rn
    }
    // AND - AND Memory with Accumulator
    fn and(&mut self, data: u8) {
        let result = self.acc & data;
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 > 0 { 1 } else { 0 });
    }
    // ASL - Shift Left One Bit (Memory or Accumulator)
    fn asl() {
        // skipping for now
    }
    // BCC - Branch on Carry Clear
    fn bcc(&mut self, offset: i8) {
        if self.get_carry_flag() != 1 {
            self.pc = (self.pc as i32 + offset as i32) as u16;
        }
    }
    // BCS - Branch on Carry Set
    fn bcs(&mut self, offset: i8) {
        if self.get_carry_flag() == 1 {
            self.pc = (self.pc as i32 + offset as i32) as u16;
        }
    }
    // BEQ - Branch on Equal (Zero flag set)
    fn beq(&mut self, offset: i8) {
        if self.get_zero_flag() == 1 {
            self.pc = (self.pc as i32 + offset as i32) as u16;
        }
    }
    // BIT - Test Bits in Memory with Accumulator
    fn bit(&mut self, data: u8) {
        self.set_negative_flag(if data & 0x80 > 0 { 1 } else { 0 });
        self.set_overflow_flag(if data & 0x40 > 0 { 1 } else { 0 });
        self.set_zero_flag(if data & self.acc == 0 { 1 } else { 0 });
    }
    // BMI - Branch on Result Minus (Negative flag set)
    fn bmi(&mut self, offset: i8) {
        if self.get_negative_flag() == 1 {
            self.pc = (self.pc as i32 + offset as i32) as u16;
        }
    }
    // BNE - Branch on Not Equal (Zero flag NOT set)
    fn bne(&mut self, offset: i8) {
        if self.get_zero_flag() == 0 {
            self.pc = (self.pc as i32 + offset as i32) as u16;
        }
    }
    // BPL - Branch on Result Plus (Negative flag NOT set)
    fn bpl(&mut self, offset: i8) {
        if self.get_negative_flag() == 0 {
            self.pc = (self.pc as i32 + offset as i32) as u16;
        }
    }
    // BRK - Force Break (Initiate interrupt)
    fn brk(&mut self) {
        //TODO:
    }
    // BVC - Branch on Overflow clear
    fn bvc(&mut self, offset: i8) {
        if self.get_overflow_flag() == 0 {
            self.pc = (self.pc as i32 + offset as i32) as u16;
        }
    }
    // BVS - Branch on Overflow set
    fn bvs(&mut self, offset: i8) {
        if self.get_overflow_flag() == 1 {
            self.pc = (self.pc as i32 + offset as i32) as u16;
        }
    }
    // CLC - Clear Carry Flag
    fn clc(&mut self) {
        self.set_carry_flag(0);
    }
    // CLD - Clear Decimal Mode
    fn cld(&mut self) {
        self.set_decimal_flag(0);
    }
    // CLI - Clear Interrupt Disable Bit
    fn cli(&mut self) {
        self.set_interrupt_flag(0);
    }
    // CLV - Clear Overflow Flag
    fn clv(&mut self) {
        self.set_overflow_flag(0);
    }
    // CMP - Compare Memory with Accumulator
    fn cmp(&mut self, data: u8) {
        let result = (self.acc as i16) - (data as i16);
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        self.set_carry_flag(if result >= 0 { 1 } else { 0 });
    }
    // CPX - Compare Memory and Index X
    fn cpx(&mut self, data: u8) {
        let result = (self.x as i16) - (data as i16);
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        self.set_carry_flag(if result >= 0 { 1 } else { 0 });
    }
    // CPY - Compare Memory and Index Y
    fn cpy(&mut self, data: u8) {
        let result = (self.y as i16) - (data as i16);
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        self.set_carry_flag(if result >= 0 { 1 } else { 0 });
    }
    // DEC - Decrement Memory
    fn dec(&mut self, data: u8, address: u16) {
        let result = data.wrapping_sub(1);
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        self.bus.write(address, result);
    }
    // DEX - Decrement X Register
    fn dex(&mut self) {
        let result = self.x.wrapping_sub(1);
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        self.x = result;
    }
    // DEY - Decrement Y Register
    fn dey(&mut self) {
        let result = self.y.wrapping_sub(1);
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        self.x = result;
    }
    // EOR - Exclusive OR
    fn eor(&mut self, data: u8) {
        let result = self.acc ^ data;
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        self.acc = result;
    }
    // INC - Increment Memory
    fn inc(&mut self, data: u8, address: u16) {
        let result = data.wrapping_add(1);
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        self.bus.write(address, result);
    }
    // INX - Increment X Register
    fn inx(&mut self) {
        let result = self.x.wrapping_add(1);
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        self.x = result;
    }
    // INY - Increment Y Register
    fn iny(&mut self) {
        let result = self.y.wrapping_add(1);
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        self.y = result;
    }
    // JMP - Jump
    fn jmp(&mut self, address: u16) {
        self.pc = address;
    }
    // JSR - Jump to Subroutine
    fn jsr(&mut self, address: u16) {
        let return_point = self.pc + 2; // Return point is the return address - 1, and since this
                                        // instruction will always use absolute addressing mode, the whole instruction is 3 bytes.
        let lo = return_point as u8;
        let hi = (return_point >> 8) as u8;
        self.push_to_stack(lo);
        self.push_to_stack(hi);
    }
    // LDA - Load Accumulator
    fn lda(&mut self, data: u8) {
        self.set_zero_flag(if data == 0 { 1 } else { 0 });
        self.set_negative_flag(if data & 0x80 != 0 { 1 } else { 0 });
        self.acc = data;
    }
    // LDX - Load X Register
    fn ldx(&mut self, data: u8) {
        self.set_zero_flag(if data == 0 { 1 } else { 0 });
        self.set_negative_flag(if data & 0x80 != 0 { 1 } else { 0 });
        self.x = data;
    }
    // LDY - Load Y Register
    fn ldy(&mut self, data: u8) {
        self.set_zero_flag(if data == 0 { 1 } else { 0 });
        self.set_negative_flag(if data & 0x80 != 0 { 1 } else { 0 });
        self.y = data;
    }
    // LSR - Logical Shift Right (Accumulator version)
    fn lsr_acc(&mut self) {
        let result = self.acc >> 1;
        self.set_carry_flag(self.acc & 0x01);
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(0); // result will always have bit 7 == 0
        self.acc = result;
    }
    // LSR - Logical Shift Right (Memory version)
    fn lsr_mem(&mut self, data: u8, address: u16) {
        let result = data >> 1;
        self.set_carry_flag(data & 0x01);
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(0); // result will always have bit 7 == 0
        self.bus.write(address, result);
    }
    // NOP - No Operation
    fn nop(&self) {}
    // ORA - Logical Inclusive OR
    fn ora(&mut self, data: u8) {
        let result = self.acc | data;
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        self.acc = result;
    }
    // PHA - Push Accumulator
    fn pha(&mut self) {
        self.push_to_stack(self.acc);
    }
    // PHP - Push Processor Status
    fn php(&mut self) {
        self.push_to_stack(self.flags);
    }
    // PLA - Pull Accumulator
    fn pla(&mut self) {
        let result = self.pop_from_stack();
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        self.acc = result;
    }
    // PLP - Pull Processor Status
    fn plp(&mut self) {
        self.flags = self.pop_from_stack();
    }
    // ROL - Rotate Left (Accumulator version)
    fn rol_acc(&mut self) {
        let result = (self.acc << 1) | self.get_carry_flag();
        self.set_carry_flag(self.acc >> 7); // old bit 7 becomes new carry
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        self.acc = result;
    }
    // ROL - Rotate Left (Memory version)
    fn rol_mem(&mut self, data: u8, address: u16) {
        let result = (data << 1) | self.get_carry_flag();
        self.set_carry_flag(data >> 7); // old bit 7 becomes new carry
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        self.bus.write(address, result);
    }
    // ROR - Rotate Right (Accumulator version)
    fn ror_acc(&mut self) {
        let result = (self.get_carry_flag() << 7) | (self.acc >> 1);
        self.set_carry_flag(self.acc & 0x01); // old bit 0 becomes new carry
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        self.acc = result;
    }
    // ROR - Rotate Right (Memory version)
    fn ror_mem(&mut self, data: u8, address: u16) {
        let result = (self.get_carry_flag() << 7) | (data >> 1);
        self.set_carry_flag(data & 0x01); // old bit 0 becomes new carry
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        self.bus.write(address, result);
    }
    // RTI - Return from Interrupt
    fn rti(&mut self) {
        //TODO:
    }
    // RTS - Return from Subroutine
    fn rts(&mut self) {
        let hi = self.pop_from_stack() as u16;
        let lo = self.pop_from_stack() as u16;
        self.pc = (hi << 8) | lo;
    }
    // SBC - Subtract with Carry
    fn sbc(&mut self, data: u8) {
        //  result = A - M - (1 - C) == A - (M + 1 - C) == A - rhs
        let rhs = (data + 1 - self.get_carry_flag()) as u16;
        let result = (self.acc as u16) - rhs;
        self.set_carry_flag(if result & 0xFF00 != 0 { 1 } else { 0 });
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        // Overflow if A < rhs
        self.set_overflow_flag(if (self.acc as u16) < rhs as u16 { 1 } else { 0 });
        self.acc = result as u8;
    }
    // SEC - Set Carry Flag
    fn sec(&mut self) {
        self.set_carry_flag(1);
    }
    // SED - Set Decimal Flag
    fn sed(&mut self) {
        self.set_decimal_flag(1);
    }
    // SEI - Set Interrupt Disable
    fn sei(&mut self) {
        self.set_interrupt_flag(1);
    }
    // STA - Store Accumulator
    fn sta(&self, address: u16) {
        self.bus.write(address, self.acc);
    }
    // STX - Store X Register
    fn stx(&self, address: u16) {
        self.bus.write(address, self.x);
    }
    // STY - Store Y Register
    fn sty(&self, address: u16) {
        self.bus.write(address, self.y);
    }
    // TAX - Transfer Accumulator to X
    fn tax(&mut self) {
        self.x = self.acc;
        self.set_zero_flag(if self.x == 0 { 1 } else { 0 });
        self.set_negative_flag(if self.x & 0x80 != 0 { 1 } else { 0 });
    }
    // TAY - Transfer Accumulator to Y
    fn tay(&mut self) {
        self.y = self.acc;
        self.set_zero_flag(if self.y == 0 { 1 } else { 0 });
        self.set_negative_flag(if self.y & 0x80 != 0 { 1 } else { 0 });
    }
    // TSX - Transfer Stack Pointer to X
    fn tsx(&mut self) {
        self.x = self.sp;
        self.set_zero_flag(if self.x == 0 { 1 } else { 0 });
        self.set_negative_flag(if self.x & 0x80 != 0 { 1 } else { 0 });
    }
    // TXA - Transfer X to Accumulator
    fn txa(&mut self) {
        self.acc = self.x;
        self.set_zero_flag(if self.acc == 0 { 1 } else { 0 });
        self.set_negative_flag(if self.acc & 0x80 != 0 { 1 } else { 0 });
    }
    // TXS - Transfer X to Stack Pointer
    fn txs(&mut self) {
        self.sp = self.x;
    }
    // TYA - Transfer Y to Accumulator
    fn tya(&mut self) {
        self.acc = self.y;
        self.set_zero_flag(if self.acc == 0 { 1 } else { 0 });
        self.set_negative_flag(if self.acc & 0x80 != 0 { 1 } else { 0 });
    }
}

#[cfg(test)]
mod tests {}
