pub mod cartridge;
pub mod graphics;
pub mod system;

use sdl2::{
    event::Event,
    keyboard::{Keycode, Mod},
    sys::KeyCode,
};
use std::time::Duration;
use system::{
    bus::Bus,
    cpu::CPU,
    nes::{self, NES},
};

use graphics::window::Window;

pub fn run() {
    let mut window = Window::new(false);
    let mut tick = 0;

    let mut nemulator = nes::NES::new("prg_tests/1.Branch_Basics.nes");

    'running: loop {
        for event in window.event_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                // | Event::KeyDown {
                //     keycode: Some(Keycode::Escape),
                //     ..
                // } => break 'running,
                _ => {}
            }
        }
        // The rest of the game loop goes here...
        // system::tick();
        window.draw(nemulator.get_cpu(), nemulator.get_bus());
        tick += 1;

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

pub fn run_debug(nes: &mut NES) {
    let mut window = Window::new(true);
    let mut tick = 0;

    let mut zoom = false;

    'running: loop {
        for event in window.event_iter() {
            match event {
                Event::Quit { .. } => break 'running,

                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    keymod: Mod::LSHIFTMOD,
                    ..
                } => {
                    if !zoom {
                        nes.cycle();
                    }
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    zoom = !zoom;
                }

                _ => {}
            }
        }

        if zoom {
            nes.cycle();
        }

        // The rest of the game loop goes here...
        // system::tick();
        // bus.write(0, tick as u8);
        window.draw(nes.get_cpu(), nes.get_bus());

        tick += 1;

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

