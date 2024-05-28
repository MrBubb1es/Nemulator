pub mod cartridge;
pub mod system;
pub mod app;

use winit::event_loop::{ControlFlow, EventLoop};

// use app::window::Window;

pub fn run() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let mut nes_app = app::NesApp::default();

    event_loop.set_control_flow(ControlFlow::Poll);

    nes_app.nestest_init();
    // nes_app.init("games/Super Mario Bros (E).nes");
    // nes_app.init("prg_tests/1.Branch_Basics.nes");
    // 1.Branch_Basics

    event_loop.run_app(&mut nes_app).unwrap();
}
