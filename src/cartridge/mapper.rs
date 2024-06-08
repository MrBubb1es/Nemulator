use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::{default, rc::Rc};

use super::mappers::{Mapper0, Mapper1, Mapper3};
use super::{cartridge::Header, Cartridge};

#[derive(Clone, Copy, Debug, Default)]
pub enum NametableMirror {
    #[default]
    Vertical,
    Horizontal,
}

/// Defines an interface for all memory mappers to implement that allows for
/// translation of the 16-bit address put on the bus to a location in
/// cartridge memory. We pass the data to be written on the _write calls to
/// allow for memory banking (e.g., to write the data into a register in
/// memory that controls which memory bank is active).
pub trait Mapper: Send + Sync {
    /// Initializes any mapper information based on the cartridge file header
    fn init(&mut self, header: &Header);
    /// Translates the given address from the CPU addressing space to the PRG
    /// ROM as decided by the mapper.
    ///
    ///  * `addr` - The CPU address to translate to a cartridge PRG ROM address
    fn get_cpu_read_addr(&mut self, addr: u16) -> Option<u16>;
    /// Translates the given address from the PPU addressing space to the CHR
    /// ROM as decided by the mapper.
    ///
    ///  * `addr` - The PPU address to translate to a cartridge PRG ROM address
    fn get_ppu_read_addr(&mut self, addr: u16) -> Option<u16>;
    /// Translates the given address from the CPU addressing space to the PRG
    /// ROM as decided by the mapper. Also takes the data in case the address
    /// being written to directly affects the mapper's internal workings.
    ///
    ///  * `addr` - The PPU address to translate to a cartridge PRG ROM address
    ///  * `data` - The data being written to PRG ROM (may be used to set internal mapper register)
    fn get_cpu_write_addr(&mut self, addr: u16, data: u8) -> Option<u16>;
    /// Translates the given address from the PPU addressing space to the CHR
    /// ROM as decided by the mapper. Also takes the data in case the address
    /// being written to directly affects the mapper's internal workings.
    ///
    ///  * `addr` - The PPU address to translate to a cartridge PRG ROM address
    ///  * `data` - The data being written to PRG ROM (may be used to set internal mapper register)
    fn get_ppu_write_addr(&mut self, addr: u16, data: u8) -> Option<u16>;
    /// Returns the direction addresses should be mirrored.
    fn get_nt_mirror_type(&self) -> NametableMirror;
}

pub fn get_mapper(header: &Header) -> Box<dyn Mapper> {
    let mut mapper: Box<dyn Mapper> = match header.mapper_num {
        0 => Box::new(Mapper0::default()),
        3 => Box::new(Mapper3::default()),
        _ => Box::new(Mapper0::default()),
    };

    mapper.init(header);

    mapper
}