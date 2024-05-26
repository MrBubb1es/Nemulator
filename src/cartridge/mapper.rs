/// Defines an interface for all memory mappers to implement that allows for
/// translation of the 16-bit address put on the bus to a location in
/// cartridge memory. We pass the data to be written on the _write calls to
/// allow for memory banking (e.g., to write the data into a register in
/// memory that controls which memory bank is active).
pub trait Mapper {
    /// Translates the given address from the CPU addressing space to the PRG 
    /// ROM as decided by the mapper.
    /// 
    ///  * `addr` - The CPU address to translate to a cartridge PRG ROM address
    fn get_cpu_read_addr(&self, addr: u16) -> Option<u16>;
    /// Translates the given address from the PPU addressing space to the CHR 
    /// ROM as decided by the mapper.
    /// 
    ///  * `addr` - The PPU address to translate to a cartridge PRG ROM address
    fn get_ppu_read_addr(&self, addr: u16) -> Option<u16>;
    /// Translates the given address from the CPU addressing space to the PRG 
    /// ROM as decided by the mapper. Also takes the data in case the address
    /// being written to directly affects the mapper's internal workings.
    /// 
    ///  * `addr` - The PPU address to translate to a cartridge PRG ROM address
    ///  * `data` - The data being written to PRG ROM (may be used to set internal mapper register)
    fn get_cpu_write_addr(&self, addr: u16, data: u8) -> Option<u16>;
    /// Translates the given address from the PPU addressing space to the CHR 
    /// ROM as decided by the mapper. Also takes the data in case the address
    /// being written to directly affects the mapper's internal workings.
    /// 
    ///  * `addr` - The PPU address to translate to a cartridge PRG ROM address
    ///  * `data` - The data being written to PRG ROM (may be used to set internal mapper register)
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
            0xC000..=0xFFFF => Some(addr - 0xC000),
            _ => None,
        }
    }

    fn get_ppu_read_addr(&self, addr: u16) -> Option<u16> {
        match addr {
            0x0000..=0x2000 => Some(addr),
            _ => None,
        }
    }

    fn get_cpu_write_addr(&self, addr: u16, _data: u8) -> Option<u16> {
        match addr {
            0x8000..=0xBFFF => Some(addr - 0x8000),
            0xC000..=0xFFFF => Some(addr - 0xC000),
            _ => None,
        }
    }

    fn get_ppu_write_addr(&self, addr: u16, _data: u8) -> Option<u16> {
        match addr {
            0x0000..=0x2000 => Some(addr),
            _ => None,
        }
    }
}

pub fn get_mapper(_mapper_id: u16) -> impl Mapper {
    Mapper0
}
