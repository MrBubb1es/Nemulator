use crate::cartridge::cartridge;
use super::mem::Memory;
use super::ppu::PPU;


/// Main bus struct connecting CPU, main memory, the cartridge, the PPU
pub struct Bus {
    memory: Memory,
    cart: &'static cartridge::Cartridge,
    ppu: &'static PPU,
}

impl Bus {
    /// Create a new bus attatched to some cartridge
    pub fn new(cart: &'static cartridge::Cartridge, ppu: &'static PPU) -> Self {
        Bus {
            memory: Memory::new(0x800),
            cart,
            ppu: ppu,
        }
    }

    /// Put a read call through the bus and redirect it to whichever device the
    /// address corresponds to.
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => {
                // First 2KiB of memory (0x0800) are mirrored thru 0x2000
                self.memory.read(address & 0x08FF)
            },
            0x2000..=0x3FFF => {
                // Next 2KiB are the 8 PPU registers mirrored over and over
                self.ppu.read(address & 0x0008)
            }
            _ => self.cart.cpu_read(address),
        }
    }

    /// Make a write call on the bus and redirect it to whichever device the
    /// given address corresponds to.
    pub fn write(&self, address: u16, data: u8) {
        match address {
            0x0000..=0x2000 => {
                // First 2KiB of memory (0x0800) are mirrored through 0x2000
                self.memory.write(address & 0x08FF, data)
            }
            0x2001..=0x3FFF => {
                // Next 2KiB are the 8 PPU registers mirrored over and over
                self.ppu.write(address & 0x0008, data)
            }
            _ => self.cart.cpu_write(address, data),
        }
    }
}
