use crate::cartridge::{mapper::NametableMirror, Cartridge, Header, Mapper};

#[derive(Default)]
pub struct Mapper3 {
    bank_select: u8,
    register: [u8; 8],

    nt_mirror_type: NametableMirror,
}

impl Mapper for Mapper3 {
    fn init(&mut self, cart: Cartridge) {
        self.bank_select = 0;
        self.register = [0; 8];

        if cart.alt_nametables {
            self.nt_mirror_type = if cart.nam {

            }
        }
        self.nt_mirror_type = NametableMirror::Vertical;
    }

    fn cpu_cart_read(&mut self, addr: u16) -> Option<u8> {
        
        None
    }

    fn ppu_cart_read(&mut self, addr: u16) -> Option<u8> {
        None
    }

    fn cpu_cart_write(&mut self, addr: u16, data: u8) -> bool {
        if 0x8000 <= addr {
            match addr {
                0x8000..=0x9FFF if addr & 0x01 == 0 => {
                    self.bank_select = data;
                },
                0x8000..=0x9FFF if addr & 0x01 == 1 => {
                    let reg_idx = (self.bank_select & 0x03) as usize;
                    if reg_idx <= 1 {
                        // Registers 0 and 1 have the low bit masked out
                        self.register[reg_idx] = data & 0xFE;
                    } else {
                        self.register[reg_idx] = data;
                    }
                },
                0xA000..=0xBFFF if addr & 0x01 == 0 => {
                    
                },
                0xA000..=0xBFFF if addr & 0x01 == 1 => {
                    self.bank_select = data;
                },
                
            }
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

    }
}
