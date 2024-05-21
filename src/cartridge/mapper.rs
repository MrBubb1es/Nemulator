/// Defines an interface for all memory mappers to implement that allows for
/// translation of the 16-bit address put on the bus to a location in
/// cartridge memory. We pass the data to be written on the _write calls to
/// allow for memory banking (e.g., to write the data into a register in
/// memory that controls which memory bank is active).
pub trait Mapper {
    fn get_cpu_read_addr(&self, addr: u16) -> Option<u16>;
    fn get_ppu_read_addr(&self, addr: u16) -> Option<u16>;
    fn get_cpu_write_addr(&self, addr: u16, data: u8) -> Option<u16>;
    fn get_ppu_write_addr(&self, addr: u16, data: u8) -> Option<u16>;
}

/// The simplest mapper, and the most common.
/// PRG: 0x8000-BFFF (mirrored 0xC000-FFFF)
/// CHR: 0x0000-2000
struct Mapper0;

impl Mapper for Mapper0 {
    fn get_cpu_read_addr(&self, addr: u16) -> Option<u16> {
        match addr {
            0x8000..=0xBFFF => Some(addr - 0x8000),
            0xC000..=0xFFFF => Some(addr - 0x8000),
            _ => None,
        }
    }

    fn get_ppu_read_addr(&self, addr: u16) -> Option<u16> {
        match addr {
            0x0000..=0x2000 => Some(addr - 0x8000),
            _ => None,
        }
    }

    fn get_cpu_write_addr(&self, addr: u16, _data: u8) -> Option<u16> {
        match addr {
            0x8000..=0xBFFF => Some(addr - 0x8000),
            0xC000..=0xFFFF => Some(addr - 0x8000),
            _ => None,
        }
    }

    fn get_ppu_write_addr(&self, addr: u16, _data: u8) -> Option<u16> {
        match addr {
            0x0000..=0x2000 => Some(addr - 0x8000),
            _ => None,
        }
    }
}

pub fn get_mapper(mapper: u16) -> impl Mapper {
    Mapper0
}
