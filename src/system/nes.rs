use std::{borrow::Borrow, fs, io::Read, rc::Rc};

use crate::cartridge::cartridge::Cartridge;

use super::{bus::Bus, cpu::CPU, ppu::PPU};

#[derive(Default)]
pub struct NES {
    cart: Option<Rc<Cartridge>>,
    bus: Option<Rc<Bus>>,
    cpu: Option<CPU>,
    ppu: Option<PPU>,

    cart_loaded: bool,
}

impl NES {
    /// Create a new NES object with a cart pre-loaded
    pub fn with_cart(cart_path_str: &str) -> Self {
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
            cart: Some(cart),
            bus: Some(bus),
            cpu: Some(cpu),
            ppu: Some(ppu),

            cart_loaded: true,
        }
    }

    /// Load a new cart into this NES object
    pub fn load_cart(&mut self, cart_path_str: &str) {
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

        self.cart = Some(cart);
        self.bus = Some(bus);
        self.cpu = Some(cpu);
        self.ppu = Some(ppu);

        self.cart_loaded = true;
    }

    /// Remove the loaded cartridge from this NES
    pub fn remove_cart(&mut self) {
        self.cart = None;
        self.bus = None;
        self.cpu = None;
        self.ppu = None;

        self.cart_loaded = false;
    }

    /// Reset the CPU
    pub fn reset_cpu(&mut self) {
        if self.cart_loaded {
            self.cpu.as_mut().unwrap().reset();
        }
    }

    /// Manually set the state of the CPU
    pub fn set_cpu_state(&mut self, 
        pc: Option<u16>, 
        sp: Option<u8>, 
        acc: Option<u8>, 
        x: Option<u8>, 
        y: Option<u8>, 
        status: Option<u8>, 
        clocks: Option<usize>) {
        if self.cart_loaded {
            let cpu = self.cpu.as_mut().unwrap();

            cpu.set_pc(pc.unwrap_or(cpu.get_pc()));
            cpu.set_sp(sp.unwrap_or(cpu.get_sp()));
            cpu.set_acc(acc.unwrap_or(cpu.get_acc()));
            cpu.set_x_reg(x.unwrap_or(cpu.get_x_reg()));
            cpu.set_y_reg(y.unwrap_or(cpu.get_y_reg()));
            cpu.set_flags(status.unwrap_or(cpu.get_flags()));
            cpu.set_clocks(clocks.unwrap_or(cpu.clocks()));
        }
    }

    /// Get a reference to the CPU if a cart is loaded
    pub fn get_cpu(&self) -> Option<&CPU> {
        self.cpu.as_ref()
    }

    /// Get a reference to the Bus if a cart is loaded
    pub fn get_bus(&self) -> Option<&Bus> {
        self.bus.as_deref()
    }

    /// Get the number of CPU cLocks
    pub fn get_clks(&self) -> usize {
        if let Some(cpu) = self.cpu.borrow() {
            cpu.clocks()
        } else {
            0
        }
    }

    /// Cycle the CPU if cart is loaded
    pub fn cycle(&mut self) {
        if self.cart_loaded {
            self.cpu.as_mut().unwrap().cycle();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::NES;

    #[test]
    fn test_nes_build() {
        let mut test_nemulator = NES::with_cart("prg_tests/cpu_tests/my_file.bin");

        let (prg_size, chr_size) = test_nemulator.cart.unwrap().get_rom_sizes();

        assert_eq!(prg_size, 0xE15);
        assert_eq!(chr_size, 0xA54);

        test_nemulator.cpu.unwrap().reset();

        // run_debug(&mut test_nemulator);
    }

    #[test]
    fn test_load_cart() {
        let mut test_nemulator = NES::with_cart("prg_tests/1.Branch_Basics.nes");

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


        let mut test_nemulator = NES::with_cart("prg_tests/nestest.nes");
        test_nemulator.set_cpu_state(Some(0xC000), None, None, None, None, None, None); // run tests automatically

        for i in 0..expected_vals.len() {
            // test_nemulator.cpu.print_state();

            let (exp_pc, exp_sp, exp_acc, exp_x, exp_y, exp_flags, exp_clks) = expected_vals[i];

            assert_eq!(exp_pc, test_nemulator.get_cpu().unwrap().get_pc());
            assert_eq!(exp_sp, test_nemulator.get_cpu().unwrap().get_sp());
            assert_eq!(exp_acc, test_nemulator.get_cpu().unwrap().get_acc());
            assert_eq!(exp_x, test_nemulator.get_cpu().unwrap().get_x_reg());
            assert_eq!(exp_y, test_nemulator.get_cpu().unwrap().get_y_reg());
            assert_eq!(exp_flags, test_nemulator.get_cpu().unwrap().get_flags());
            assert_eq!(exp_clks, test_nemulator.get_cpu().unwrap().clocks());

            test_nemulator.cycle();
        }

        assert_eq!(test_nemulator.get_bus().unwrap().read(0x0002), 0);
        assert_eq!(test_nemulator.get_bus().unwrap().read(0x0003), 0);
    }
}