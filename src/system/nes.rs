use std::{borrow::Borrow, cell::{Ref, RefCell, RefMut}, fs, io::Read, rc::Rc};

use crate::cartridge::{cartridge::Cartridge, mapper::Mapper};

use super::{controller::{ControllerButton, ControllerUpdate, NesController}, cpu::{self, Cpu6502, CpuState}, ppu::Ppu2C02, ppu_util::PpuMask};

pub struct NES {
    cpu: Option<Cpu6502>,
    ppu: Option<Rc<RefCell<Ppu2C02>>>,
    mapper: Option<Rc<RefCell<dyn Mapper>>>,

    p1_controller: NesController,
    p2_controller: NesController,

    clocks: u64,

    // Keeps track of when the CPU has initiated an OAM DMA transfer
    dma_in_progress: bool,

    cart: Option<Cartridge>,
    cart_loaded: bool,
}

impl Default for NES {
    fn default() -> Self {
        NES {
            cpu: None,
            ppu: None,
            mapper: None,

            p1_controller: NesController::default(),
            p2_controller: NesController::default(),

            clocks: 0,

            dma_in_progress: false,

            cart: None,
            cart_loaded: false,
        }
    }
}

impl NES {
    /// Create a new NES object with a cart pre-loaded
    pub fn with_cart(cart_path_str: &str) -> Self {
        let mut nes = NES::default();
        nes.load_cart(cart_path_str);

        nes
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

        // Parse cartridge from file bytes
        let cart = Cartridge::from_bytes(data.as_slice()).unwrap();

        // let ppu_regs = Rc::new(PpuRegisters::default());
        let mapper: Rc<RefCell<dyn Mapper>> = cart.get_mapper();

        let ppu = Rc::new(RefCell::new(
            Ppu2C02::new(
                cart.get_chr_rom(), 
                Rc::clone(&mapper),
            ))
        );
        let cpu = Cpu6502::new(cart.get_prg_rom(), Rc::clone(&ppu), Rc::clone(&mapper));

        self.cpu = Some(cpu);
        self.ppu = Some(ppu);
        self.mapper = Some(mapper);

        self.cart = Some(cart);
        self.cart_loaded = true;
    }

    /// Remove the loaded cartridge from this NES
    pub fn remove_cart(&mut self) {
        self.cpu = None;
        self.ppu = None;
        self.mapper = None;

        self.cart = None;
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
                        clocks: Option<u64>) {
        if self.cart_loaded {
            let cpu = self.cpu.as_mut().unwrap();

            cpu.set_pc(pc.unwrap_or(cpu.get_pc()));
            cpu.set_sp(sp.unwrap_or(cpu.get_sp()));
            cpu.set_acc(acc.unwrap_or(cpu.get_acc()));
            cpu.set_x_reg(x.unwrap_or(cpu.get_x_reg()));
            cpu.set_y_reg(y.unwrap_or(cpu.get_y_reg()));
            cpu.set_status(status.unwrap_or(cpu.get_status()));
            cpu.set_total_clocks(clocks.unwrap_or(cpu.total_clocks()));
        }
    }

    pub fn update_controllers(&mut self, update: ControllerUpdate) {
        match update.player_id {
            0 => NES::update_controller_state(&mut self.p1_controller, update),
            1 => NES::update_controller_state(&mut self.p2_controller, update),
            _ => {},
        }
    }

    fn update_controller_state(controller: &mut NesController, update: ControllerUpdate) {
        match update.button {
            ControllerButton::A => controller.set_a(update.pressed),
            ControllerButton::B => controller.set_b(update.pressed),
            ControllerButton::Select => controller.set_select(update.pressed),
            ControllerButton::Start => controller.set_start(update.pressed),
            ControllerButton::Up => controller.set_up(update.pressed),
            ControllerButton::Down => controller.set_down(update.pressed),
            ControllerButton::Left => controller.set_left(update.pressed),
            ControllerButton::Right => controller.set_right(update.pressed),
        }
    }

    /// Get a reference to the CPU. Does not check if a cart is loaded.
    pub fn get_cpu(&self) -> &Cpu6502 {
        self.cpu.as_ref().unwrap()
    }

    /// Get a mutable reference to the CPU. Does not check if a cart is loaded.
    pub fn get_cpu_mut(&mut self) -> &mut Cpu6502 {
        self.cpu.as_mut().unwrap()
    }

    // Get a reference to the PPU. Does not check if a cart is loaded.
    pub fn get_ppu(&self) -> Ref<Ppu2C02> {
        self.ppu.as_ref().unwrap().as_ref().borrow()
    }

    // Get a mutable reference to the PPU. Does not check if a cart is loaded.
    pub fn get_ppu_mut(&self) -> RefMut<Ppu2C02> {
        self.ppu.as_ref().unwrap().as_ref().borrow_mut()
    }

    /// Get the number of CPU cLocks
    pub fn get_cpu_clks(&self) -> u64 {
        if let Some(cpu) = self.cpu.borrow() {
            cpu.total_clocks()
        } else {
            0
        }
    }

    // Cycles the system through one system clock. The PPU will cycle, the CPU
    // might cycle (CPU cycles every 3 PPU cycles). Returns a bool reporting 
    // whether the CPU was cycled.
    pub fn cycle(&mut self) -> bool {
        self.get_ppu_mut().cycle();


        let mut cpu_cycled = false;
        
        if self.clocks % 3 == 0 {
            if self.get_cpu().dma_in_progress() {

                // Even CPU cycles are read cycles
                if self.get_cpu().total_clocks() & 1 == 0 {
                    self.get_cpu_mut().read_next_oam_data();
                    self.get_cpu_mut().increment_clock();
                } 
                // Odd CPU cycles are write cycles
                else {
                    let addr = self.get_cpu().get_oam_addr();
                    let data = self.get_cpu().get_oam_data();

                    self.get_ppu_mut().oam_dma_write(data, addr);
                    self.get_cpu_mut().increment_clock();
                }

            } else {

                let p1_controller_state = self.p1_controller;
                let p2_controller_state = self.p2_controller;

                cpu_cycled = self.get_cpu_mut().cycle(p1_controller_state, p2_controller_state);
            }
        } else {
            cpu_cycled = false;
        };

        if self.get_ppu().cpu_nmi_flag() {
            self.cpu.as_mut().unwrap().trigger_ppu_nmi();
            self.get_ppu_mut().set_cpu_nmi_flag(false);
        }

        self.clocks += 1;

        cpu_cycled
    }

    /// Cycle the CPU until a new instruction in executed (if cart is loaded).
    /// Also cycles the PPU.
    pub fn cycle_instr(&mut self) {
        // loop while cycle returns false => loop until cpu cycled
        while !self.cycle() {}
    }

    pub fn get_cpu_state(&self) -> CpuState {
        if let Some(cpu) = &self.cpu {
            cpu.get_state()
        } else {
            CpuState::default()
        }
    }

    pub fn get_pgtbl1(&self) -> Box<[u8; 0x1000]> {
        if let Some(ppu) = &self.ppu {
            ppu.as_ref().borrow().pgtbl1.clone()
        } else {
            Box::new([0; 0x1000])
        }
    }

    pub fn get_pgtbl2(&self) -> Box<[u8; 0x1000]> {
        if let Some(ppu) = &self.ppu {
            ppu.as_ref().borrow().pgtbl2.clone() // fix this later i too tired
        } else {
            Box::new([0; 0x1000])
        }
    }

    pub fn cycle_until_frame(&mut self) {
        if self.cart_loaded {
            while !self.get_ppu().frame_finished() {
                self.cycle();
            }

            self.get_ppu_mut().set_frame_finished(false);
        }
    }

    /// Get a string showing the contents of the Zero Page of system ram
    pub fn zpage_str(&mut self) -> String {
        let mut mem_str: String = String::from("");

        for i in 0..16 {
            let prefix = format!("${:04X}:", i*16);
            mem_str.push_str(&prefix);
            for j in 0..16 {
                let mem_val = if let Some(cpu) = &mut self.cpu {
                    cpu.read(i * 16 + j)
                } else {
                    0xEE
                };
                let val_str = format!(" {mem_val:02X}");
                mem_str.push_str(&val_str);
            }
            let suffix = "\n";
            mem_str.push_str(&suffix);
        }
    
        mem_str
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::NES;

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
                    temp[6] as u64)         // clocks
                }
            )
        }


        let mut test_nemulator = NES::with_cart("prg_tests/nestest.nes");
        test_nemulator.set_cpu_state(Some(0xC000), None, None, None, None, None, None); // run tests automatically

        for i in 0..expected_vals.len() {
            // test_nemulator.cpu.as_ref().unwrap().print_state();

            let (exp_pc, exp_sp, exp_acc, exp_x, exp_y, exp_flags, exp_clks) = expected_vals[i];

            assert_eq!(exp_pc, test_nemulator.get_cpu().get_pc());
            assert_eq!(exp_sp, test_nemulator.get_cpu().get_sp());
            assert_eq!(exp_acc, test_nemulator.get_cpu().get_acc());
            assert_eq!(exp_x, test_nemulator.get_cpu().get_x_reg());
            assert_eq!(exp_y, test_nemulator.get_cpu().get_y_reg());
            assert_eq!(exp_flags, test_nemulator.get_cpu().get_status());
            assert_eq!(exp_clks, test_nemulator.get_cpu().total_clocks());

            test_nemulator.cycle_instr()
        }

        assert_eq!(test_nemulator.get_cpu().read(0x0002), 0);
        assert_eq!(test_nemulator.get_cpu().read(0x0003), 0);
    }
}