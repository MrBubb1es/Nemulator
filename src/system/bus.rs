use super::mem::Memory;
use crate::cartridge::cartridge;

/// Main bus struct connecting CPU, main memory, the cartridge, the PPU
pub struct Bus {
    memory: Memory,
    cartridge: &'static cartridge::Cartridge,
}

impl Bus {
    /// Create a new bus attatched to some cartridge
    pub fn new(cart: &'static cartridge::Cartridge) -> Self {
        Bus {
            memory: Memory::new(0x800),
            cartridge: cart,
        }
    }

    /// Put a read call through the bus and redirect it to whichever device the
    /// address corresponds to.
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => {
                // First 2KiB of memory (0x0800) are mirrored thru 0x2000
                self.memory.read(address & 0x08FF)
            }
            _ => self.cartridge.read(address),
        }
    }

    /// Make a write call on the bus and redirect it to whichever device the
    /// given address corresponds to.
    pub fn write(&self, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => {
                // First 2KiB of memory (0x0800) are mirrored through 0x2000
                self.memory.write(address & 0x08FF, data)
            }
            _ => self.cartridge.write(address, data),
        }
    }
}
