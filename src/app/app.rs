use gilrs::{Gilrs, Mapping};
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use rodio::Sink;
use winit::dpi::PhysicalSize;
use winit::event::{self, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;
use winit::{application::ApplicationHandler, window::WindowId};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::app::draw::DEFAULT_DEBUG_PAL;
use crate::system::controller::{ControllerButton, ControllerReadState, ControllerUpdate, NesController};
use crate::system::nes::Nes;
use crate::RuntimeConfig;

use super::util::{ControllerSprite, MenuSound, SliderSprite};
use super::draw::{self, draw_paused_menu_bg, GAME_FRAME_HEIGHT, GAME_FRAME_WIDTH};

const MICROS_PER_FRAME: u128 = 1_000_000 / 60;
const MIN_SAMPLES_THRESH: usize = 600;
const VOLUME_CONTROL_SPEED: f32 = 0.05;
const MAX_VOLUME: f32 = 2.0;

#[derive(Default, Clone, Copy, PartialEq)]
pub enum PauseMenuItem {
    #[default]
    Continue,
    ControllerMap,
    Volume,
    NoLimit,
    Quit,
}

impl PauseMenuItem {
    /// Selects the next menu item, wrapping if the last one is selected and wrap
    /// is true. Returns a bool reporting whether a new item was selected.
    fn go_next(&mut self, wrap: bool) -> bool {
        let new_val = match self {
            Self::Continue => Self::ControllerMap,
            Self::ControllerMap => Self::Volume,
            Self::Volume => Self::NoLimit,
            Self::NoLimit => Self::Quit,
            Self::Quit => {
                if wrap {
                    Self::Continue
                } else {
                    Self::Quit
                }
            },
        };

        let new_val_chosen = new_val != *self;
        *self = new_val;

        new_val_chosen
    }

    /// Selects the previous menu item, wrapping if the first one is selected and wrap
    /// is true. Returns a bool reporting whether a new item was selected.
    fn go_prev(&mut self, wrap: bool) -> bool {
        let new_val = match self {
            Self::Continue => {
                if wrap {
                    Self::Quit
                } else {
                    Self::Continue
                }
            },
            Self::ControllerMap => Self::Continue,
            Self::Volume => Self::ControllerMap,
            Self::NoLimit => Self::Volume,
            Self::Quit => Self::NoLimit,
        };

        let new_val_chosen = new_val != *self;
        *self = new_val;

        new_val_chosen
    }
}

pub struct PauseMenu {
    pub selected: PauseMenuItem,

    pub mapping_controller: bool,
    pub map_controller1: bool,
    pub map_controller2: bool,
    pub test_controller1: bool,
    pub test_controller2: bool,
    pub controller_state: NesController,
    pub controller_read: ControllerReadState,
    pub controller_sprite: ControllerSprite,

    pub setting_volume: bool,
    pub volume_percent: f32,
    pub slider_sprite: SliderSprite,

    pub move_sound: MenuSound,
    pub select_sound: MenuSound,
    pub reject_sound: MenuSound,
    pub pause_sound: MenuSound,
}

impl PauseMenu {
    fn new() -> Self {
        Self {
            selected: PauseMenuItem::Continue,

            mapping_controller: false,
            map_controller1: false,
            map_controller2: false,
            test_controller1: false,
            test_controller2: false,
            controller_state: NesController::default(),
            controller_read: ControllerReadState::new(),
            controller_sprite: ControllerSprite::new(),

            setting_volume: false,
            volume_percent: 0.50,
            slider_sprite: SliderSprite::new(20, 200, 26),

            move_sound: MenuSound::new("src/app/assets/sounds/move.wav"),
            select_sound: MenuSound::new("src/app/assets/sounds/select.wav"),
            reject_sound: MenuSound::new("src/app/assets/sounds/reject.wav"),
            pause_sound: MenuSound::new("src/app/assets/sounds/pause.wav"),
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum ViewMode {
    #[default]
    Normal,
    Debug,
}

pub struct NesApp {
    window: Option<Window>,
    pixel_buf: Option<Pixels>,
    
    audio_sink: Option<Sink>,
    audio_stream_queue: Option<Arc<Mutex<VecDeque<f32>>>>,

    nes: Nes,
    paused: bool,
    view_mode: ViewMode,
    pause_menu: PauseMenu,

    controller_handler: Gilrs,
    controller1_map: Mapping,
    controller2_map: Mapping,

    limit_fps: bool,
    can_debug: bool,
    last_frame: std::time::Instant,
    fps: usize,
    frame_count: u64,

    // Flag keeping track of if the NES was just reset.
    reset: bool,
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
        let wgpu_bg_col = pixels::wgpu::Color {
            r: (bg_col.r as f64) / 255.0,
            g: (bg_col.g as f64) / 255.0,
            b: (bg_col.b as f64) / 255.0,
            a: 1.0,
        };

        let pixel_surface =
            SurfaceTexture::new(size.width, size.height, self.window.as_ref().unwrap());

        match self.view_mode {
            ViewMode::Debug => {
                let pixels_builder = PixelsBuilder::new(
                    draw::DEBUG_FRAME_WIDTH as u32,
                    draw::DEBUG_FRAME_HEIGHT as u32,
                    pixel_surface,
                )
                .enable_vsync(false)
                .clear_color(wgpu_bg_col);

                self.pixel_buf = Some(pixels_builder.build().unwrap());

                draw::draw_debug_bg(
                    self.pixel_buf.as_mut().unwrap().frame_mut(),
                    DEFAULT_DEBUG_PAL,
                    &self.nes,
                );
            }
            ViewMode::Normal => {
                let pixels_builder = PixelsBuilder::new(
                    draw::GAME_FRAME_WIDTH as u32,
                    draw::GAME_FRAME_HEIGHT as u32,
                    pixel_surface,
                )
                .enable_vsync(false)
                .clear_color(wgpu_bg_col);

                self.pixel_buf = Some(pixels_builder.build().unwrap());

                draw::draw_game_view_bg(
                    self.pixel_buf.as_mut().unwrap().frame_mut(),
                    DEFAULT_DEBUG_PAL,
                );
            }
        }

        self.window.as_ref().unwrap().request_redraw();

        self.last_frame = std::time::Instant::now();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        win_event: WindowEvent,
    ) {
        self.handle_controller_input();

        match win_event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }

            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event, event_loop);
            }

            WindowEvent::Resized(new_size) => {
                if let Some(buf) = self.pixel_buf.as_mut() {
                    let _ = buf.resize_surface(new_size.width, new_size.height);
                }
            }

            WindowEvent::RedrawRequested => {

                let micros_since_frame = self.last_frame.elapsed().as_micros();

                if !self.limit_fps || 
                    micros_since_frame > MICROS_PER_FRAME || 
                    (!self.paused && self.nes.audio_samples_queued() < MIN_SAMPLES_THRESH) {

                    self.last_frame = std::time::Instant::now();

                    self.fps = (1_000_000 / micros_since_frame) as usize;
                    
                    if let Some(buf) = self.pixel_buf.as_mut() {
                        let frame = buf.frame_mut();
    
                        if self.can_debug {
                            match self.view_mode {
                                ViewMode::Debug => {
                                    draw::draw_debug(frame, draw::DEFAULT_DEBUG_PAL, &mut self.nes, self.fps);
                                }
                                ViewMode::Normal => {
                                    draw::draw_game_view(frame, &mut self.nes);
                                }
                            }
                        } else {
                            if !self.paused {
                                draw::draw_game_view(frame, &mut self.nes);
                            } else {

                                if self.pause_menu.mapping_controller && self.frame_count % 32 == 0 {
                                    self.pause_menu.controller_state = NesController::from_bits(
                                        !self.pause_menu.controller_state.into_bits()
                                    );
                                }

                                draw::draw_menu(frame, &self.pause_menu, self.limit_fps);
                            }
                        }
    
                        buf.render().unwrap();
                    }

                    if !self.paused {
                        self.nes.cycle_until_frame();
                        self.nes.swap_screen_buffers();
                    }

                    self.frame_count += 1;
                }

                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

impl NesApp {
    pub fn new() -> Self {
        Self {
            window: None,
            pixel_buf: None,

            audio_sink: None,
            audio_stream_queue: None,

            nes: Nes::default(),
            paused: false,
            view_mode: ViewMode::default(),
            pause_menu: PauseMenu::new(),

            controller_handler: Gilrs::new().unwrap(),
            controller1_map: Mapping::new(),
            controller2_map: Mapping::new(),

            limit_fps: false,
            can_debug: true,
            last_frame: std::time::Instant::now(),
            fps: 0,
            frame_count: 0,

            reset: false,
        }
    }

    pub fn init(&mut self, config: RuntimeConfig, sample_queue: Arc<Mutex<VecDeque<f32>>>) {
        self.audio_stream_queue = Some(Arc::clone(&sample_queue));
        self.nes.load_cart(&config.cart_path, sample_queue);
        self.limit_fps = config.limit_fps;
        self.can_debug = config.can_debug;
    }

    pub fn switch_view_mode(&mut self) {
        match self.view_mode {
            ViewMode::Debug => {
                let buf = self.pixel_buf.as_mut().unwrap();

                buf.resize_buffer(
                    draw::GAME_FRAME_WIDTH as u32,
                    draw::GAME_FRAME_HEIGHT as u32,
                )
                .unwrap();

                let frame = buf.frame_mut();

                self.view_mode = ViewMode::Normal;

                draw::draw_game_view_bg(frame, draw::DEFAULT_DEBUG_PAL);
            }
            ViewMode::Normal => {
                let buf = self.pixel_buf.as_mut().unwrap();

                buf.resize_buffer(
                    draw::DEBUG_FRAME_WIDTH as u32,
                    draw::DEBUG_FRAME_HEIGHT as u32,
                )
                .unwrap();

                let frame = buf.frame_mut();

                frame.fill(0);

                self.view_mode = ViewMode::Debug;

                draw::draw_debug_bg(frame, DEFAULT_DEBUG_PAL, &self.nes);
            }
        }
    }

    /// If the input should go to the NES (i.e. it is controller input), then
    /// this function creates the controller state
    fn handle_nes_input(&mut self, event: KeyEvent) -> bool {
        if event.repeat {
            return false;
        }

        let new_state = event.state == ElementState::Pressed;

        const PLAYER1_ID: usize = 0;
        const PLAYER2_ID: usize = 1;

        let controller_update: Option<ControllerUpdate> = match event.physical_key {
            // Player 1 keys
            PhysicalKey::Code(KeyCode::KeyZ) => Some(ControllerUpdate {
                button: ControllerButton::A,
                player_id: PLAYER1_ID,
                pressed: new_state,
            }),
            PhysicalKey::Code(KeyCode::KeyX) => Some(ControllerUpdate {
                button: ControllerButton::B,
                player_id: PLAYER1_ID,
                pressed: new_state,
            }),
            PhysicalKey::Code(KeyCode::ShiftRight) => Some(ControllerUpdate {
                button: ControllerButton::Select,
                player_id: PLAYER1_ID,
                pressed: new_state,
            }),
            PhysicalKey::Code(KeyCode::Enter) => Some(ControllerUpdate {
                button: ControllerButton::Start,
                player_id: PLAYER1_ID,
                pressed: new_state,
            }),
            PhysicalKey::Code(KeyCode::ArrowUp) => Some(ControllerUpdate {
                button: ControllerButton::Up,
                player_id: PLAYER1_ID,
                pressed: new_state,
            }),
            PhysicalKey::Code(KeyCode::ArrowDown) => Some(ControllerUpdate {
                button: ControllerButton::Down,
                player_id: PLAYER1_ID,
                pressed: new_state,
            }),
            PhysicalKey::Code(KeyCode::ArrowLeft) => Some(ControllerUpdate {
                button: ControllerButton::Left,
                player_id: PLAYER1_ID,
                pressed: new_state,
            }),
            PhysicalKey::Code(KeyCode::ArrowRight) => Some(ControllerUpdate {
                button: ControllerButton::Right,
                player_id: PLAYER1_ID,
                pressed: new_state,
            }),
            _ => None,
        };

        if let Some(update) = controller_update {
            self.nes.update_controllers(update);
            return true;
        }

        false
    }

    fn handle_controller_input(&mut self) {
        const DPAD_PRESSED_THRESH: f32 = 0.90;
        // Handle controller input
        if let Some(controller_event) = self.controller_handler.next_event() {

            if self.paused && self.pause_menu.mapping_controller {
                let map = if self.pause_menu.map_controller1 {
                    &mut self.controller1_map
                } else {
                    &mut self.controller2_map
                };

                Self::add_mapping(map, controller_event);

                self.pause_menu.controller_read = self.pause_menu.controller_read.next();
                
                return;
            }

            let update = match controller_event {
                // Up / Down input
                gilrs::Event {
                    id,
                    event: gilrs::EventType::ButtonChanged(gilrs::Button::DPadUp, val, ..),
                    ..
                } => {
                    let down_pressed = val > DPAD_PRESSED_THRESH;
                    let up_pressed = val < (1.0 - DPAD_PRESSED_THRESH);

                    self.nes.update_controllers(ControllerUpdate {
                        button: ControllerButton::Down,
                        player_id: 0,
                        pressed: down_pressed,
                    });
                    self.nes.update_controllers(ControllerUpdate {
                        button: ControllerButton::Up,
                        player_id: 0,
                        pressed: up_pressed,
                    });
                }

                // Left / Right input
                gilrs::Event {
                    id,
                    event: gilrs::EventType::ButtonChanged(gilrs::Button::DPadRight, val, ..),
                    ..
                } => {
                    let right_pressed = val > DPAD_PRESSED_THRESH;
                    let left_pressed = val < (1.0 - DPAD_PRESSED_THRESH);

                    self.nes.update_controllers(ControllerUpdate {
                        button: ControllerButton::Right,
                        player_id: 0,
                        pressed: right_pressed,
                    });
                    self.nes.update_controllers(ControllerUpdate {
                        button: ControllerButton::Left,
                        player_id: 0,
                        pressed: left_pressed,
                    });
                }

                // Start button input
                gilrs::Event {
                    id,
                    event: gilrs::EventType::ButtonChanged(gilrs::Button::Start, val, ..),
                    ..
                } => {
                    let start_pressed = val > 0.50;

                    self.nes.update_controllers(ControllerUpdate {
                        button: ControllerButton::Start,
                        player_id: 0,
                        pressed: start_pressed,
                    });
                }

                // Select button input
                gilrs::Event {
                    id,
                    event: gilrs::EventType::ButtonChanged(gilrs::Button::Select, val, ..),
                    ..
                } => {
                    let select_pressed = val > 0.50;

                    self.nes.update_controllers(ControllerUpdate {
                        button: ControllerButton::Select,
                        player_id: 0,
                        pressed: select_pressed,
                    });
                }

                // A button pressed
                gilrs::Event {
                    id,
                    event: gilrs::EventType::ButtonChanged(gilrs::Button::South, val, ..),
                    ..
                } => {
                    let a_pressed = val > 0.50;

                    self.nes.update_controllers(ControllerUpdate {
                        button: ControllerButton::A,
                        player_id: 0,
                        pressed: a_pressed,
                    });
                }

                // B button pressed
                gilrs::Event {
                    id,
                    event: gilrs::EventType::ButtonChanged(gilrs::Button::East, val, ..),
                    ..
                } => {
                    let b_pressed = val > 0.50;

                    self.nes.update_controllers(ControllerUpdate {
                        button: ControllerButton::B,
                        player_id: 0,
                        pressed: b_pressed,
                    });
                }

                _ => (),
            };
        }
    }

    fn handle_menu_input(&mut self, event: KeyEvent, event_loop: &ActiveEventLoop) -> bool {

        if self.pause_menu.mapping_controller {
            if self.pause_menu.map_controller1 {
                if self.frame_count % 60 == 0 {
                    let read_state = self.pause_menu.controller_read;
                    let current_button = read_state.button();
                    let current_val = self.pause_menu.controller_state.read_button(read_state) == 1;

                    self.pause_menu
                        .controller_state
                        .set_button(current_button, !current_val);
                }

            }
            false
        }
        // Using volume slider
        else if self.pause_menu.setting_volume {

            // Ignore when keys are released
            if event.state == ElementState::Released {
                return false;
            }

            match event.physical_key {
                PhysicalKey::Code(KeyCode::KeyZ) |
                PhysicalKey::Code(KeyCode::Enter) => {
                    self.update_audio_volume(self.pause_menu.volume_percent);
                    self.pause_menu.setting_volume = false;

                    self.play_menu_sound(&self.pause_menu.select_sound);

                    true
                },
                PhysicalKey::Code(KeyCode::ArrowUp) => {
                    self.play_menu_sound(&self.pause_menu.reject_sound);

                    true
                },
                PhysicalKey::Code(KeyCode::ArrowDown) => {
                    self.play_menu_sound(&self.pause_menu.reject_sound);

                    true
                },
                PhysicalKey::Code(KeyCode::ShiftRight) => {
                    self.play_menu_sound(&self.pause_menu.reject_sound);

                    true
                },
                PhysicalKey::Code(KeyCode::ArrowLeft) => {
                    if self.pause_menu.volume_percent > 1e-3 {
                        self.pause_menu.volume_percent -= VOLUME_CONTROL_SPEED;
    
                        if self.pause_menu.volume_percent < 0.0 {
                            self.pause_menu.volume_percent = 0.0;
                            self.play_menu_sound(&self.pause_menu.reject_sound);
                        } else {
                            self.play_menu_sound(&self.pause_menu.move_sound);
                        }
    
                        self.update_audio_volume(self.pause_menu.volume_percent);
                        
                    } else if !event.repeat {
                        self.play_menu_sound(&self.pause_menu.reject_sound);
                    }

                    true
                },
                PhysicalKey::Code(KeyCode::ArrowRight) => {
                    if self.pause_menu.volume_percent < 0.999 {
                        self.pause_menu.volume_percent += VOLUME_CONTROL_SPEED;
    
                        if self.pause_menu.volume_percent > 1.0 {
                            self.pause_menu.volume_percent = 1.0;
                            self.play_menu_sound(&self.pause_menu.reject_sound);
                        } else {
                            self.play_menu_sound(&self.pause_menu.move_sound);
                        }
    
                        self.update_audio_volume(self.pause_menu.volume_percent);
                        
                    } else if !event.repeat {
                        self.play_menu_sound(&self.pause_menu.reject_sound);
                    }

                    true
                },
                
                _ => { false }
            }
        }
        // Selecting menu option
        else {
            // Ignore when keys are released
            if event.state == ElementState::Released || event.repeat {
                return false;
            }

            let handled = match event.physical_key {
                PhysicalKey::Code(KeyCode::KeyZ) |
                PhysicalKey::Code(KeyCode::Enter) => {
                    match self.pause_menu.selected {
                        PauseMenuItem::Continue => {
                            self.unpause();
                        }

                        PauseMenuItem::ControllerMap => {
                            self.pause_menu.mapping_controller = true;

                            self.play_menu_sound(&self.pause_menu.select_sound);
                        }

                        PauseMenuItem::Volume => {
                            self.pause_menu.setting_volume = true;

                            self.play_menu_sound(&self.pause_menu.select_sound);
                        }

                        PauseMenuItem::NoLimit => {
                            self.limit_fps = !self.limit_fps;
                            self.nes.set_block_audio_samples(!self.limit_fps);

                            self.play_menu_sound(&self.pause_menu.select_sound);
                        }

                        PauseMenuItem::Quit => { 
                            println!("Quit button pressed, exiting.");
                            event_loop.exit();
                        }

                        _ => {},
                    }

                    true
                },
                PhysicalKey::Code(KeyCode::ArrowUp) => {
                    let cursor_moved = self.pause_menu.selected.go_prev(false);

                    self.play_menu_sound(
                        if cursor_moved { &self.pause_menu.move_sound }
                        else { &self.pause_menu.reject_sound }
                    );

                    true
                },
                PhysicalKey::Code(KeyCode::ArrowDown) => {
                    let cursor_moved = self.pause_menu.selected.go_next(false);

                    self.play_menu_sound(
                        if cursor_moved { &self.pause_menu.move_sound }
                        else { &self.pause_menu.reject_sound }
                    );

                    true
                }
                PhysicalKey::Code(KeyCode::ShiftRight) => {
                    let cursor_moved = self.pause_menu.selected.go_next(true);

                    self.play_menu_sound(
                        if cursor_moved { &self.pause_menu.move_sound }
                        else { &self.pause_menu.reject_sound }
                    );

                    true
                },
                PhysicalKey::Code(KeyCode::ArrowLeft) => {
                    self.play_menu_sound(&self.pause_menu.reject_sound);

                    true
                },
                PhysicalKey::Code(KeyCode::ArrowRight) => {
                    self.play_menu_sound(&self.pause_menu.reject_sound);

                    true
                },
                
                _ => { false }
            };

            handled
        }
    }

    fn handle_keyboard_input(&mut self, event: KeyEvent, event_loop: &ActiveEventLoop) {
        let handled = if !self.paused {
            self.handle_nes_input(event.clone())
        } else {
            self.handle_menu_input(event.clone(), event_loop)
        };

        if !handled {
            match event {
                KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::KeyV),
                    state: ElementState::Pressed,
                    repeat: false,
                    ..
                } => {
                    if self.can_debug {
                        self.switch_view_mode();
                    }
                },

                KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::KeyC),
                    state: ElementState::Pressed,
                    ..
                } => {
                    if self.paused && self.can_debug {
                        self.nes.cycle_instr();
                    }
                }

                KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::KeyF),
                    state: ElementState::Pressed,
                    ..
                } => {
                    if self.paused && self.can_debug {
                        self.nes.cycle_until_frame();
                        self.nes.swap_screen_buffers();
                    }
                }

                KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::KeyR),
                    state: ElementState::Pressed,
                    repeat: true,
                    ..
                } => {
                    if !self.reset {
                        self.nes.reset();
                        self.reset = true;
                    }
                }

                KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::KeyR),
                    state: ElementState::Released,
                    repeat: false,
                    ..
                } => {
                    self.reset = false;
                }

                KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                    state: ElementState::Pressed,
                    repeat: false,
                    ..
                } => {
                    if self.paused {
                        self.unpause();
                    } else {
                        self.pause();

                        // Pause menu gui can't be opened if debug is enabled
                        if !self.can_debug {
                            if let Some(buf) = self.pixel_buf.as_mut() {
                                let frame = buf.frame_mut();
        
                                draw_paused_menu_bg(frame);
                            }
                        }
                    }
                }

                _ => {}
            }
        }
    }

    pub fn attatch_sound_sink(&mut self, sink: Sink) {
        self.audio_sink = Some(sink);
        self.update_audio_volume(self.pause_menu.volume_percent);
    }

    fn update_audio_volume(&mut self, max_volume_percent: f32) {
        if let Some(sink) = &mut self.audio_sink {
            sink.set_volume(MAX_VOLUME * max_volume_percent);
        }
    }

    fn play_menu_sound(&self, sound: &MenuSound) {
        if let Some(stream) = self.audio_stream_queue.clone() {

            stream.lock().unwrap().clear();

            sound.play_to_stream(stream);
        }
    }

    fn pause(&mut self) {
        self.paused = true;
        if let Some(stream) = self.audio_stream_queue.clone() {
            stream.lock().unwrap().clear();

            self.play_menu_sound(&self.pause_menu.pause_sound);
        }
    }

    fn unpause(&mut self) {
        self.paused = false;
        if let Some(stream) = self.audio_stream_queue.clone() {
            stream.lock().unwrap().clear();
        }
    }

    fn add_mapping(map: &mut Mapping, event: gilrs::Event) {

    }
}

