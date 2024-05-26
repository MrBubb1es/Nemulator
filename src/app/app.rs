use winit::event::{ElementState, KeyEvent, Modifiers, WindowEvent};
use winit::keyboard::{Key, ModifiersKeyState, ModifiersState, NamedKey};
use winit::{application::ApplicationHandler, window::WindowId};
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;

use crate::app::draw::DEFAULT_DEBUG_PAL;
use crate::system::nes::NES;

use super::draw;

#[derive(Default)]
pub struct NesApp {
    window: Option<Window>,
    pixel_buf: Option<Pixels>,
    modifiers: Option<Modifiers>,

    nes: NES,
    paused: bool,
}

impl ApplicationHandler for NesApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("NEmulator")
            .with_inner_size(PhysicalSize::new(1920, 1080));
        let window = event_loop.create_window(window_attributes).unwrap();
        let size = window.inner_size();

        self.window = Some(window);

        let pixel_surface = SurfaceTexture::new(size.width, size.height, self.window.as_ref().unwrap());

        self.pixel_buf = Some(Pixels::new(
            draw::DEBUG_FRAME_WIDTH as u32, 
            draw::DEBUG_FRAME_HEIGHT as u32, 
            pixel_surface)
            .unwrap());

        let bg_col = draw::DEFAULT_DEBUG_PAL.bg_col;
        let wgpu_bg_col = pixels::wgpu::Color{
            r: (bg_col.r as f64) / 255.0, 
            g: (bg_col.g as f64) / 255.0, 
            b: (bg_col.b as f64) / 255.0, 
            a: 1.0
        };

        dbg!(bg_col);
        dbg!(wgpu_bg_col);

        self.pixel_buf.as_mut().unwrap().clear_color(wgpu_bg_col);

        draw::draw_debug_bg(self.pixel_buf.as_mut().unwrap().frame_mut(), DEFAULT_DEBUG_PAL, &self.nes);

        self.window.as_ref().unwrap().request_redraw();

        self.modifiers = Some(Modifiers::default());
        self.paused = true;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::ModifiersChanged(new_mods) => {
                self.modifiers = Some(new_mods);
            },
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    logical_key: Key::Named(
                        NamedKey::Space,
                    ),
                    state: ElementState::Pressed,
                    repeat: false,
                    ..
                },
                ..
            } => {
                if self.modifiers.unwrap().state().shift_key() {
                    if self.paused {
                        self.nes.cycle();
                    } else {
                        self.paused = true;
                    }
                } else {
                    self.paused = !self.paused;
                }
            },
            WindowEvent::RedrawRequested => {
                // Draw.
                if let Some(buf) = self.pixel_buf.as_mut() {
                    let frame = buf.frame_mut();
                    draw::draw_debug(frame, draw::DEFAULT_DEBUG_PAL, &self.nes);

                    buf.render().unwrap();
                }
                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }

        if !self.paused {
            self.nes.cycle();
        }
    }
}

impl NesApp {
    pub fn init(&mut self, cart_path_str: &str) {
        self.nes.load_cart(cart_path_str);
    }

    pub fn nestest_init(&mut self) {
        self.nes.load_cart("prg_tests/nestest.nes");
        self.nes.set_cpu_state(Some(0xC000), None, None, None, None, None, None);
    }
}