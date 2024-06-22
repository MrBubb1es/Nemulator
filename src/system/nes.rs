use std::{
    borrow::Borrow, cell::{Ref, RefCell, RefMut}, collections::VecDeque, fs, io::Read, rc::Rc, sync::{Arc, Mutex}
};

use crate::cartridge::{cartridge::Cartridge, mapper::{self, Mapper, NametableMirror}};

use super::{
    apu::Apu2A03,
    controller::{ControllerButton, ControllerUpdate, NesController},
    cpu::{Cpu6502, CpuState},
    ppu::Ppu2C02,
};

pub const NES_SCREEN_WIDTH: usize = 256;
pub const NES_SCREEN_HEIGHT: usize = 240;

// times 4 bc there are 4 colors per pixel: R, G, B, A
pub const NES_SCREEN_BUF_SIZE: usize = NES_SCREEN_WIDTH * NES_SCREEN_HEIGHT * 4;

pub struct Nes {
    cpu: Option<Cpu6502>,
    apu: Option<Rc<RefCell<Apu2A03>>>,
    ppu: Option<Rc<RefCell<Ppu2C02>>>,
    mapper: Option<Rc<RefCell<dyn Mapper>>>,

    p1_controller: NesController,
    p2_controller: NesController,

    // The screen buffer currently being drawn to by the ppu
    screen_buf1: Box<[u8; NES_SCREEN_BUF_SIZE]>,
    // The screen buffer currently being rendered by the app
    screen_buf2: Box<[u8; NES_SCREEN_BUF_SIZE]>,

    clocks: u64,

    cart_loaded: bool,
}

impl Default for Nes {
    fn default() -> Self {
        Nes {
            cpu: None,
            apu: None,
            ppu: None,
            mapper: None,

            p1_controller: NesController::default(),
            p2_controller: NesController::default(),

            screen_buf1: Box::new([0; NES_SCREEN_BUF_SIZE]),
            screen_buf2: Box::new([0; NES_SCREEN_BUF_SIZE]),

            clocks: 0,

            cart_loaded: false,
        }
    }
}

impl Nes {
    /// Load a new cart into this NES object
    pub fn load_cart(&mut self, cart_path_str: &str, sample_queue: Arc<Mutex<VecDeque<f32>>>) {
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

        let mapper = mapper::mapper_from_cart(cart);

        let apu = Apu2A03::new(sample_queue, Rc::clone(&mapper));
        let apu = Rc::new(RefCell::new(apu));

        let ppu = Ppu2C02::new(Rc::clone(&mapper));
        let ppu = Rc::new(RefCell::new(ppu));

        let cpu = Cpu6502::new(
            Rc::clone(&ppu), 
            Rc::clone(&apu),
            Rc::clone(&mapper));

        self.cpu = Some(cpu);
        self.apu = Some(apu);
        self.ppu = Some(ppu);
        self.mapper = Some(mapper);

        self.cart_loaded = true;
    }

    /// Remove the loaded cartridge from this NES
    pub fn remove_cart(&mut self) {
        self.cpu = None;
        self.ppu = None;
        self.mapper = None;
        self.cart_loaded = false;
    }

    /// Manually set the state of the CPU
    pub fn set_cpu_state(
        &mut self,
        pc: Option<u16>,
        sp: Option<u8>,
        acc: Option<u8>,
        x: Option<u8>,
        y: Option<u8>,
        status: Option<u8>,
        clocks: Option<u64>,
    ) {
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
            0 => Nes::update_controller_state(&mut self.p1_controller, update),
            1 => Nes::update_controller_state(&mut self.p2_controller, update),
            _ => {}
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

    // Get a reference to the APU. Does not check if a cart is loaded.
    pub fn get_apu(&self) -> Ref<Apu2A03> {
        self.apu.as_ref().unwrap().as_ref().borrow()
    }
    // Get a mutable reference to the APU. Does not check if a cart is loaded.
    pub fn get_apu_mut(&mut self) -> RefMut<Apu2A03> {
        self.apu.as_ref().unwrap().as_ref().borrow_mut()
    }

    pub fn get_mapper(&self) -> Ref<dyn Mapper> {
        self.mapper.as_ref().unwrap().as_ref().borrow()
    }

    pub fn get_mapper_mut(&mut self) -> RefMut<dyn Mapper> {
        self.mapper.as_ref().unwrap().as_ref().borrow_mut()
    }

    /// Get the number of CPU cLocks
    pub fn get_cpu_clks(&self) -> u64 {
        if let Some(cpu) = self.cpu.borrow() {
            cpu.total_clocks()
        } else {
            0
        }
    }

    pub fn reset(&mut self) {
        self.get_cpu_mut().reset();
        self.get_ppu_mut().reset();
        self.get_mapper_mut().reset();
    }

    // Cycles the system through one system clock. The PPU will cycle, the CPU
    // might cycle (CPU cycles every 3 PPU cycles). Returns a bool reporting
    // whether the CPU was cycled.
    pub fn cycle(&mut self) -> bool {
        self.ppu
            .as_ref()
            .unwrap()
            .as_ref()
            .borrow_mut()
            .cycle(self.screen_buf1.as_mut_slice());

        let mut cpu_cycled = false;

        if self.clocks % 3 == 0 {
            // APU cycles with CPU clock
            self.get_apu_mut().cycle();

            let p1_controller_state = self.p1_controller;
            let p2_controller_state = self.p2_controller;

            cpu_cycled = self
                .get_cpu_mut()
                .cycle(p1_controller_state, p2_controller_state);
        }

        self.handle_cpu_interrupts();

        self.clocks += 1;

        cpu_cycled
    }

    /// Cycle the CPU until a new instruction in executed (if cart is loaded).
    /// Also cycles the PPU.
    pub fn cycle_instr(&mut self) {
        // loop while cycle returns false => loop until cpu cycled
        while !self.cycle() {}
    }

    fn handle_cpu_interrupts(&mut self) {
        if self.get_mapper_mut().irq_requested() {
            self.get_cpu_mut().irq();
            self.get_mapper_mut().irq_handled();
        }

        if self.get_ppu().cpu_nmi_flag() {
            self.cpu.as_mut().unwrap().trigger_ppu_nmi();
            self.get_ppu_mut().set_cpu_nmi_flag(false);
        }

        if self.get_apu().trigger_irq() {
            self.get_cpu_mut().trigger_apu_irq();
            self.get_apu_mut().set_trigger_irq(false);
        }

        if self.get_apu().dmc_trigger_irq() {
            self.get_cpu_mut().trigger_apu_irq();
            self.get_apu_mut().set_dmc_irq_flag(false);
        }
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
            ppu.as_ref().borrow_mut().get_pgtbl1()
        } else {
            Box::new([0; 0x1000])
        }
    }

    pub fn get_pgtbl2(&self) -> Box<[u8; 0x1000]> {
        if let Some(ppu) = &self.ppu {
            ppu.as_ref().borrow_mut().get_pgtbl2()
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
            let prefix = format!("${:04X}:", i * 16);
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

    pub fn swap_screen_buffers(&mut self) {
        let buf_ptr1 = self.screen_buf1.as_mut_ptr() as *mut [u8; NES_SCREEN_BUF_SIZE];
        let buf_ptr2 = self.screen_buf2.as_mut_ptr() as *mut [u8; NES_SCREEN_BUF_SIZE];

        unsafe { std::ptr::swap(buf_ptr1, buf_ptr2); }
    }

    pub fn screen_buf_slice(&self) -> &[u8] {
        self.screen_buf2.as_slice()
    }

    pub fn audio_samples_queued(&self) -> usize {
        self.get_apu().audio_samples_queued()
    }

    pub fn large_sprites(&self) -> bool {
        self.get_ppu().using_large_sprites()
    }

    pub fn current_mirror_type(&self) -> NametableMirror {
        self.get_mapper().get_nt_mirror_type()
    }

    pub fn set_block_audio_samples(&mut self, val: bool) {
        self.get_apu_mut().set_block_samples(val);
    }
}