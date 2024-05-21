pub mod cartridge;
pub mod graphics;
pub mod system;

use cartridge::cartridge::Cartridge;
use sdl2::{event::Event, keyboard::Keycode};
use std::time::Duration;
use system::{bus::Bus, cpu::CPU, ppu::PPU};

use graphics::window::Window;

pub fn test_cart(f: &str) {
    // Initialize NES
    let path = std::path::Path::new(f);
    let cart_data = std::fs::read(path).expect("Couldn't read file");
    let cart = Cartridge::from_bytes(&cart_data[..]).expect("Couldn't parse cart file");
    let bus = Bus::new(&cart);
    let mut cpu = CPU::new(&bus);
    let ppu = PPU::new(&cart);

    // Set up CPU
    // cpu.reset();
    for _ in 0..20 {
        cpu.cycle();
    }
}

// pub fn run() {
//     let mut window = Window::new(false);
//     let mut tick = 0;
//
//     let bus = Bus::new();
//     let mut cpu = CPU::new(&bus);
//
//     cpu.set_carry_flag(1);
//
//     'running: loop {
//         for event in window.event_iter() {
//             match event {
//                 Event::Quit { .. } => break 'running,
//                 // | Event::KeyDown {
//                 //     keycode: Some(Keycode::Escape),
//                 //     ..
//                 // } => break 'running,
//                 _ => {}
//             }
//         }
//         // The rest of the game loop goes here...
//         // system::tick();
//         bus.write(0, tick as u8);
//         window.draw(&cpu, &bus);
//         tick += 1;
//
//         std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
//     }
// }
//
// pub fn run_debug(cpu: &mut CPU, bus: &Bus) {
//     let mut window = Window::new(true);
//     let mut tick = 0;
//
//     'running: loop {
//         for event in window.event_iter() {
//             match event {
//                 Event::Quit { .. } => break 'running,
//
//                 Event::KeyDown {
//                     keycode: Some(Keycode::Space),
//                     ..
//                 } => { cpu.cycle(cpu.read(cpu.get_pc())); },
//
//                 _ => {}
//             }
//         }
//         // The rest of the game loop goes here...
//         // system::tick();
//         // bus.write(0, tick as u8);
//         window.draw(&cpu, &bus);
//         tick += 1;
//
//         std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
//     }
// }
