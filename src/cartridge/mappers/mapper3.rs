use crate::cartridge::{mapper::NametableMirror, Cartridge, Header, Mapper};

#[derive(Default)]
pub struct Mapper3 {
    chr_bank_select: usize,

    nt_mirror_type: NametableMirror,
    num_prg_banks: usize,

    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
}

impl Mapper for Mapper3 {
    fn init(&mut self, cart: Cartridge) {
        self.nt_mirror_type = if cart.header.hardwired_nametable {
            NametableMirror::Horizontal
        } else {
            NametableMirror::Vertical
        };

        self.num_prg_banks = cart.prg_rom_banks();
        self.prg_rom = cart.get_prg_rom();
        self.chr_rom = cart.get_chr_rom();
    }

    fn cpu_cart_read(&mut self, addr: u16) -> Option<u8> {
        if 0x8000 <= addr {
            let mut mapped_addr = addr;

            if self.num_prg_banks == 2 {
                mapped_addr &= 0x7FFF;
            } else if self.num_prg_banks == 1 {
                mapped_addr &= 0x3FFF;
            } else {
                panic!("Mapper 0 should have 1 or 2 prg rom banks");
            };

            return Some( self.prg_rom[mapped_addr as usize] );
        }

        None
    }

    fn ppu_cart_read(&mut self, addr: u16) -> Option<u8> {
        if addr <= 0x1FFF {
            let mapped_addr = (self.chr_bank_select * 0x2000) + addr as usize;

            return Some( self.chr_rom[mapped_addr] );
        }

        None
    }

    fn cpu_cart_write(&mut self, addr: u16, data: u8) -> bool {
        if 0x8000 <= addr {
            self.chr_bank_select = (data & 3) as usize;
        }

        false
    }

    fn ppu_cart_write(&mut self, _addr: u16, _data: u8) -> bool {
        false
    }

    fn get_nt_mirror_type(&self) -> NametableMirror {
        self.nt_mirror_type
    }

    fn reset(&mut self) {
        self.chr_bank_select = 0;
    }
}
