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
    fn get_carry_flag(&self) -> u8 { self.flags & 0x01 }
    fn get_zero_flag(&self) -> u8 { (self.flags & 0x02) >> 1 }
    fn get_interrupt_flag(&self) -> u8 { (self.flags & 0x04) >> 2 }
    fn get_decimal_flag(&self) -> u8 { (self.flags & 0x08) >> 3 }
    fn get_b_flag(&self) -> u8 { (self.flags & 0x10) >> 4 }
    fn get_blank_flag(&self) -> u8 { (self.flags & 0x20) >> 5 }
    fn get_overflow_flag(&self) -> u8 { (self.flags & 0x40) >> 6 }
    fn get_negative_flag(&self) -> u8 { (self.flags & 0x80) >> 7 }

    fn set_carry_flag(&mut self, val: u8) { self.flags |= val }
    fn set_zero_flag(&mut self, val: u8) { self.flags |= val << 1 }
    fn set_interrupt_flag(&mut self, val: u8) { self.flags |= val << 2 }
    fn set_decimal_flag(&mut self, val: u8) { self.flags |= val << 3 }
    fn set_b_flag(&mut self, val: u8) { self.flags |= val << 4 }
    fn set_blank_flag(&mut self, val: u8) { self.flags |= val << 5 }
    fn set_overflow_flag(&mut self, val: u8) { self.flags |= val << 6 }
    fn set_negative_flag(&mut self, val: u8) { self.flags |= val << 7 }

    fn get_acc(&self) -> u8 { self.acc }
    fn get_x_reg(&self) -> u8 { self.x }
    fn get_y_reg(&self) -> u8 { self.y }
    fn get_sp(&self) -> u8 { self.sp }
    fn get_pc(&self) -> u16 { self.pc }

    fn set_acc(&self, val: u8) { self.acc = val }
    fn set_x_reg(&self, val: u8) { self.x = val }
    fn set_y_reg(&self, val: u8) { self.y = val }
    fn set_sp(&self, val: u8) { self.sp = val }
    fn set_pc(&self, val: u16) { self.pc = val }


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


    // RESET FUNCTION


    // ADDRESSING MODES - Fetches data from 
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
    fn adc(&mut self, arg: u8) {
        let result = (arg as u16) + (self.acc as u16) + (self.get_carry_flag() as u16);
        self.set_acc(result as u8);
        self.set_carry_flag(if result & 0xFF00 > 0 { 1 } else { 0 });
        self.set_zero_flag(if result & 0xFF == 0 { 1 } else { 0 });
        // NOTE: come back and do the overflow (V) flag once you know how it works
        // NOTE: and the negative flag bc I can't be bothered to rn
    }

    fn and(&mut self, arg: u8) {
        let result = self.acc & arg;
        self.set_zero_flag(if result == 0 { 1 } else { 0 });
        self.set_negative_flag(if result & 0x80 > 0 { 1 } else { 0 });
    }

    fn asl() {
        // skipping for now
    }

    fn bcc(&mut self, offset: i8) {
        if self.get_carry_flag() != 1 {
            self.pc = (self.pc as i32 + offset as i32) as u16;
        }
    }

    fn bcs(&mut self, offset: i8) {
        if self.get_carry_flag() == 1 {
            self.pc = (self.pc as i32 + offset as i32) as u16;
        }
    }

    fn beq(&mut self, offset: i8) {
        if self.get_zero_flag() == 1 {
            self.pc = (self.pc as i32 + offset as i32) as u16;
        }
    }

    fn bit(&mut self, arg: u8) {
        self.set_negative_flag(if arg & 0x80 > 0 { 1 } else { 0 });
        self.set_overflow_flag(if arg & 0x40 > 0 { 1 } else { 0 });
        self.set_zero_flag(if arg & self.acc == 0 { 1 } else { 0 });
    }

    fn bmi(&mut self, offset: i8) {
        if self.get_negative_flag() == 1 {
            self.pc = (self.pc as i32 + offset as i32) as u16;
        }
    }
}


#[cfg(test)]
mod tests {

}
