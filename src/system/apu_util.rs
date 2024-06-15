use std::{collections::{vec_deque, VecDeque}, ops::Rem, sync::{Arc, Mutex}, time::Duration};

use bitfield_struct::bitfield;
use rodio::Source;

use super::apu::{CPU_CYCLE_PERIOD, NES_AUDIO_FREQUENCY};

/// https://www.nesdev.org/wiki/APU_Length_Counter#Table_structure
/// Lookup table for the lengths of the notes given a 5 bit number
const PULSE_COUNTER_LOOKUP: [usize; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 
    160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30
];


#[derive(Debug, Default, Clone)]
pub struct NesAudioStream {
    sample_queue: Arc<Mutex<VecDeque<f32>>>,
}

impl Iterator for NesAudioStream {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let mut sample_queue = self.sample_queue.lock().unwrap();

        Some(sample_queue.pop_front().unwrap_or(0.0))
    }
}

impl Source for NesAudioStream {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        NES_AUDIO_FREQUENCY
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl NesAudioStream {
    pub fn new() -> (Self, Arc<Mutex<VecDeque<f32>>>) {
        let sample_queue = Arc::new(Mutex::new(VecDeque::new()));
        let stream = Self { 
            sample_queue: Arc::clone(&sample_queue) 
        };

        (stream, sample_queue)
    }

    pub fn push_sample(&mut self, sample: f32) {
        self.sample_queue.lock().unwrap().push_back(sample);
    }

    pub fn push_sample_batch(&mut self, sample_batch: Vec<f32>) {
        self.sample_queue.lock().unwrap().extend(sample_batch.into_iter());
    }

    // Clones the audio stream and empties the current sample queue
    // pub fn drain_as_clone(&mut self) -> Self {
    //     Self {
    //         sample_queue: self.sample_queue.drain(..).collect(),
    //     }
    // }

    // pub fn is_full(&self) -> bool { self.sample_queue.len() >= SAMPLE_BATCH_SIZE }
}


/// This struct encapsulates all 4 registers of the pulse
/// channels into a single object.
#[bitfield(u32)]
pub struct PulseRegisters {
    // First byte
    #[bits(2)]
    pub duty_cycle: u8,
    #[bits(1)]
    pub disable: bool,
    #[bits(1)]
    pub const_volume: bool,
    #[bits(4)]
    pub envelope_volume: u8,

    // Second byte
    #[bits(1)]
    pub sweep_enabled: bool,
    #[bits(3)]
    pub sweep_period: u8,
    #[bits(1)]
    pub sweep_negative: bool,
    #[bits(3)]
    pub sweep_shift: u8,

    // Third byte
    #[bits(8)]
    pub timer_lo: u8,

    // Fourth byte
    #[bits(5)]
    pub length_counter_load: u8,
    #[bits(3)]
    pub timer_hi: u8,
}

#[derive(Default)]
pub struct PulseChannel {
    pub timer: usize,

    pub freq: f64,
    pub enabled: bool,
    pub duty_cycle_percent: f64,

    pub length_counter: usize,
    pub counter_enabled: bool,
}

impl PulseChannel {
    pub fn sample(&mut self, total_clocks: u64) -> f32 {
        let mut sample = 0.0;

        if self.enabled && self.timer > 8 {
            if self.length_counter > 0 || !self.counter_enabled {
                let time = total_clocks as f64 * CPU_CYCLE_PERIOD;
    
                let remainder = (time * self.freq).fract();
    
                if remainder < self.duty_cycle_percent {
                    sample = 1.0;
                } else {
                    sample = 0.0;
                }
            }
        }

        sample
    }

    pub fn set_length_counter(&mut self, data: u8) {
        self.length_counter = PULSE_COUNTER_LOOKUP[data as usize];
    }

    pub fn update_counter(&mut self) {
        if !self.enabled {
            self.length_counter = 0;
        } else if self.length_counter > 0 && self.counter_enabled {
            self.length_counter -= 1;
        }
    }
}

/// This struct encapsulates all 3 registers used for the
/// triangle channel of the APU.
#[bitfield(u32)]
pub struct TriangleRegisters {
    // First byte
    #[bits(1)]
    pub counter_disabled: bool,
    #[bits(7)]
    pub counter_reload: u8,

    // Second byte
    #[bits(8)]
    pub timer_lo: u8,

    // Third byte
    #[bits(5)]
    pub counter_load: u8,
    #[bits(3)]
    pub timer_hi: u8,

    #[bits(8)]
    _unused: u8,
}

/// This struct encapsulates all 3 registers of the noise channel in the APU.
#[bitfield(u16)]
pub struct NoiseRegisters {
    // First byte
    // #[bits(2)]
    // _unused: u8,
    #[bits(1)]
    pub loop_disabled: bool,
    #[bits(1)]
    pub const_volume: bool,
    #[bits(4)]
    pub volume: u8,

    // Second byte
    #[bits(1)]
    pub loop_noise: bool,
    // #[bits(3)]
    // _unused: u8,
    #[bits(4)]
    pub period: u8,

    // Third byte
    #[bits(5)]
    pub counter_load: u8,
    // #[bits(3)]
    // _unused: u8,

    // #[bits(8)]
    // _unused: u8,
}

/// This struct encapsulates all of the DMC registers in the APU.
#[bitfield(u32)]
pub struct DmcRegisters {
    // First byte
    #[bits(1)]
    pub irq_enabled: bool,
    #[bits(1)]
    pub loop_enabled: bool,
    #[bits(2)]
    _unused: u8,
    #[bits(4)]
    pub freq_idx: u8,

    // Second byte
    #[bits(1)]
    _unused: bool,
    #[bits(7)]
    pub direct_load: u8,

    // Third byte
    #[bits(8)]
    pub sample_addr: u8,

    // Fourth byte
    #[bits(8)]
    pub sample_length: u8,
}

#[bitfield(u8)]
pub struct ApuControl {
    #[bits(3)]
    _unused: u8,
    #[bits(1)]
    pub dmc_enabled: bool,
    #[bits(1)]
    pub noise_counter_enabled: bool,
    #[bits(1)]
    pub triangle_counter_enabled: bool,
    #[bits(1)]
    pub pulse2_counter_enabled: bool,
    #[bits(1)]
    pub pulse1_counter_enabled: bool,
}

#[bitfield(u8)]
pub struct ApuStatus {
    #[bits(1)]
    pub dmc_interrupt: bool,
    #[bits(1)]
    pub frame_interrupt: bool,
    #[bits(1)]
    _unused: bool,
    #[bits(1)]
    pub dmc_active: bool,
    #[bits(1)]
    pub noise_counter_status: bool,
    #[bits(1)]
    pub triangle_counter_status: bool,
    #[bits(1)]
    pub pulse2_counter_status: bool,
    #[bits(1)]
    pub pulse1_counter_status: bool,
}



