use std::{collections::VecDeque, path::Path, sync::{Arc, Mutex}};

use crate::system::controller::{ControllerButton, NesController};

pub struct Sprite {
    sprite_rgba: Vec<u8>,
    width: usize,
    height: usize,
}

impl Sprite {
    pub fn new(path: &str) -> Self {
        let img = image::open(path).expect("Failed to open image");
        let rgba_image = img.to_rgba8();

        Self {
            sprite_rgba: rgba_image.into_raw(),
            width: img.width() as usize,
            height: img.height() as usize,
        }
    }

    fn draw(&self, frame: &mut [u8], frame_width: usize, frame_height: usize, x: usize, y: usize) {
        for (row, row_pixels) in self.sprite_rgba.chunks(self.width * 4).enumerate() {
            let row_start = ((y + row)*frame_width + x) * 4;
            let row_end = row_start + self.width * 4;
            let dest_row = &mut frame[row_start..row_end];

            dest_row.copy_from_slice(row_pixels);
        }
    }
}


pub struct ControllerSprite {
    base: Sprite,
    a_button: Sprite,
    b_button: Sprite,
    select: Sprite,
    start: Sprite,
    up_arrow: Sprite,
    down_arrow: Sprite,
    left_arrow: Sprite,
    right_arrow: Sprite,
}

impl ControllerSprite {
    pub fn new() -> Self {
        Self {
            base: Sprite::new("src/app/assets/sprites/NemulatorController.png"),
            a_button: Sprite::new("src/app/assets/sprites/a_button.png"),
            b_button: Sprite::new("src/app/assets/sprites/b_button.png"),
            select: Sprite::new("src/app/assets/sprites/select_button.png"),
            start: Sprite::new("src/app/assets/sprites/start_button.png"),
            up_arrow: Sprite::new("src/app/assets/sprites/up_arrow.png"),
            down_arrow: Sprite::new("src/app/assets/sprites/down_arrow.png"),
            left_arrow: Sprite::new("src/app/assets/sprites/left_arrow.png"),
            right_arrow: Sprite::new("src/app/assets/sprites/right_arrow.png"),
        }
    }

    pub fn draw(&self, frame: &mut [u8], frame_width: usize, frame_height: usize, x: usize, y: usize, state: NesController) {
        self.base.draw(frame, frame_width, frame_height, x, y);
        
        if state.a() {
            self.a_button.draw(frame, frame_width, frame_height, x + 156, y + 54)
        }

        if state.b() {
            self.b_button.draw(frame, frame_width, frame_height, x + 126, y + 54)
        }

        if state.up() {
            self.up_arrow.draw(frame, frame_width, frame_height, x + 30, y + 31)
        }

        if state.down() {
            self.down_arrow.draw(frame, frame_width, frame_height, x + 30, y + 61)
        }

        if state.left() {
            self.left_arrow.draw(frame, frame_width, frame_height, x + 14, y + 47)
        }

        if state.right() {
            self.right_arrow.draw(frame, frame_width, frame_height, x + 46, y + 47)
        }

        if state.select() {
            self.select.draw(frame, frame_width, frame_height, x + 78, y + 43)
        }

        if state.start() {
            self.start.draw(frame, frame_width, frame_height, x + 100, y + 43)
        }
    }
}

pub struct SliderSprite {
    base: Sprite,
    dot: Sprite,
    dot_min: usize,
    dot_max: usize,
    dot_y: usize,
}

impl SliderSprite {
    pub fn new(dot_min: usize, dot_max: usize, dot_y: usize,) -> Self {
        Self {
            base: Sprite::new("src/app/assets/sprites/slider.png"),
            dot: Sprite::new("src/app/assets/sprites/slider_dot.png"),
            dot_min,
            dot_max,
            dot_y,
        }
    }

    pub fn draw(&self, frame: &mut [u8], frame_width: usize, frame_height: usize, x: usize, y: usize, percent: f32) {
        self.base.draw(frame, frame_width, frame_height, x, y);

        let dot_x = ((self.dot_max - self.dot_min) as f32 * percent) as usize + self.dot_min;
        
        self.dot.draw(frame, frame_width, frame_height, x + dot_x, y + self.dot_y);
    }
}

pub struct MenuSound {
    raw_samples: Vec<f32>,
}

impl MenuSound {
    pub fn new(path: &str) -> Self {
        // Path to the WAV file
        let path = Path::new(path);

        // Open the WAV file
        let mut reader = hound::WavReader::open(path).expect("Failed to open WAV file");

        // Create a vector to hold the samples
        let mut raw_samples: Vec<f32> = Vec::new();

        // Check the sample format and process accordingly
        match reader.spec().sample_format {
            hound::SampleFormat::Float => {
                // Read samples as f32 directly
                for sample in reader.samples::<f32>() {
                    raw_samples.push(sample.expect("Failed to read sample"));
                }
            }
            hound::SampleFormat::Int => {
                // Read samples as i16 and convert to f32
                let max_amplitude = 2_i32.pow(reader.spec().bits_per_sample as u32 - 1) as f32;
                for sample in reader.samples::<i16>() {
                    raw_samples.push(sample.expect("Failed to read sample") as f32 / max_amplitude);
                }
            }
        }

        Self {
            raw_samples
        }
    }

    pub fn play_to_stream(&self, stream: Arc<Mutex<VecDeque<f32>>>) {
        stream.lock().unwrap().extend(self.raw_samples.clone());
    }
}

#[derive(Default)]
struct ButtonMapping {
    to_button: ControllerButton,
    from_code: Option<gilrs::ev::Code>,
    from_axis: Option<gilrs::Axis>,
    axis_dir_positive: Option<bool>,
    from_button: Option<gilrs::Button>,
    button_dir_positive: Option<bool>,
}

impl ButtonMapping {
    fn new_from_code(to_button: ControllerButton, code: gilrs::ev::Code) -> Self {
        Self {
            to_button,
            from_code: Some(code),
            ..Default::default()
        }
    }

    fn new_from_axis(to_button: ControllerButton, axis: gilrs::Axis, positive_dir: bool) -> Self {
        Self {
            to_button,
            from_axis: Some(axis),
            axis_dir_positive: Some(positive_dir),
            ..Default::default()
        }
    }

    fn new_from_button(to_button: ControllerButton, button: gilrs::Button, positive_dir: bool) -> Self {
        Self {
            to_button,
            from_button: Some(button),
            button_dir_positive: Some(positive_dir),
            ..Default::default()
        }
    }

    fn set_mapping_from_code(&mut self, code: gilrs::ev::Code) {
        self.from_code = Some(code);
        self.from_axis = None;
        self.axis_dir_positive = None;
        self.from_button = None;
    }

    fn set_mapping_from_axis(&mut self, axis: gilrs::Axis, positive_dir: bool) {
        self.from_code = None;
        self.from_axis = Some(axis);
        self.axis_dir_positive = Some(positive_dir);
        self.from_button = None;
    }

    fn set_mapping_from_button(&mut self, button: gilrs::Button, positive_dir: bool) {
        self.from_code = None;
        self.from_axis = None;
        self.axis_dir_positive = None;
        self.from_button = Some(button);
        self.button_dir_positive = Some(positive_dir);
    }

    fn is_this_button(&self, controller_event: gilrs::Event) -> bool {
        if let Some(code) = self.from_code {
            match controller_event.event {
                gilrs::EventType::AxisChanged(_, _, ev_code) |
                gilrs::EventType::ButtonChanged(_, _, ev_code) => {
                    return code == ev_code;
                }

                _ => {}
            }
        }

        if let (Some(axis), Some(dir_positive)) = (self.from_axis, self.axis_dir_positive) {
            match controller_event.event {
                gilrs::EventType::AxisChanged(ev_axis, ev_dir, _) => {
                    return (axis == ev_axis) && (dir_positive == (ev_dir > 0.0));
                }

                _ => {}
            }
        }

        if let (Some(button), Some(dir_positive)) = (self.from_button, self.button_dir_positive) {
            match controller_event.event {
                gilrs::EventType::ButtonChanged(ev_button, ev_dir, _) => {
                    return (button == ev_button) && (dir_positive == (ev_dir > 0.0));
                }

                _ => {}
            }
        }

        return false;
    }

    fn is_this_button_from_button(&self, ev_button: gilrs::Button, val: f32) -> bool {
        if let (Some(button), Some(dir_positive)) = (self.from_button, self.button_dir_positive) {
            if button == ev_button {
                return (dir_positive == (val > 0.0)) || (val.abs() < 0.5);
            }
        }

        false
    }
}

pub struct ControllerMapping {
    a_map: ButtonMapping,
    b_map: ButtonMapping,
    up_map: ButtonMapping,
    down_map: ButtonMapping,
    left_map: ButtonMapping,
    right_map: ButtonMapping,
    select_map: ButtonMapping,
    start_map: ButtonMapping,
}

impl Default for ControllerMapping {
    fn default() -> Self {
        Self {
            a_map: ButtonMapping::new_from_button(ControllerButton::A, gilrs::Button::South, true),
            b_map: ButtonMapping::new_from_button(ControllerButton::B, gilrs::Button::East, true),
            up_map: ButtonMapping::new_from_button(ControllerButton::Up, gilrs::Button::DPadUp, false),
            down_map: ButtonMapping::new_from_button(ControllerButton::Down, gilrs::Button::DPadUp, true),
            right_map: ButtonMapping::new_from_button(ControllerButton::Right, gilrs::Button::DPadRight, true),
            left_map: ButtonMapping::new_from_button(ControllerButton::Left, gilrs::Button::DPadRight, false),
            select_map: ButtonMapping::new_from_button(ControllerButton::Select, gilrs::Button::Select, true),
            start_map: ButtonMapping::new_from_button(ControllerButton::Start, gilrs::Button::Start, true),
        }
    }
}

impl ControllerMapping {
    pub fn set_button_mapping(&mut self, to_button: ControllerButton, from_button: gilrs::Button, val: f32) {
        let button_map = match to_button {
            ControllerButton::A => &mut self.a_map,
            ControllerButton::B => &mut self.b_map,
            ControllerButton::Up => &mut self.up_map,
            ControllerButton::Down => &mut self.down_map,
            ControllerButton::Left => &mut self.left_map,
            ControllerButton::Right => &mut self.right_map,
            ControllerButton::Select => &mut self.select_map,
            ControllerButton::Start => &mut self.start_map,
        };

        button_map.set_mapping_from_button(from_button, val > 0.0);
    }

    pub fn get_mapped_button(&self, from_button: gilrs::Button, val: f32) -> Option<ControllerButton> {
        if self.a_map.is_this_button_from_button(from_button, val) {
            Some( ControllerButton::A )
        } else if self.b_map.is_this_button_from_button(from_button, val) {
            Some( ControllerButton::B )
        } else if self.up_map.is_this_button_from_button(from_button, val) {
            Some( ControllerButton::Up )
        } else if self.down_map.is_this_button_from_button(from_button, val) {
            Some( ControllerButton::Down )
        } else if self.left_map.is_this_button_from_button(from_button, val) {
            Some( ControllerButton::Left )
        } else if self.right_map.is_this_button_from_button(from_button, val) {
            Some( ControllerButton::Right )
        } else if self.select_map.is_this_button_from_button(from_button, val) {
            Some( ControllerButton::Select )
        } else if self.start_map.is_this_button_from_button(from_button, val) {
            Some( ControllerButton::Start )
        } else {
            None
        }
    }
}