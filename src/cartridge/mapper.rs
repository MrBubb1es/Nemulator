use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::{default, rc::Rc};

use super::mappers::{Mapper0, Mapper1, Mapper3};
use super::{cartridge::Header, Cartridge};

#[derive(Clone, Copy, Debug, Default)]
pub enum NametableMirror {
    #[default]
    Vertical,
    Horizontal,
    SingleScreenLower,
    SingleScreenUpper,
}

/// Defines an interface for all memory mappers to implement that allows for
/// translation of the 16-bit address put on the bus to a location in
/// cartridge memory. We pass the data to be written on the _write calls to
/// allow for memory banking (e.g., to write the data into a register in
/// memory that controls which memory bank is active).
pub trait Mapper {
    /// Initializes any mapper information based on the cartridge file header
    fn init(&mut self, cart: Cartridge);
    /// Translates the given address from the CPU addressing space to the PRG
    /// ROM as decided by the mapper.
    ///
    ///  * `addr` - The CPU address to translate to a cartridge PRG ROM address
    fn cpu_cart_read(&mut self, addr: u16) -> Option<u8>;
    /// Translates the given address from the PPU addressing space to the CHR
    /// ROM as decided by the mapper.
    ///
    ///  * `addr` - The PPU address to translate to a cartridge PRG ROM address
    fn ppu_cart_read(&mut self, addr: u16) -> Option<u8>;
    /// Translates the given address from the CPU addressing space to the PRG
    /// ROM as decided by the mapper. Also takes the data in case the address
    /// being written to directly affects the mapper's internal workings.
    ///
    ///  * `addr` - The PPU address to translate to a cartridge PRG ROM address
    ///  * `data` - The data being written to PRG ROM (may be used to set internal mapper register)
    fn cpu_cart_write(&mut self, addr: u16, data: u8) -> bool;
    /// Translates the given address from the PPU addressing space to the CHR
    /// ROM as decided by the mapper. Also takes the data in case the address
    /// being written to directly affects the mapper's internal workings.
    ///
    ///  * `addr` - The PPU address to translate to a cartridge PRG ROM address
    ///  * `data` - The data being written to PRG ROM (may be used to set internal mapper register)
    fn ppu_cart_write(&mut self, addr: u16, data: u8) -> bool;
    /// Returns the direction addresses should be mirrored.
    fn get_nt_mirror_type(&self) -> NametableMirror;
}

pub fn mapper_from_cart(cart: Cartridge) -> Rc<RefCell<dyn Mapper>> {
    println!("Loading cart with mapper {}", cart.header.mapper_num);

    let mapper: Rc<RefCell<dyn Mapper>> = match cart.header.mapper_num {
        0 => Rc::new(RefCell::new(Mapper0::default())),
        // 1 => Box::new(Mapper1::default()),
        3 => Rc::new(RefCell::new(Mapper3::default())),
        _ => panic!("unimplemented mapper number {}", cart.header.mapper_num),
    };

    mapper.as_ref().borrow_mut().init(cart);

    mapper
}
