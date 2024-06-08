use std::sync::Mutex;

use gilrs::Gilrs;
use winit::event::{ElementState, KeyEvent, Modifiers, WindowEvent};
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
use winit::{application::ApplicationHandler, window::WindowId};
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};

use crate::app::draw::DEFAULT_DEBUG_PAL;
use crate::system::controller::{ControllerButton, ControllerUpdate};
use crate::system::nes::Nes;

use super::draw;

#[derive(Default)]
pub enum ViewMode {
    #[default]
    NORMAL,
    DEBUG,
}

pub struct NesApp {
    window: Option<Window>,
    pixel_buf: Option<Pixels>,

    nes: Mutex<Nes>,
    paused: bool,
    view_mode: ViewMode,

    controller_handler: Gilrs,

    last_frame: std::time::Instant,
}

impl Default for NesApp {
    fn default() -> Self {
        NesApp {
            window: None,
            pixel_buf: None,

            nes: Mutex::new(Nes::default()),
            paused: false,
            view_mode: ViewMode::default(),

            controller_handler: Gilrs::new().unwrap(),

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

        let vsync_enable = true;

        match self.view_mode {
            ViewMode::DEBUG => {
                let buf_builder = PixelsBuilder::new(
                    draw::DEBUG_FRAME_WIDTH as u32, 
                    draw::DEBUG_FRAME_HEIGHT as u32, 
                    pixel_surface)
                    .enable_vsync(vsync_enable)
                    .clear_color(wgpu_bg_col);
        
                self.pixel_buf = Some(buf_builder.build().unwrap());

                draw::draw_debug_bg(self.pixel_buf.as_mut().unwrap().frame_mut(), DEFAULT_DEBUG_PAL, self.nes.lock().as_ref().unwrap());
            },
            ViewMode::NORMAL => {
                let buf_builder = PixelsBuilder::new(
                    draw::GAME_FRAME_WIDTH as u32, 
                    draw::GAME_FRAME_HEIGHT as u32, 
                    pixel_surface)
                    .enable_vsync(vsync_enable)
                    .clear_color(wgpu_bg_col);
        
                self.pixel_buf = Some(buf_builder.build().unwrap());

                draw::draw_game_view_bg(self.pixel_buf.as_mut().unwrap().frame_mut(), DEFAULT_DEBUG_PAL);
            },
        }

        self.window.as_ref().unwrap().request_redraw();

        self.paused = true;

        self.last_frame = std::time::Instant::now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, win_event: WindowEvent) {
        // Check for and handle controller input
        self.handle_controller_input();
        
        match win_event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },

            WindowEvent::KeyboardInput {
                event, ..
            } => {
                self.handle_keyboard_input(event);
            },

            WindowEvent::RedrawRequested => {
                let sys_thread = tokio::spawn(async {
                    let nes = self.nes.lock().as_mut().unwrap();

                    nes.cycle_until_frame();
                });

                // Draw.
                if let Some(buf) = self.pixel_buf.as_mut() {
                    let frame = buf.frame_mut();
                    let mut nes_lock = self.nes.lock();
                    let nes = nes_lock.as_mut().unwrap();

                    match self.view_mode {
                        ViewMode::DEBUG => {
                            draw::draw_debug(frame, draw::DEFAULT_DEBUG_PAL, nes);
                        },
                        ViewMode::NORMAL => {
                            draw::draw_game_view(frame, nes);
                        }
                    }
                    
                    buf.render().unwrap();
                }
                

                self.window.as_ref().unwrap().request_redraw();

                // let micros_elapsed = self.last_frame.elapsed().as_micros();
                // let fps = 1000000 / micros_elapsed;
                // println!("FPS: {fps}");
                // self.last_frame = std::time::Instant::now();
            }
            _ => (),
        }
    }
}

impl NesApp {
    pub fn init(&mut self, cart_path_str: &str) {
        self.nes.lock().as_mut().unwrap().load_cart(cart_path_str);
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
                draw::draw_debug_bg(frame, DEFAULT_DEBUG_PAL, self.nes.lock().as_ref().unwrap());
            }
        }
    }

    fn handle_keyboard_input(&mut self, event: KeyEvent) {
        let mut handled = false;
                
        if !event.repeat {
            handled = self.handle_nes_input(event.clone());
        }

        if !handled {
            match event {
                KeyEvent { 
                    physical_key: PhysicalKey::Code(KeyCode::KeyV),
                    state: ElementState::Pressed,
                    repeat: false,
                    ..
                } => {
                    self.switch_view_mode()
                },

                KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::KeyC),
                    state: ElementState::Pressed,
                    ..
                } => {
                    if self.paused {
                        self.nes.lock().as_mut().unwrap().cycle_instr();
                    }
                }

                KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::KeyF),
                    state: ElementState::Pressed,
                    ..
                } => {
                    if self.paused {
                        self.nes.lock().as_mut().unwrap().cycle_until_frame();
                    }
                },

                KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::Space),
                    state: ElementState::Pressed,
                    ..
                } => {
                    self.paused = !self.paused;
                },

                _ => {},
            }
        }
    }

    /// If the input should go to the NES (i.e. it is controller input), then
    /// this function creates the controller state
    fn handle_nes_input(&mut self, event: KeyEvent) -> bool {
        let new_state = event.state == ElementState::Pressed;

        const PLAYER1_ID: usize = 0;
        const PLAYER2_ID: usize = 1;

        let controller_update: Option<ControllerUpdate> = match event.physical_key {
            // Player 1 keys
            PhysicalKey::Code(KeyCode::KeyZ)       => Some(ControllerUpdate{button: ControllerButton::A,      player_id: PLAYER1_ID, pressed: new_state}),
            PhysicalKey::Code(KeyCode::KeyX)       => Some(ControllerUpdate{button: ControllerButton::B,      player_id: PLAYER1_ID, pressed: new_state}),
            PhysicalKey::Code(KeyCode::ShiftRight) => Some(ControllerUpdate{button: ControllerButton::Select, player_id: PLAYER1_ID, pressed: new_state}),
            PhysicalKey::Code(KeyCode::Enter)      => Some(ControllerUpdate{button: ControllerButton::Start,  player_id: PLAYER1_ID, pressed: new_state}),
            PhysicalKey::Code(KeyCode::ArrowUp)    => Some(ControllerUpdate{button: ControllerButton::Up,     player_id: PLAYER1_ID, pressed: new_state}),
            PhysicalKey::Code(KeyCode::ArrowDown)  => Some(ControllerUpdate{button: ControllerButton::Down,   player_id: PLAYER1_ID, pressed: new_state}),
            PhysicalKey::Code(KeyCode::ArrowLeft)  => Some(ControllerUpdate{button: ControllerButton::Left,   player_id: PLAYER1_ID, pressed: new_state}),
            PhysicalKey::Code(KeyCode::ArrowRight) => Some(ControllerUpdate{button: ControllerButton::Right,  player_id: PLAYER1_ID, pressed: new_state}),
            _ => None,
        };

        if let Some(update) = controller_update {
            self.nes.lock().as_mut().unwrap().update_controllers(update);
            return true;
        }

        false
    }

    fn handle_controller_input(&mut self) {
        const DPAD_PRESSED_THRESH: f32 = 0.90;
        // Handle controller input
        if let Some(controller_event) = self.controller_handler.next_event() {
            let mut nes_lock = self.nes.lock();
            let nes = nes_lock.as_mut().unwrap();

            let update = match controller_event {
                // Up / Down input
                gilrs::Event { id, event: gilrs::EventType::ButtonChanged(gilrs::Button::DPadUp, val, ..), .. } => {                    
                    let down_pressed = val > DPAD_PRESSED_THRESH;
                    let up_pressed = val < (1.0 - DPAD_PRESSED_THRESH);

                    nes.update_controllers(
                        ControllerUpdate{
                            button: ControllerButton::Down,
                            player_id: 0,
                            pressed: down_pressed });
                    nes.update_controllers(
                        ControllerUpdate{
                            button: ControllerButton::Up,
                            player_id: 0,
                            pressed: up_pressed });
                }

                // Left / Right input
                gilrs::Event { id, event: gilrs::EventType::ButtonChanged(gilrs::Button::DPadRight, val, ..), .. } => {                    
                    let right_pressed = val > DPAD_PRESSED_THRESH;
                    let left_pressed = val < (1.0 - DPAD_PRESSED_THRESH);

                    nes.update_controllers(
                        ControllerUpdate{
                            button: ControllerButton::Right,
                            player_id: 0,
                            pressed: right_pressed });
                    nes.update_controllers(
                        ControllerUpdate{
                            button: ControllerButton::Left,
                            player_id: 0,
                            pressed: left_pressed });
                }

                // Start button input
                gilrs::Event { id, event: gilrs::EventType::ButtonChanged(gilrs::Button::Start, val, .. ), .. } => {
                    let start_pressed = val > 0.50;

                    nes.update_controllers(
                        ControllerUpdate{
                            button: ControllerButton::Start,
                            player_id: 0,
                            pressed: start_pressed });
                }

                // Select button input
                gilrs::Event { id, event: gilrs::EventType::ButtonChanged(gilrs::Button::Select, val, .. ), .. } => {
                    let select_pressed = val > 0.50;

                    nes.update_controllers(
                        ControllerUpdate{
                            button: ControllerButton::Select,
                            player_id: 0,
                            pressed: select_pressed });
                }

                // A button pressed
                gilrs::Event { id, event: gilrs::EventType::ButtonChanged(gilrs::Button::South, val, .. ), .. } => {
                    let a_pressed = val > 0.50;

                    nes.update_controllers(
                        ControllerUpdate{
                            button: ControllerButton::A,
                            player_id: 0,
                            pressed: a_pressed });
                }

                // B button pressed
                gilrs::Event { id, event: gilrs::EventType::ButtonChanged(gilrs::Button::East, val, .. ), .. } => {
                    let b_pressed = val > 0.50;

                    nes.update_controllers(
                        ControllerUpdate{
                            button: ControllerButton::B,
                            player_id: 0,
                            pressed: b_pressed });
                }

                _ => (),
            };
        }
    }
}