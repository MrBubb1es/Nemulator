use std::{collections::{HashMap, VecDeque}, path::Path, sync::{Arc, Mutex}};

use gilrs::Mapping;
use rodio::Sink;
use std::fs::File;
use std::io::BufReader;
use serde_json::Result;
use crate::system::{apu_util::NesAudioStream, controller::NesController};

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
    up: Sprite,
    down: Sprite,
    left: Sprite,
    right: Sprite,
}

impl ControllerSprite {
    pub fn new() -> Self {
        Self {
            base: Sprite::new("src/app/assets/sprites/NemulatorController.png"),
            a_button: Sprite::new("src/app/assets/sprites/a_button.png"),
            b_button: Sprite::new("src/app/assets/sprites/b_button.png"),
            select: Sprite::new("src/app/assets/sprites/select_button.png"),
            start: Sprite::new("src/app/assets/sprites/start_button.png"),
            up: Sprite::new("src/app/assets/sprites/up_button.png"),
            down: Sprite::new("src/app/assets/sprites/down_button.png"),
            left: Sprite::new("src/app/assets/sprites/left_button.png"),
            right: Sprite::new("src/app/assets/sprites/right_button.png"),
        }
    }

    pub fn draw(&self, frame: &mut [u8], frame_width: usize, frame_height: usize, x: usize, y: usize, state: NesController) {
        self.base.draw(frame, frame_width, frame_height, x, y);
        
        if state.a() {
            self.a_button.draw(frame, frame_width, frame_height, x, y)
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