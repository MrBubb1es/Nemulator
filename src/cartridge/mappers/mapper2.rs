use crate::cartridge::{Cartridge, Header};
use crate::cartridge::mapper::{Mapper, NametableMirror};

/// In this mapper, the low half of the PRG ROM address space ($8000-$BFFF) is
/// switchable, while the higher half ($C000-$FFFF) is locked to the highest bank
/// of PRG ROM that the cartridge has.
///
/// Additionally, if the ROM indicates that there are 0 CHR ROM banks, then the
/// CHR ROM is actually treated as CHR RAM.
#[derive(Debug, Default)]
pub struct Mapper2 {
    nt_mirror_type: NametableMirror,

    prg_rom: Vec<u8>,
    chr_mem: Vec<u8>, // chr_mem b/c this may be treated as ROM or RAM

    num_prg_banks: usize,
    num_chr_banks: usize,

    prg_bank_select_lo: usize,
    prg_bank_select_hi: usize,
}

impl Mapper for Mapper2 {
    fn init(&mut self, cart: Cartridge) {
        self.nt_mirror_type = if cart.header.hardwired_nametable {
            NametableMirror::Vertical
        } else {
            NametableMirror::Horizontal
        };

        self.num_prg_banks = cart.prg_rom_banks();
        self.num_chr_banks = cart.chr_rom_banks();

        self.prg_rom = cart.get_prg_rom();
        self.chr_mem = cart.get_chr_rom();

        self.reset();
    }

    fn cpu_cart_read(&mut self, addr: u16) -> Option<u8> {
        match addr {
            // PRG bank low
            0x8000..=0xBFFF => {
                let mapped_addr = self.prg_bank_select_lo * 0x4000 + (addr & 0x3FFF) as usize;

                Some( self.prg_rom[mapped_addr] )
            }

            // PRG bank high
            0xC000..=0xFFFF => {
                let mapped_addr = self.prg_bank_select_hi * 0x4000 + (addr & 0x3FFF) as usize;

                Some( self.prg_rom[mapped_addr] )
            }

            _ => None,
        }
    }

    fn ppu_cart_read(&mut self, addr: u16) -> Option<u8> {
        if addr <= 0x1FFF {
            return Some( self.chr_mem[addr as usize] );
        }

        None
    }

    fn cpu_cart_write(&mut self, addr: u16, data: u8) -> bool {
        if 0x8000 <= addr {
            self.prg_bank_select_lo = (data & 0x0F) as usize;
        }
        
        false
    }

    fn ppu_cart_write(&mut self, addr: u16, data: u8) -> bool {
        // If # CHR banks == 0, treat CHR ROM as CHR RAM
        if self.num_chr_banks == 0 && addr <= 0x1FFF {
            self.chr_mem[addr as usize] = data;
            
            return true;
        }
        
        false
    }

    fn get_nt_mirror_type(&self) -> NametableMirror {
        self.nt_mirror_type
    }

    fn reset(&mut self) {
        self.prg_bank_select_lo = 0;
        self.prg_bank_select_hi = self.num_prg_banks - 1;
    }
}
