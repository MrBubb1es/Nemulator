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

pub struct PulseChannel {
    channel_type: NesChannel,

    // Basic channel registers
    pub timer_reload: usize,

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
            timer_reload: 0,
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
        let change_amount = self.timer_reload >> self.sweeper_shift;

        self.target_period = match self.channel_type {
            NesChannel::Pulse1 => {
                if self.sweeper_negate {
                    ((self.timer_reload as isize) - (change_amount as isize + 1)).max(0) as usize
                } else {
                    self.timer_reload + change_amount
                }
            },
            NesChannel::Pulse2 => {
                if self.sweeper_negate {
                    ((self.timer_reload as isize) - (change_amount as isize)).max(0) as usize
                } else {
                    self.timer_reload + change_amount
                }
            },
            _ => {panic!("Only pulse 1 and pulse 2 channels should be sweeping in the pulse channel struct")}
        };

        if self.timer_reload < 8 || self.target_period > 0x7FF {
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
        self.timer_reload = val;
        self.freq = CPU_FREQ / (16.0 * (val + 1) as f64);
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



#[derive(Default)]
pub struct TriangleChannel {
    // Basic channel registers
    // pub timer: usize,
    pub timer_reload: usize,
    pub freq: f64,
    pub enabled: bool,

    pub length_counter: usize,
    pub counter_enabled: bool,

    pub linear_counter: usize,
    pub linear_reload: usize,
    pub linear_loop: bool,
    pub linear_control: bool,
}

impl TriangleChannel {
    // https://www.nesdev.org/wiki/APU_Triangle
    const SEQUENCER_LOOKUP: [f32; 32] = [
        15.0, 14.0, 13.0, 12.0, 11.0, 10.0,  9.0,  8.0, 
         7.0,  6.0,  5.0,  4.0,  3.0,  2.0,  1.0,  0.0,
         0.0,  1.0,  2.0,  3.0,  4.0,  5.0,  6.0,  7.0, 
         8.0,  9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
    ];

    pub fn new() -> Self {
        Self::default()
    }

    pub fn sample(&mut self, total_clocks: u64) -> f32 {
        if self.linear_counter > 0 {
            if self.length_counter > 0 || !self.counter_enabled {
                let time = total_clocks as f64 * CPU_CYCLE_PERIOD;
    
                let remainder = (time * self.freq).fract();

                let sequencer_idx = (32.0 * remainder) as usize;

                return TriangleChannel::SEQUENCER_LOOKUP[sequencer_idx];
            }
        }

        0.0
    }

    pub fn set_timer_reload(&mut self, val: usize) {
        self.timer_reload = val;
        self.freq = CPU_FREQ / (32.0 * (val + 1) as f64);
    }

    pub fn set_length_counter(&mut self, data: u8) {
        self.length_counter = PULSE_COUNTER_LOOKUP[data as usize];
    }

    pub fn update_length_counter(&mut self) {
        if !self.enabled {
            self.length_counter = 0;
        } else if self.length_counter > 0 && self.counter_enabled {
            self.length_counter -= 1;
        }
    }

    pub fn update_linear_counter(&mut self) {
        if !self.linear_loop {
            self.linear_counter = self.linear_reload;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }

        if !self.linear_control {
            self.linear_loop = false;
        }
    }
}



pub struct NoiseChannel {
    // Basic channel registers
    rand_shifter: u16,

    pub period_reload: usize,
    pub period: usize,

    pub enabled: bool,
    pub mode: bool,

    pub length_counter: usize,
    pub counter_enabled: bool,

    pub constant_volume: bool,

    // Envelope registers
    pub envelope_start: bool,
    pub envelope_divider: usize,
    pub envelope_decay: usize,
    pub envelope_volume: usize,
    pub envelope_loop: bool,
}

impl NoiseChannel {
    const PERIOD_LOOKUP: [usize; 16] = [
        4, 8, 16, 32, 64, 96, 128, 160, 202,
        254, 380, 508, 762, 1016, 2034, 4068
    ];

    pub fn new() -> Self {
        Self {
            rand_shifter: 1,
            period_reload: 0,
            period: 0,
            enabled: false,
            mode: false,
            length_counter: 0,
            counter_enabled: false,
            constant_volume: false,
            envelope_start: false,
            envelope_divider: 0,
            envelope_decay: 0,
            envelope_volume: 0,
            envelope_loop: false,
        }
    }

    pub fn sample(&mut self) -> f32 {
        if self.length_counter > 0 || !self.counter_enabled {
            if self.rand_shifter & 1 == 0 {
                // The sample is in the range [0.0, 15.0]
                if self.constant_volume {
                    return self.envelope_volume as f32;
                } else {
                    return self.envelope_decay as f32;
                }
            }
        }

        0.0
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

    fn update_shifter(&mut self) {
        let first_bit = self.rand_shifter & 1;
        let second_bit = (self.rand_shifter >> if self.mode { 6 } else { 1 }) & 1;

        let new_bit = first_bit ^ second_bit;

        self.rand_shifter >>= 1;
        self.rand_shifter |= new_bit << 14;
    }

    pub fn update_period(&mut self) {
        if self.period == 0 {
            self.period = self.period_reload;

            self.update_shifter();
        } else {
            self.period -= 1;
        }
    }

    pub fn set_length_counter(&mut self, data: u8) {
        self.length_counter = PULSE_COUNTER_LOOKUP[data as usize];
    }

    pub fn set_period(&mut self, data: u8) {
        self.period_reload = NoiseChannel::PERIOD_LOOKUP[data as usize];
        self.period = self.period_reload;
    }

    pub fn update_counter(&mut self) {
        if !self.enabled {
            self.length_counter = 0;
        } else if self.length_counter > 0 && self.counter_enabled {
            self.length_counter -= 1;
        }
    }
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



