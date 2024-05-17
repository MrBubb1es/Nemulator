use crate::cartridge::cartridge;
use crate::system::mem::Memory;

pub struct Bus {
    memory: Memory,
    cartridge: &'static cartridge::Cartridge,
}

impl Bus {
    pub fn new(&'static cart: cartridge::Cartridge) -> Self {
        Bus {
            memory: Memory::new(0x800),
            cartridge: &cart,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x2000 => {
                // First 2KiB of memory (0x0800) are mirrored through 0x2000
                self.memory.read(address & 0x08FF)
            }
            _ => self.cartridge.write(address, data),
        }
    }

    pub fn write(&self, address: u16, data: u8) {
        match address {
            0x0000..=0x2000 => {
                // First 2KiB of memory (0x0800) are mirrored through 0x2000
                self.memory.write(address & 0x08FF, data)
            }
            _ => self.cartridge.write(address, data),
        }
    }
}
