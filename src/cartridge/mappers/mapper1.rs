use crate::cartridge::{mapper::NametableMirror, Cartridge, Mapper};

#[derive(Default)]
pub struct Mapper1 {
    control: u8,
    chr_bank0: u8,
    chr_bank1: u8,
    prg_bank: u8,

    write_count: usize,
    shift_reg: u8,

    prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    chr_rom: Vec<u8>,
    chr_ram: Vec<u8>,
}

impl Mapper for Mapper1 {
    fn init(&mut self, cart: Cartridge) {
        self.prg_rom = cart.get_prg_rom();
        self.chr_rom = cart.get_chr_rom();
    }

    fn cpu_cart_read(&mut self, addr: u16) -> Option<u8> {
        todo!()
    }

    fn ppu_cart_read(&mut self, addr: u16) -> Option<u8> {
        todo!()
    }

    fn cpu_cart_write(&mut self, addr: u16, data: u8) -> bool {
        match addr {
            0x6000..=0x7FFF => {

            }
            0x8000..=0xFFFF => {
                if data & 0x80 == 0 {
                    self.cpu_write_regs(addr, data);
                } else {
                    self.shift_reg = 0;
                    self.write_count = 0;
                }
            }
            _ => {}
        };
        
        false
    }

    fn ppu_cart_write(&mut self, addr: u16, data: u8) -> bool {
        todo!()
    }

    fn get_nt_mirror_type(&self) -> NametableMirror {
        match self.control & 3 {
            0 => NametableMirror::SingleScreenLower,
            1 => NametableMirror::SingleScreenUpper,
            2 => NametableMirror::Vertical,
            3 => NametableMirror::Horizontal,
            _ => {unreachable!("Things are wrong :O")},
        }
    }
}


impl Mapper1 {
    fn cpu_write_regs(&mut self, address: u16, data: u8) {
        self.shift_reg <<= 1;
        self.shift_reg |= data & 1;
        self.write_count += 1;

        if self.write_count == 5 {
            let mapper_reg = match (address >> 12) & 3 {
                0 => &mut self.control,
                1 => &mut self.chr_bank0,
                2 => &mut self.chr_bank1,
                3 => &mut self.prg_bank,
                _ => {unreachable!("Bruh whatchu doin?")}
            };

            *mapper_reg = self.shift_reg & 0x1F;
        
            self.shift_reg = 0;
        }
    }
}