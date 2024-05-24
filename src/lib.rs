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

pub fn run() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let mut nes_app = app::NesApp::default();

    event_loop.set_control_flow(ControlFlow::Poll);

    println!("Starting");

    event_loop.run_app(&mut nes_app).unwrap();
}

pub fn run_debug(nes: &mut NES) {
    // let mut window = Window::new(true);
    // let mut tick = 0;

    // let mut zoom = false;

    // 'running: loop {
    //     for event in window.event_iter() {
    //         match event {
    //             Event::Quit { .. } => break 'running,

    //             Event::KeyDown {
    //                 keycode: Some(Keycode::Space),
    //                 keymod: Mod::LSHIFTMOD,
    //                 ..
    //             } => {
    //                 if !zoom {
    //                     nes.cycle();
    //                 }
    //             }

    //             Event::KeyDown {
    //                 keycode: Some(Keycode::Space),
    //                 ..
    //             } => {
    //                 zoom = !zoom;
    //             }

    //             _ => {}
    //         }
    //     }

    //     if zoom {
    //         nes.cycle();
    //     }

    //     // The rest of the game loop goes here...
    //     // system::tick();
    //     // bus.write(0, tick as u8);
    //     window.draw(&nes);

    //     tick += 1;

    //     std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    // }
}

