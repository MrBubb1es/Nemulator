use crate::system::{cpu::CPU, bus::Bus, ppu::PPU};
use crate::cartridge::cartridge::Cartridge;

/// The object that contains all the parts and functionality of the NES
/// hardware.
struct NES {
    cpu: CPU<'static>,
    ppu: PPU,
    bus: Bus,
    cart: Cartridge,
}

impl NES {
    pub fn load_cart_from_file(path: &str) -> Self {
        let path = std::path::Path::new(path);
        let cart_data = std::fs::read(path).expect("Couldn't read cart from file");
        
        let cart = Cartridge::from_bytes(&cart_data[..]).expect("Couldn't parse cartridge file");

        let bus = Bus::new(&cart);
        let cpu = CPU::new(&bus);
        let ppu = PPU::new();

        NES {
            cpu,
            ppu,
            bus,
            cart,
        }
    }
    
    pub fn cycle(&mut self) {
        self.cpu.cycle();
    }
}

