use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::{default, rc::Rc};

use crate::cartridge::mappers::Mapper2;

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

/*
ON HOW MAPPERS WORK:

Mappers, at their core, are simply a way to take in an address from CPU or PPU
addressing space, and translate it to a new address in cartridge addressing space.
Mappers 0 & 3 are the simplest mappers as they do very little to the given address. 
Mappers get first dibs on if they want to handle a read/write, but they have the
option to ignore it and allow the read/write to be handled somewhere else in the
system.

Mapper 2 is the next easiest mapper to understand. It has the ability to switch
which PRG ROM bank is active. The CPU's address space is only 16 bits wide, and
PRG ROM only gets the addresses $8000-$FFFF (32 KiB). Many games have more than
32 KiB of PRG ROM, however. The CPU needs to be able to access all of the data,
so the mapper has the ability to switch out which cartridge memory is actually
being accessed when the CPU reads from PRG ROM. Only 16 KiB of address space is
usually switched at a time so that the program that is currently running in PRG
ROM isn't switched out as it is executing (e.g. if the program counter = $8291
then the high program rom bank would be switched, resulting in the addresses from
$C000-$FFFF being swapped out. This way the program running at the program counter
isn't affected). Mapper 2 allows this kind of switching, which is done whenever
the cartridge detects that the CPU has made a write to PRG ROM. By definition,
PRG ROM cannot be written to, so instead the mapper intercepts this write and
uses it to change internal registers that decide where reads from PRG ROM are routed.
This method of changing internal registers to switch PRG/CHR ROM banks through 
writes to $8000-$FFFF is common in many mappers.
*/

/// Defines an interface for all memory mappers to implement that allows for
/// translation of the 16-bit address put on the bus to a location in
/// cartridge memory. We pass the data to be written on the _write calls to
/// allow for memory banking (e.g., to write the data into a register in
/// memory that controls which memory bank is active).
pub trait Mapper {
    /// Initializes any mapper/cartridge information based on the cartridge file header
    fn init(&mut self, cart: Cartridge);
    /// If the mapper maps the address given by the CPU to somewhere in PRG ROM
    /// or RAM, then the memory internal to the cartridge is accessed and the
    /// data is returned. If the mapper doesn't want the address, i.e. the read
    /// is meant for somewhere else in the system, then none is returned instead. 
    ///
    ///  * `addr` - The CPU address to translate to a cartridge PRG ROM address
    fn cpu_cart_read(&mut self, addr: u16) -> Option<u8>;
    /// If the mapper maps the address given by the PPU to somewhere in CHR ROM
    /// or RAM, then the memory internal to the cartridge is accessed and the
    /// data is returned. If the mapper doesn't want the address, i.e. the read
    /// is meant for somewhere else in the system, then none is returned instead. 
    ///
    ///  * `addr` - The PPU address to translate to a cartridge CHR ROM address
    fn ppu_cart_read(&mut self, addr: u16) -> Option<u8>;
    /// If the mapper maps the address given by the CPU to somewhere in PRG RAM
    /// then the internal cartridge memory is written to. If the address is mapped
    /// to PRG ROM, then the mapper may use the data to modify its internal registers.
    /// In both of these cases, true is returned to indicate that the mapper has
    /// handled the write, otherwise false is returned and the write can go
    /// somewhere else.
    ///
    ///  * `addr` - The PPU address to translate to a cartridge PRG ROM address
    ///  * `data` - The data being written to PRG ROM/RAM (may be used to set internal mapper register)
    fn cpu_cart_write(&mut self, addr: u16, data: u8) -> bool;
    /// If the mapper maps the address given by the PPU to somewhere in CHR RAM
    /// then the internal cartridge memory is written to. In this case, true is 
    /// returned to indicate that the mapper has handled the write, otherwise 
    /// false is returned and the write can go somewhere else.
    ///
    ///  * `addr` - The PPU address to translate to a cartridge CHR ROM/RAM address
    ///  * `data` - The data being written to CHR ROM/RAM
    fn ppu_cart_write(&mut self, addr: u16, data: u8) -> bool;
    /// Returns the direction addresses should be mirrored.
    fn get_nt_mirror_type(&self) -> NametableMirror;
    /// Resets the mapper to a known state. This happens whenever the NES system is reset.
    fn reset(&mut self) {}
}

pub fn mapper_from_cart(cart: Cartridge) -> Rc<RefCell<dyn Mapper>> {
    println!("Loading cart with mapper {}", cart.header.mapper_num);

    let mapper: Rc<RefCell<dyn Mapper>> = match cart.header.mapper_num {
        0 => Rc::new(RefCell::new(Mapper0::default())),
        1 => Rc::new(RefCell::new(Mapper1::default())),
        2 => Rc::new(RefCell::new(Mapper2::default())),
        3 => Rc::new(RefCell::new(Mapper3::default())),
        _ => panic!("unimplemented mapper number {}", cart.header.mapper_num),
    };

    mapper.as_ref().borrow_mut().init(cart);

    mapper
}
