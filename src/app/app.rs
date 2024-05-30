use bitfield_struct::bitfield;
use winit::event::{ElementState, KeyEvent, Modifiers, WindowEvent};
use winit::keyboard::{Key, KeyCode, ModifiersKeyState, ModifiersState, NamedKey, PhysicalKey, SmolStr};
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
pub enum ViewMode {
    #[default]
    NORMAL,
    DEBUG,
}

#[bitfield(u8)]
pub struct ControllerState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub start: bool,
    pub select: bool,
    pub a: bool,
    pub b: bool,
}

pub struct NesApp {
    window: Option<Window>,
    pixel_buf: Option<Pixels>,
    modifiers: Option<Modifiers>,

    player1_controller: Option<ControllerState>,
    player2_controller: Option<ControllerState>,

    nes: NES,
    paused: bool,
    view_mode: ViewMode,

    last_frame: std::time::Instant,
}

impl Default for NesApp {
    fn default() -> Self {
        NesApp {
            window: None,
            pixel_buf: None,
            modifiers: None,

            player1_controller: None,
            player2_controller: None,

            nes: NES::default(),
            paused: false,
            view_mode: ViewMode::default(),

            last_frame: std::time::Instant::now(),
        }
    }
}

impl ApplicationHandler for NesApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("NEmulator")
            .with_inner_size(PhysicalSize::new(1920, 1080));
        let window = event_loop.create_window(window_attributes).unwrap();
        let size = window.inner_size();

        self.window = Some(window);
        
        let bg_col = draw::DEFAULT_DEBUG_PAL.bg_col;
        let wgpu_bg_col = pixels::wgpu::Color{
            r: (bg_col.r as f64) / 255.0, 
            g: (bg_col.g as f64) / 255.0, 
            b: (bg_col.b as f64) / 255.0, 
            a: 1.0
        };

        let pixel_surface = SurfaceTexture::new(size.width, size.height, self.window.as_ref().unwrap());

        match self.view_mode {
            ViewMode::DEBUG => {
                self.pixel_buf = Some(Pixels::new(
                    draw::DEBUG_FRAME_WIDTH as u32, 
                    draw::DEBUG_FRAME_HEIGHT as u32, 
                    pixel_surface)
                    .unwrap());
        
                self.pixel_buf.as_mut().unwrap().clear_color(wgpu_bg_col);

                draw::draw_debug_bg(self.pixel_buf.as_mut().unwrap().frame_mut(), DEFAULT_DEBUG_PAL, &self.nes);
            },
            ViewMode::NORMAL => {
                self.pixel_buf = Some(Pixels::new(
                    draw::GAME_FRAME_WIDTH as u32, 
                    draw::GAME_FRAME_HEIGHT as u32, 
                    pixel_surface)
                    .unwrap());
                
                self.pixel_buf.as_mut().unwrap().clear_color(wgpu_bg_col);

                draw::draw_game_view_bg(self.pixel_buf.as_mut().unwrap().frame_mut(), DEFAULT_DEBUG_PAL);
            },
        }

        self.window.as_ref().unwrap().request_redraw();

        self.player1_controller = Some(ControllerState::default());
        self.player2_controller = Some(ControllerState::default());

        self.modifiers = Some(Modifiers::default());
        self.paused = true;

        self.last_frame = std::time::Instant::now();
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
                    // repeat: false,
                    ..
                },
                ..
            } => {
                // if !self.nes.handle_input()
                if self.modifiers.unwrap().state().shift_key() {
                    if self.paused {
                        self.nes.cycle_instr();
                    } else {
                        self.paused = true;
                    }
                } else {
                    self.paused = !self.paused;
                }
            },
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::KeyV),
                    state: ElementState::Pressed,
                    repeat: false,
                    ..
                },
                ..
            } => {
                self.switch_view_mode();
            },
            WindowEvent::RedrawRequested => {
                if !self.paused {
                    self.nes.cycle_until_frame();
                }
                // Draw.
                if let Some(buf) = self.pixel_buf.as_mut() {
                    let frame = buf.frame_mut();

                    match self.view_mode {
                        ViewMode::DEBUG => {
                            draw::draw_debug(frame, draw::DEFAULT_DEBUG_PAL, &mut self.nes);
                        },
                        ViewMode::NORMAL => {
                            draw::draw_game_view(frame, &mut self.nes);
                        }
                    }

                    buf.render().unwrap();
                }
                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();

                // let micros_elapsed = self.last_frame.elapsed().as_micros();
                // let fps = 1000000 / micros_elapsed;
                // println!("FPS: {fps}");
                self.last_frame = std::time::Instant::now();
            }
            _ => (),
        }
    }
}

impl NesApp {
    pub fn init(&mut self, cart_path_str: &str) {
        self.nes.load_cart(cart_path_str);
    }

    pub fn nestest_init(&mut self) {
        self.nes.load_cart("prg_tests/nestest.nes");
        // self.nes.set_cpu_state(Some(0xC000), None, None, None, None, None, None);
    }

    pub fn switch_view_mode(&mut self) {
        match self.view_mode {
            ViewMode::DEBUG => {
                let buf = self.pixel_buf.as_mut().unwrap();
                
                buf.resize_buffer(draw::GAME_FRAME_WIDTH as u32, 
                    draw::GAME_FRAME_HEIGHT as u32).unwrap();

                let frame = buf.frame_mut();
                
                for i in 0..draw::GAME_FRAME_HEIGHT {
                    for j in 0..draw::GAME_FRAME_WIDTH {
                        let pix_idx = (i * draw::GAME_FRAME_WIDTH + j) * 4; 
                        frame[pix_idx + 0] = 0x00;
                        frame[pix_idx + 1] = 0x00;
                        frame[pix_idx + 2] = 0x00;
                        frame[pix_idx + 3] = 0xFF;
                    }
                }

                self.view_mode = ViewMode::NORMAL;
                draw::draw_game_view_bg(frame, draw::DEFAULT_DEBUG_PAL);
            },
            ViewMode::NORMAL => {
                let buf = self.pixel_buf.as_mut().unwrap();
                
                buf.resize_buffer(draw::DEBUG_FRAME_WIDTH as u32, 
                    draw::DEBUG_FRAME_HEIGHT as u32).unwrap();
                
                let frame = buf.frame_mut();
                
                for i in 0..draw::DEBUG_FRAME_HEIGHT {
                    for j in 0..draw::DEBUG_FRAME_WIDTH {
                        let pix_idx = (i * draw::DEBUG_FRAME_WIDTH + j) * 4; 
                        frame[pix_idx + 0] = 0x00;
                        frame[pix_idx + 1] = 0x00;
                        frame[pix_idx + 2] = 0x00;
                        frame[pix_idx + 3] = 0xFF;
                    }
                }

                self.view_mode = ViewMode::DEBUG;
                draw::draw_debug_bg(frame, DEFAULT_DEBUG_PAL, &self.nes);
            }
        }
    }
}