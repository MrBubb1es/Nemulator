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

    // nes_app.init("games/Super Mario Bros (E).nes");
    // nes_app.init("games/Donkey Kong Classics (U).nes");
    nes_app.init("games/donkey kong.nes");

    // ===== TESTS =====

    // CPU TESTS
    // nes_app.init("prg_tests/nestest.nes"); // Passing
    // nes_app.init("prg_tests/1.Branch_Basics.nes"); // Passing
    // nes_app.init("prg_tests/2.Backward_Branch.nes"); // Passing
    // nes_app.init("prg_tests/3.Forward_Branch.nes"); // Passing

    // PPU TESTS
    // nes_app.init("prg_tests/ppu_tests/color_test.nes");
    // nes_app.init("prg_tests/ppu_tests/full_nes_palette.nes");

    event_loop.run_app(&mut nes_app).unwrap();
}
