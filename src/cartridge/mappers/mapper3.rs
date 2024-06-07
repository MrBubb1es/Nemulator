use std::cell::Cell;

use crate::cartridge::{mapper::NametableMirror, Cartridge, Header, Mapper};

#[derive(Default)]
pub struct Mapper3 {
    bank_select: u8,

    nt_mirror_type: NametableMirror,
    num_prg_banks: usize,
}

impl Mapper for Mapper3 {
    fn init(&mut self, header: &Header) {
        self.nt_mirror_type = if header.hardwired_nametable {
            NametableMirror::Horizontal
        } else {
            NametableMirror::Vertical
        };

        self.num_prg_banks = Cartridge::rom_size(header.prg_rom_size);
    }

    fn get_cpu_read_addr(&mut self, addr: u16) -> Option<u16> {
        match addr {
            0x8000..=0xFFFF => Some(
                addr & (if self.num_prg_banks > 1 {
                    0x7FFF
                } else {
                    0x3FFF
                }),
            ),
            _ => None,
        }
    }

    fn get_ppu_read_addr(&mut self, addr: u16) -> Option<u16> {
        match addr {
            0x0000..=0x1FFF => Some( ((self.bank_select as u16) * 0x2000) + addr ),
            _ => None,
       }
    }

    fn get_cpu_write_addr(&mut self, addr: u16, data: u8) -> Option<u16> {
        match addr {
            0x8000..=0xFFFF => {
                self.bank_select = data;
                None
            },
            _ => None,
        }
    }

    fn get_ppu_write_addr(&mut self, addr: u16, _data: u8) -> Option<u16> {
        None
    }

    fn get_nt_mirror_type(&self) -> NametableMirror {
        self.nt_mirror_type
    }
}
