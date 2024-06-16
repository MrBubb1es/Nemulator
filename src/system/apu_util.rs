use std::{collections::{vec_deque, VecDeque}, ops::{Neg, Rem}, sync::{Arc, Mutex}, time::Duration};

use bitfield_struct::bitfield;
use rodio::Source;

use super::apu::{CPU_CYCLE_PERIOD, CPU_FREQ, NES_AUDIO_FREQUENCY};

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
    // Using an arc mutex vecdeque allows us to directly queue up samples within
    // this source. I would prefer to use rodio's SourcesInputQueue, but there
    // are weird popping artifacts in between sources if we do it that way.
    // Appending samples directly to a single source gets rid of this popping as
    // long as we always have samples in the queue.
    sample_queue: Arc<Mutex<VecDeque<f32>>>,
}

impl Iterator for NesAudioStream {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let mut sample_queue = self.sample_queue.lock().unwrap();

        // Always return some so the source is never destroyed, even if it gets ahead.
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
}


pub enum NesChannel {
    Pulse1,
    Pulse2,
    Triangle,
    Noise,
    DMC,
}

// #[derive(Default)]
// pub struct PulseChannel {
//     // Basic channel registers
//     pub timer: usize,

//     pub freq: f64,
//     pub enabled: bool,
//     pub duty_cycle_percent: f64,

//     pub length_counter: usize,
//     pub counter_enabled: bool,

//     pub constant_volume: bool,

//     // Sweeper registers
//     pub sweeper_divider: usize,
//     pub sweeper_divider_period: usize,
//     pub sweeper_enabled: bool,
//     pub sweeper_negate: bool,
//     pub sweeper_shift: usize,
//     pub sweeper_reload: bool,

//     // Envelope registers
//     pub envelope_start: bool,
//     pub envelope_divider: usize,
//     pub envelope_decay: usize,
//     pub envelope_volume: usize,
//     pub envelope_loop: bool,
// }

// impl PulseChannel {
//     pub fn sample(&mut self, total_clocks: u64) -> f32 {
//         let mut sample = 0.0;

//         if self.enabled && self.timer > 8 {
//             if self.length_counter > 0 || !self.counter_enabled {
//                 let time = total_clocks as f64 * CPU_CYCLE_PERIOD;
    
//                 let remainder = (time * self.freq).fract();
    
//                 if remainder < self.duty_cycle_percent {
//                     sample = 1.0;
//                 } else {
//                     sample = 0.0;
//                 }
//             }
//         }

//         // The sample is scaled to be a value in the range [0.0, 15.0]
//         sample * if self.constant_volume { 
//             self.envelope_volume as f32
//         } else {
//             self.envelope_decay as f32
//         }
//     }

//     // https://www.nesdev.org/wiki/APU_Sweep
//     pub fn sweep_update(&mut self, channel_type: NesChannel) {
//         if self.sweeper_divider == 0 && self.sweeper_enabled && self.sweeper_shift != 0 {
//             let change_amount = self.timer >> self.sweeper_shift;

//             let target_period = match channel_type {
//                 NesChannel::Pulse1 => {
//                     if self.sweeper_negate {
//                         ((self.timer as isize) - (change_amount as isize + 1)).max(0) as usize
//                     } else {
//                         self.timer + change_amount
//                     }
//                 },
//                 NesChannel::Pulse2 => {
//                     if self.sweeper_negate {
//                         ((self.timer as isize) - (change_amount as isize)).max(0) as usize
//                     } else {
//                         self.timer + change_amount
//                     }
//                 },
//                 _ => {panic!("Only pulse 1 and pulse 2 channels should be sweeping in the pulse channel struct")}
//             };

//             if self.timer < 8 || 0x7FF < target_period {
//                 self.enabled = false;
//             } else {
//                 self.set_timer_reload(target_period);
//             }
//         }

//         if self.sweeper_divider == 0 || self.sweeper_reload {
//             self.sweeper_divider = self.sweeper_divider_period;
//             self.sweeper_reload = false;
//         } else {
//             self.sweeper_divider -= 1;
//         }
//     }

//     // https://www.nesdev.org/wiki/APU_Envelope
//     pub fn update_envelope(&mut self) {
//         if self.envelope_start {
//             self.envelope_start = false;
//             self.envelope_decay = 0xF;
//             self.envelope_divider = self.envelope_volume;
//         } else {
//             if self.envelope_divider == 0 {
//                 self.envelope_divider = self.envelope_volume;

//                 if self.envelope_decay != 0 {
//                     self.envelope_decay -= 1;
//                 } else {
//                     if self.envelope_loop {
//                         self.envelope_decay = 0xF;
//                     }
//                 }
//             } else {
//                 self.envelope_divider -= 1;
//             }
//         }
//     }

//     pub fn set_timer_reload(&mut self, val: usize) {
//         self.timer = val;
//         self.freq = CPU_FREQ / (16.0 * (val + 1) as f64);
//     }

//     pub fn set_length_counter(&mut self, data: u8) {
//         self.length_counter = PULSE_COUNTER_LOOKUP[data as usize];
//     }

//     pub fn update_counter(&mut self) {
//         if !self.enabled {
//             self.length_counter = 0;
//         } else if self.length_counter > 0 && self.counter_enabled {
//             self.length_counter -= 1;
//         }
//     }
// }

pub struct PulseChannel {
    channel_type: NesChannel,

    // Basic channel registers
    pub timer: usize,

    pub freq: f64,
    pub enabled: bool,
    pub duty_cycle_percent: f64,

    pub length_counter: usize,
    pub counter_enabled: bool,

    pub constant_volume: bool,

    // Sweeper registers
    pub sweeper_divider: usize,
    pub sweeper_divider_period: usize,
    pub sweeper_enabled: bool,
    pub sweeper_negate: bool,
    pub sweeper_shift: usize,
    pub sweeper_reload: bool,
    target_period: usize,
    sweeper_mute: bool,

    // Envelope registers
    pub envelope_start: bool,
    pub envelope_divider: usize,
    pub envelope_decay: usize,
    pub envelope_volume: usize,
    pub envelope_loop: bool,
}

impl PulseChannel {
    pub fn new(channel_type: NesChannel) -> Self {
        Self {
            channel_type,
            timer: 0,
            freq: 0.0,
            enabled: false,
            duty_cycle_percent: 0.0,
            length_counter: 0,
            counter_enabled: false,
            constant_volume: false,
            sweeper_divider: 0,
            sweeper_divider_period: 0,
            sweeper_enabled: false,
            sweeper_negate: false,
            sweeper_shift: 0,
            sweeper_reload: false,
            target_period: 0,
            sweeper_mute: false,
            envelope_start: false,
            envelope_divider: 0,
            envelope_decay: 0,
            envelope_volume: 0,
            envelope_loop: false,
        }
    }

    pub fn sample(&mut self, total_clocks: u64) -> f32 {
        let mut sample = 0.0;

        self.sweep_continuous_update();

        if self.enabled && !self.sweeper_mute {
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

        // The sample is scaled to be a value in the range [0.0, 15.0]
        sample * if self.constant_volume {
            self.envelope_volume as f32
        } else {
            self.envelope_decay as f32
        }
    }

    pub fn sweep_continuous_update(&mut self) {
        let change_amount = self.timer >> self.sweeper_shift;

        self.target_period = match self.channel_type {
            NesChannel::Pulse1 => {
                if self.sweeper_negate {
                    ((self.timer as isize) - (change_amount as isize + 1)).max(0) as usize
                } else {
                    self.timer + change_amount
                }
            },
            NesChannel::Pulse2 => {
                if self.sweeper_negate {
                    ((self.timer as isize) - (change_amount as isize)).max(0) as usize
                } else {
                    self.timer + change_amount
                }
            },
            _ => {panic!("Only pulse 1 and pulse 2 channels should be sweeping in the pulse channel struct")}
        };

        if self.timer < 8 || self.target_period > 0x7FF {
            self.sweeper_mute = true;
        } else {
            self.sweeper_mute = false;
        }
    }

    // https://www.nesdev.org/wiki/APU_Sweep
    pub fn sweep_frame_update(&mut self) {
        self.sweep_continuous_update();

        if self.sweeper_divider == 0 && self.sweeper_enabled && self.sweeper_shift != 0 && !self.sweeper_mute {
            self.set_timer_reload(self.target_period);
        } else {
            if self.sweeper_divider == 0 || self.sweeper_reload {
                self.sweeper_divider = self.sweeper_divider_period;
                self.sweeper_reload = false;
            } else {
                self.sweeper_divider -= 1;
            }
        }
    }

    // https://www.nesdev.org/wiki/APU_Envelope
    pub fn update_envelope(&mut self) {
        if self.envelope_start {
            self.envelope_start = false;
            self.envelope_decay = 0xF;
            self.envelope_divider = self.envelope_volume;
        } else {
            if self.envelope_divider == 0 {
                self.envelope_divider = self.envelope_volume;

                if self.envelope_decay != 0 {
                    self.envelope_decay -= 1;
                } else {
                    if self.envelope_loop {
                        self.envelope_decay = 0xF;
                    }
                }
            } else {
                self.envelope_divider -= 1;
            }
        }
    }

    pub fn set_timer_reload(&mut self, val: usize) {
        self.timer = val;
        self.freq = CPU_FREQ / (16.0 * (val + 1) as f64);

        match self.channel_type {
            NesChannel::Pulse1 => {
                println!("New Timer & Freq: {}, {}", self.timer, self.freq);
            }
            _ => {}
        }
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



