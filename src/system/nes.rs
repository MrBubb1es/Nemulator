use std::{fs, io::Read, rc::Rc};

use crate::cartridge::cartridge::Cartridge;

use super::{bus::Bus, cpu::CPU, ppu::PPU};

pub struct NES {
    cart: Rc<Cartridge>,
    bus: Rc<Bus>,
    pub cpu: CPU,
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

    pub fn get_clks(&self) -> usize {
        self.cpu.clocks()
    }

    pub fn cycle(&mut self) {
        self.cpu.cycle();
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

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
        // Big vector of the expected CPU states from start to finish of the nestest program.
        // Stored as tuples of (PC, SP, ACC, X, Y, STATUS, CLOCKS)
        let mut expected_vals = Vec::new();
        
        for line in read_to_string("prg_tests/cpu_tests/expected_log.txt").unwrap().lines() {
            expected_vals.push(
                {
                    let temp: Vec<usize> = line.split(",").map(|num| {
                        if num.contains("0x") {
                            usize::from_str_radix(&num[2..], 16).unwrap()
                        } else {
                            usize::from_str_radix(num, 10).unwrap()
                        }
                    }).collect();

                    (temp[0] as u16, // pc
                    temp[1] as u8,   // sp
                    temp[2] as u8,   // acc
                    temp[3] as u8,   // x
                    temp[4] as u8,   // y
                    temp[5] as u8,   // status
                    temp[6])         // clocks
                }
            )
        }


        let mut test_nemulator = NES::new("prg_tests/nestest.nes");
        test_nemulator.cpu.set_pc(0xC000); // run tests automatically

        for i in 0..expected_vals.len() {
            // test_nemulator.get_cpu().print_state();

            let (exp_pc, exp_sp, exp_acc, exp_x, exp_y, exp_flags, exp_clks) = expected_vals[i];

            assert_eq!(exp_pc, test_nemulator.get_cpu().get_pc());
            assert_eq!(exp_sp, test_nemulator.get_cpu().get_sp());
            assert_eq!(exp_acc, test_nemulator.get_cpu().get_acc());
            assert_eq!(exp_x, test_nemulator.get_cpu().get_x_reg());
            assert_eq!(exp_y, test_nemulator.get_cpu().get_y_reg());
            assert_eq!(exp_flags, test_nemulator.get_cpu().get_flags());
            assert_eq!(exp_clks, test_nemulator.get_cpu().clocks());

            test_nemulator.cycle();
        }

        assert_eq!(test_nemulator.bus.read(0x0002), 0);
        assert_eq!(test_nemulator.bus.read(0x0003), 0);
    }
}
