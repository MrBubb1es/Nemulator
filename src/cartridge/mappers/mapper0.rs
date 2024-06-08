use crate::cartridge::{Cartridge, Header};
use crate::cartridge::mapper::{Mapper, NametableMirror};

/// The simplest mapper, and the most common.
/// PRG: 0x8000-BFFF (mirrored 0xC000-FFFF)
/// CHR: 0x0000-2000
#[derive(Debug, Default)]
pub struct Mapper0 {
    nt_mirror_type: NametableMirror,
    num_prg_banks: usize,
}

impl Mapper for Mapper0 {
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

    fn get_ppu_read_addr(&mut self, _addr: u16) -> Option<u16> {
        // Mapper zero doesn't touch ppu addresses
        None
    }

    fn get_cpu_write_addr(&mut self, _addr: u16, _data: u8) -> Option<u16> {
        // match addr {
        //     0x8000..=0xFFFF => Some(
        //         addr & (if self.num_prg_banks > 1 {
        //             0x7FFF
        //         } else {
        //             0x3FFF
        //         }),
        //     ),
        //     _ => None,
        // }
        None
    }

    fn get_ppu_write_addr(&mut self, _addr: u16, _data: u8) -> Option<u16> {
        None
    }

    fn get_nt_mirror_type(&self) -> NametableMirror {
        self.nt_mirror_type
    }
}

unsafe impl Send for Mapper0 {}
unsafe impl Sync for Mapper0 {}