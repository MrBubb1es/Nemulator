pub struct CPU {
    acc: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    flags: u8,
}

impl CPU {
    fn get_carry_flag(&self) -> u8 {
        self.flags & 0x01
    }
    fn get_zero_flag(&self) -> u8 {
        (self.flags & 0x02) >> 1
    }
    fn get_interrupt_flag(&self) -> u8 {
        (self.flags & 0x04) >> 2
    }
    fn get_decimal_flag(&self) -> u8 {
        (self.flags & 0x08) >> 3
    }
    fn get_b_flag(&self) -> u8 {
        (self.flags & 0x10) >> 4
    }
    fn get_blank_flag(&self) -> u8 {
        (self.flags & 0x20) >> 5
    }
    fn get_overflow_flag(&self) -> u8 {
        (self.flags & 0x40) >> 6
    }
    fn get_negative_flag(&self) -> u8 {
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

    fn get_acc(&self) -> u8 {
        self.acc
    }
    fn get_x_reg(&self) -> u8 {
        self.x
    }
    fn get_y_reg(&self) -> u8 {
        self.y
    }
    fn get_sp(&self) -> u8 {
        self.sp
    }
    fn get_pc(&self) -> u16 {
        self.pc
    }

    fn set_acc(&self, val: u8) {
        self.acc = val
    }
    fn set_x_reg(&self, val: u8) {
        self.x = val
    }
    fn set_y_reg(&self, val: u8) {
        self.y = val
    }
    fn set_sp(&self, val: u8) {
        self.sp = val
    }
    fn set_pc(&self, val: u16) {
        self.pc = val
    }

    // OPCODES - all the cpu instructions
    fn adc(&mut self, arg: u8) {
        let result = (arg as u16) + (self.get_acc() as u16) + (self.get_carry() as u16);
        self.set_acc((result & 0xFF) as u8);
        self.set_carry_flag(if result & 0xFF00 > 0 { 1 } else { 0 });
        self.set_zero_flag(if result & 0xFF == 0 { 1 } else { 0 });
        // NOTE: come back and do the overflow (V) flag once you know how it works
        // NOTE: and the negative flag bc I can't be bothered to rn
    }

    fn and(&mut self, arg: u8) {
        let result = self.get_acc() & arg;
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 > 0 { 1 } else { 0 });
    }

    fn asl() {
        // skipping for now
    }

    fn bcc(&mut self, offset: i8) {
        if self.get_carry_flag() != 1 {
            self.set_pc((self.get_pc() as i32 + offset as i32) as u16);
        }
    }

    fn bcs(&mut self, offset: i8) {
        if self.get_carry_flag() == 1 {
            self.set_pc((self.get_pc() as i32 + offset as i32) as u16);
        }
    }

    fn beq(&mut self, offset: i8) {
        if self.get_zero_flag() == 1 {
            self.set_pc((self.get_pc() as i32 + offset as i32) as u16);
        }
    }

    fn bit(&mut self, arg: u8) {
        self.set_negative_flag(if arg & 0b10000000 > 0 { 1 } else { 0 });
        self.set_overflow_flag(if arg & 0b01000000 > 0 { 1 } else { 0 });
        self.set_zero_flag(if arg & self.get_acc() == 0 { 1 } else { 0 });
    }

    fn bmi(&mut self, offset: i8) {
        if self.get_negative_flag() == 1 {
            self.set_pc((self.get_pc() as i32 + offset as i32) as u16);
        }
    }
}
