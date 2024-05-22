use std::{fs, io::Read, rc::Rc};

use crate::cartridge::cartridge::Cartridge;

use super::{bus::Bus, cpu::CPU, ppu::PPU};

pub struct NES {
    cart: Rc<Cartridge>,
    bus: Rc<Bus>,
    cpu: CPU,
    ppu: PPU,
}

impl NES {
    pub fn new(cart_path_str: &str) -> Self {
        let mut cart_file = match fs::File::open(cart_path_str) {
            Ok(v) => v,
            Err(..) => panic!("Could not find file '{cart_path_str}'"),
        };

        let mut data = Vec::new();

        if let Err(..) = cart_file.read_to_end(&mut data) {
            panic!("Failed to read cartridge from '{cart_path_str}' to buffer");
        }

        let cart = Rc::new(Cartridge::from_bytes(data.as_slice()).unwrap());
        let bus = Rc::new(Bus::new(Rc::clone(&cart)));
        let ppu = PPU::new(Rc::clone(&cart));
        let cpu = CPU::new(Rc::clone(&bus));

        NES {
            cart,
            bus,
            cpu,
            ppu,
        }
    }

    pub fn reset_cpu(&mut self) {
        self.cpu.reset();
    }

    pub fn get_cpu(&self) -> &CPU {
        &self.cpu
    }

    pub fn get_bus(&self) -> &Bus {
        &self.bus
    }

    pub fn cycle(&mut self) {
        self.cpu.cycle();
    }
}

#[cfg(test)]
mod tests {
    use crate::run_debug;

    use super::NES;

    #[test]
    fn test_nes_build() {
        let mut test_nemulator = NES::new("prg_tests/cpu_tests/my_file.bin");

        let (prg_size, chr_size) = test_nemulator.cart.get_rom_sizes();

        assert_eq!(prg_size, 0xE15);
        assert_eq!(chr_size, 0xA54);

        test_nemulator.cpu.reset();

        // run_debug(&mut test_nemulator);
    }

    #[test]
    fn test_load_cart() {
        let mut test_nemulator = NES::new("prg_tests/1.Branch_Basics.nes");

        test_nemulator.reset_cpu();

        //run_debug(&mut test_nemulator);
    }

    #[test]
    fn run_nes_test() {
        let mut test_nemulator = NES::new("prg_tests/nestest.nes");
        test_nemulator.reset_cpu();
        test_nemulator.cpu.set_pc(0xC000); // run tests automatically

        for _ in 0..100000 {
            // I don't know how else to drive the cpu rn
            test_nemulator.cycle();
        }

        // run_debug(&mut test_nemulator);

        assert_eq!(test_nemulator.bus.read(0x0002), 0);
        assert_eq!(test_nemulator.bus.read(0x0003), 0);
    }
}
