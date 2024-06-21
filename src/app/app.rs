use gilrs::{GamepadId, Gilrs, Mapping};
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

use super::util::{ControllerMapping, ControllerSprite, MenuSound, SliderSprite};
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
    pub player1_map_selected: bool,
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
            player1_map_selected: true,
            controller_state: NesController::default(),
            controller_read: ControllerReadState::new(),
            controller_sprite: ControllerSprite::new(),

            setting_volume: false,
            volume_percent: 0.25,
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
    controller1_map: ControllerMapping,
    controller2_map: ControllerMapping,
    p1_controller_id: Option<gilrs::GamepadId>,
    p2_controller_id: Option<gilrs::GamepadId>,

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
                                    let new_state = (self.frame_count >> 5) & 1 == 1; // Alternates about every 1/2 second

                                    self.pause_menu.controller_state = match self.pause_menu.controller_read.button() {
                                        ControllerButton::A => NesController::new().with_a(new_state),
                                        ControllerButton::B => NesController::new().with_b(new_state),
                                        ControllerButton::Up => NesController::new().with_up(new_state),
                                        ControllerButton::Down => NesController::new().with_down(new_state),
                                        ControllerButton::Left => NesController::new().with_left(new_state),
                                        ControllerButton::Right => NesController::new().with_right(new_state),
                                        ControllerButton::Select => NesController::new().with_select(new_state),
                                        ControllerButton::Start => NesController::new().with_start(new_state),
                                    };
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
            controller1_map: ControllerMapping::default(),
            controller2_map: ControllerMapping::default(),
            p1_controller_id: None,
            p2_controller_id: None,

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

        self.find_gamepads();
    }

    // Find the first two connected gamepads
    fn find_gamepads(&mut self) {
        let mut player1: Option<GamepadId> = None;
        let mut player2: Option<GamepadId> = None;

        for (_id, gamepad) in self.controller_handler.gamepads() {
            if gamepad.is_connected() {
                if player1.is_none() {
                    player1 = Some(gamepad.id());
                } else if player2.is_none() {
                    player2 = Some(gamepad.id());
                    break;
                }
            }
        }

        self.p1_controller_id = player1;
        self.p2_controller_id = player2;
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

    /// If the input should go to the NES, then
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

    fn handle_nes_controller_input(&mut self, button: gilrs::Button, val: f32, gamepad_id: GamepadId) {
        if self.controller_handler.connected_gamepad(gamepad_id).is_some() {
            
            let mut button_map = None;
            let mut player_id = 0;

            if let Some(p1_id) = self.p1_controller_id {
                if p1_id == gamepad_id {
                    button_map = Some(&self.controller1_map);
                    player_id = 0;
                }
            }

            if let Some(p2_id) = self.p2_controller_id {
                if p2_id == gamepad_id {
                    button_map = Some(&self.controller2_map);
                    player_id = 1;
                }
            }

            if button_map.is_none() {
                return;
            }
            let button_map = button_map.unwrap();

            if let Some(nes_button) = button_map.get_mapped_button(button, val) {
                let pressed = val.abs() > 0.5;

                let controller_update = ControllerUpdate {
                    button: nes_button,
                    player_id,
                    pressed: pressed,
                };

                self.nes.update_controllers(controller_update);

                if !pressed {
                    let extra_update = match nes_button {
                        ControllerButton::Up => {Some(ControllerUpdate{
                            button: ControllerButton::Down,
                            player_id,
                            pressed: false,
                        })}

                        ControllerButton::Down => {Some(ControllerUpdate{
                            button: ControllerButton::Up,
                            player_id,
                            pressed: false,
                        })}

                        ControllerButton::Right => {Some(ControllerUpdate{
                            button: ControllerButton::Left,
                            player_id,
                            pressed: false,
                        })}

                        ControllerButton::Left => {Some(ControllerUpdate{
                            button: ControllerButton::Right,
                            player_id,
                            pressed: false,
                        })}

                        _ => { None }
                    };

                    if let Some(update) = extra_update {
                        self.nes.update_controllers(update);
                    }
                }
            }
        }
    }

    fn handle_controller_input(&mut self) {
        // Handle controller input
        if let Some(controller_event) = self.controller_handler.next_event() {

            match controller_event.event {
                gilrs::EventType::Connected |
                gilrs::EventType::Disconnected => {
                    self.find_gamepads();
                }

                gilrs::EventType::ButtonChanged(button, val, _) => {
                    // Puts dpad inputs on a range from -1 to 1
                    let val = match button {
                        gilrs::Button::DPadUp |
                        gilrs::Button::DPadDown |
                        gilrs::Button::DPadLeft |
                        gilrs::Button::DPadRight => {
                            2.0 * val - 1.0
                        }

                        _ => { val }
                    };

                    if !self.paused {
                        self.handle_nes_controller_input(button, val, controller_event.id);
                    } else {
                        self.handle_menu_controller_input(button, val, controller_event.id);
                    }
                }

                _ => {}
            };
        }
    }

    fn handle_menu_input(&mut self, event: KeyEvent, event_loop: &ActiveEventLoop) -> bool {

        if self.pause_menu.mapping_controller {

            if !self.pause_menu.map_controller1 && !self.pause_menu.map_controller2 {
                if event.repeat || event.state == ElementState::Released {
                    return false;
                }

                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyZ) |
                    PhysicalKey::Code(KeyCode::Enter) => {
                        self.pause_menu.controller_read = ControllerReadState::new();

                        if self.pause_menu.player1_map_selected {
                            self.controller1_map = ControllerMapping::default();
                            self.pause_menu.map_controller1 = true;
                        } else {
                            self.controller2_map = ControllerMapping::default();
                            self.pause_menu.map_controller2 = true;
                        }

                        self.play_menu_sound(&self.pause_menu.select_sound);

                        true
                    },
                    PhysicalKey::Code(KeyCode::KeyX) => {
                        self.pause_menu.mapping_controller = false;

                        self.play_menu_sound(&self.pause_menu.reject_sound);

                        true
                    }
                    PhysicalKey::Code(KeyCode::ArrowUp) |
                    PhysicalKey::Code(KeyCode::ArrowDown) |
                    PhysicalKey::Code(KeyCode::ShiftRight) => {
                        self.pause_menu.player1_map_selected = !self.pause_menu.player1_map_selected;

                        self.play_menu_sound(&self.pause_menu.move_sound);
    
                        true
                    },
                    PhysicalKey::Code(KeyCode::ArrowLeft) |
                    PhysicalKey::Code(KeyCode::ArrowRight) => {
                        self.play_menu_sound(&self.pause_menu.reject_sound);
    
                        true
                    },
                    
                    _ => { false }
                }
            } else {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Escape) |
                    PhysicalKey::Code(KeyCode::KeyX) => {
                        self.pause_menu.mapping_controller = false;
                        self.pause_menu.map_controller1 = false;
                        self.pause_menu.map_controller2 = false;

                        self.pause_menu.controller_read = ControllerReadState::new();
                        
                        self.play_menu_sound(&self.pause_menu.reject_sound);

                        true
                    }

                    _ => { false }
                }
            }
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
                            self.pause_menu.controller_read = ControllerReadState::new();

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
                PhysicalKey::Code(KeyCode::KeyX) => {
                    self.unpause();

                    true
                }
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

    fn handle_menu_controller_input(&mut self, button: gilrs::Button, val: f32, gamepad_id: GamepadId) {
        if val.abs() < 0.5 || !self.pause_menu.mapping_controller {
            return;
        }

        let mut mapped_button = false;
                
        if self.pause_menu.map_controller1 {
            if let Some(p2_id) = self.p2_controller_id {
                if p2_id == gamepad_id {
                    self.p2_controller_id = None;
                }
            }

            self.p1_controller_id = Some(gamepad_id);

            self.controller1_map.set_button_mapping(
                self.pause_menu.controller_read.button(),
                button,
                val,
            );
            mapped_button = true;

        } else if self.pause_menu.map_controller2 {
            if let Some(p1_id) = self.p1_controller_id {
                if p1_id == gamepad_id {
                    self.p1_controller_id = None;
                }
            }

            self.p2_controller_id = Some(gamepad_id);

            self.controller2_map.set_button_mapping(
                self.pause_menu.controller_read.button(),
                button,
                val,
            );
            mapped_button = true;
        }

        if mapped_button {
            self.pause_menu.controller_read = self.pause_menu.controller_read.next();
        
            if self.pause_menu.controller_read.finished() {
                self.pause_menu.mapping_controller = false;
                self.pause_menu.map_controller1 = false;
                self.pause_menu.map_controller2 = false;
            }
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
}

