pub struct CPU {
    acc: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    flags: u8,
}

impl CPU {
    pub fn get_carry_flag(&self) -> u8 { self.flags & 0x01 }
    pub fn get_zero_flag(&self) -> u8 { (self.flags & 0x02) >> 1 }
    pub fn get_interrupt_flag(&self) -> u8 { (self.flags & 0x04) >> 2 }
    pub fn get_decimal_flag(&self) -> u8 { (self.flags & 0x08) >> 3 }
    pub fn get_b_flag(&self) -> u8 { (self.flags & 0x10) >> 4 }
    pub fn get_blank_flag(&self) -> u8 { (self.flags & 0x20) >> 5 }
    pub fn get_overflow_flag(&self) -> u8 { (self.flags & 0x40) >> 6 }
    pub fn get_negative_flag(&self) -> u8 { (self.flags & 0x80) >> 7 }

    pub fn set_carry_flag(&mut self, val: u8) { self.flags |= val }
    pub fn set_zero_flag(&mut self, val: u8) { self.flags |= val << 1 }
    pub fn set_interrupt_flag(&mut self, val: u8) { self.flags |= val << 2 }
    pub fn set_decimal_flag(&mut self, val: u8) { self.flags |= val << 3 }
    pub fn set_b_flag(&mut self, val: u8) { self.flags |= val << 4 }
    pub fn set_blank_flag(&mut self, val: u8) { self.flags |= val << 5 }
    pub fn set_overflow_flag(&mut self, val: u8) { self.flags |= val << 6 }
    pub fn set_negative_flag(&mut self, val: u8) { self.flags |= val << 7 }

    pub fn get_acc(&self) -> u8 { self.acc }
    pub fn get_x_reg(&self) -> u8 { self.x }
    pub fn get_y_reg(&self) -> u8 { self.y }
    pub fn get_sp(&self) -> u8 { self.sp }
    pub fn get_pc(&self) -> u16 { self.pc }

    pub fn set_acc(&self, val: u8) { self.acc = val }
    pub fn set_x_reg(&self, val: u8) { self.x = val }
    pub fn set_y_reg(&self, val: u8) { self.y = val }
    pub fn set_sp(&self, val: u8) { self.sp = val }
    pub fn set_pc(&self, val: u16) { self.pc = val }
}
