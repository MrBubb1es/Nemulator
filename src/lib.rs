pub mod app;
pub mod cartridge;
pub mod system;

use winit::event_loop::{ControlFlow, EventLoop};

// use app::window::Window;

pub fn run(path: &str) {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let mut nes_app = app::NesApp::default();

    event_loop.set_control_flow(ControlFlow::Poll);

    nes_app.init(path);
    // nes_app.init("games/Donkey Kong Classics (U).nes");
    // nes_app.init("games/donkey kong.nes");

    // ===== TESTS =====

    // CPU TESTS
    // nes_app.init("prg_tests/nestest.nes"); // Passing
    // nes_app.init("prg_tests/1.Branch_Basics.nes"); // Passing
    // nes_app.init("prg_tests/2.Backward_Branch.nes"); // Passing
    // nes_app.init("prg_tests/3.Forward_Branch.nes"); // Passing

    // PPU TESTS
    // nes_app.init("prg_tests/ppu_tests/color_test.nes"); // Passing (except color emphasis)
    // nes_app.init("prg_tests/ppu_tests/full_nes_palette.nes"); // Shows some colors, not all colors

    // nes_app.init("prg_tests/ppu_tests/sprite_hit_tests_2005.10.05/01.basics.nes"); // Passing
    // nes_app.init("prg_tests/ppu_tests/sprite_hit_tests_2005.10.05/02.alignment.nes"); // Passing
    // nes_app.init("prg_tests/ppu_tests/sprite_hit_tests_2005.10.05/03.corners.nes"); // Passing
    // nes_app.init("prg_tests/ppu_tests/sprite_hit_tests_2005.10.05/04.flip.nes"); // Passing
    // nes_app.init("prg_tests/ppu_tests/sprite_hit_tests_2005.10.05/05.left_clip.nes"); // Passing
    // nes_app.init("prg_tests/ppu_tests/sprite_hit_tests_2005.10.05/06.right_edge.nes"); // Passing
    // nes_app.init("prg_tests/ppu_tests/sprite_hit_tests_2005.10.05/07.screen_bottom.nes"); // Failing
    // nes_app.init("prg_tests/ppu_tests/sprite_hit_tests_2005.10.05/08.double_height.nes"); // Failing

    // nes_app.init("prg_tests/ppu_tests/sprite_overflow_tests/1.Basics.nes"); // Passing
    // nes_app.init("prg_tests/ppu_tests/sprite_overflow_tests/2.Details.nes"); // Passing
    // nes_app.init("prg_tests/ppu_tests/sprite_overflow_tests/3.Timing.nes"); // Failing

    nes_app.init("prg_tests/ppu_tests/ppu_read_buffer/test_ppu_read_buffer.nes");

    event_loop.run_app(&mut nes_app).unwrap();
}
