pub mod cartridge;
pub mod graphics;
pub mod system;

use sdl2::{event::Event, keyboard::Keycode};
use system::{bus::Bus, cpu::CPU};
use std::time::Duration;

use graphics::window::Window;

pub fn run() {
    let mut window = Window::new(false);
    let mut tick = 0;

    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.set_carry_flag(1);

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
        window.draw(&cpu, &bus);
        tick += 1;

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

pub fn run_debug(cpu: &mut CPU, bus: &Bus) {
    let mut window = Window::new(true);
    let mut tick = 0;

    'running: loop {
        for event in window.event_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => { cpu.cycle(cpu.read(cpu.get_pc())); },
                
                _ => {}
            }
        }
        // The rest of the game loop goes here...
        // system::tick();
        // bus.write(0, tick as u8);
        window.draw(&cpu, &bus);
        tick += 1;

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}