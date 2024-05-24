pub mod cartridge;
// pub mod app;
pub mod system;
pub mod app;

// use sdl2::{
//     event::Event,
//     keyboard::{Keycode, Mod},
//     sys::KeyCode,
// };
use std::time::Duration;
use system::{
    bus::Bus,
    cpu::CPU,
    nes::{self, NES},
};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};

// use app::window::Window;

#[cfg(debug_assertions)]
pub fn run() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let mut nes_app = app::NesApp::default();

    event_loop.set_control_flow(ControlFlow::Poll);

    nes_app.nestest_init();

    event_loop.run_app(&mut nes_app).unwrap();
}

#[cfg(not(debug_assertions))]
pub fn run(nes: &mut NES) {
    
}

