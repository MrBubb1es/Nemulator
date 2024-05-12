/* NES EMULATOR
*
* Created by MrBubbles and Logocrazymon
*
* This application is designed to be a basic implementation of a NES emulator.
*
* Planned Features:
*   - Full ability to play (legally obtained!!!) iNES and NES 2.0 roms
*   - Nerd view (mostly for debugging but it could just be neat to see what the CPU is up to)
*   - Controller support
*   - Local (Couch) Co-op
*
* Possible Future Features:
*   - Local online Co-op
*   - Online multiplayer (with servers and junk)
*/

pub mod cartridge;
pub mod drawer;
pub mod system;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

// fn main() {
//     const TEST_ARR: [u8; 4] = [0, 1, 2, 3];
//
//     let other_arr: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
//
//     let eq = TEST_ARR == other_arr[0..4];
//
//     println!("{eq}");
//
//     println!("Hello, world!");
// }
