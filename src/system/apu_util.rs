use std::{collections::{vec_deque, VecDeque}, ops::{Neg, Rem}, sync::{Arc, Mutex}, time::Duration};

use bitfield_struct::bitfield;
use rodio::Source;

use super::apu::{CPU_CYCLE_PERIOD, CPU_FREQ, NES_AUDIO_FREQUENCY};

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


#[derive(Default)]
pub struct LengthCounter {
    halted: bool,
    counter: usize,

    channel_enabled: bool,
    silence_channel: bool,
}

impl LengthCounter {
    /// https://www.nesdev.org/wiki/APU_Length_Counter#Table_structure
    /// Lookup table for the lengths of the notes given a 5 bit number
    const LENGTH_LOOKUP: [usize; 32] = [
        10, 254, 20, 2, 40, 4, 80, 6, 
        160, 8, 60, 10, 14, 12, 26, 14,
        12, 16, 24, 18, 48, 20, 96, 22,
        192, 24, 72, 26, 16, 28, 32, 30
    ];

    pub fn update_count(&mut self) {
        if !self.channel_enabled {
            self.counter = 0;
        } else if !self.halted { 
            if self.counter > 0 {
                self.counter -= 1;
            } else {
                self.silence_channel = true;
            }
        }
    }

    pub fn set_counter(&mut self, val: usize) {
        if self.channel_enabled {
            self.counter = Self::LENGTH_LOOKUP[val];
            self.silence_channel = false;
        }
    }

    pub fn set_halted(&mut self, val: bool) {
        self.halted = val;
    }

    pub fn silence_channel(&self) -> bool {
        self.silence_channel
    }
}

#[derive(Default)]
pub struct LinearCounter {
    control: bool,
    reload_flag: bool,
    reload_value: usize,
    counter: usize,
}

impl LinearCounter {
    pub fn update_count(&mut self) {
        if self.reload_flag {
            self.counter = self.reload_value;
        } else {
            if self.counter > 0 {
                self.counter -= 1;
            }
        }

        if !self.control {
            self.reload_flag = false;
        }
    }

    pub fn set_reload_value(&mut self, val: usize) {
        self.counter = val;
    }

    pub fn set_reload_flag(&mut self, val: bool) {
        self.control = val;
    }

    pub fn silence_channel(&self) -> bool {
        self.counter == 0
    }

    pub fn set_control_flag(&mut self, val: bool) {
        self.control = val;
    }
}

#[derive(Default)]
pub struct VolumeEnvelope {
    start: bool,
    const_volume: bool,
    divider: usize,
    decay: usize,
    volume: usize, // Volume/Period
    loop_flag: bool,
}

// https://www.nesdev.org/wiki/APU_Envelope
impl VolumeEnvelope {
    pub fn update_output(&mut self) {
        if self.start {
            self.decay = 15;
            self.start = false;
            self.divider = self.volume;
        } else {
            if self.divider == 0 {
                self.divider = self.volume;
                
                if self.decay == 0 {
                    if self.loop_flag {
                        self.decay = 15;
                    }
                } else {
                    self.decay -= 1;
                }
            } else {
                self.divider -= 1;
            }
        }
    }

    pub fn output(&self) -> usize {
        if self.const_volume {
            self.volume
        } else {
            self.decay
        }
    }

    pub fn set_start_flag(&mut self, val: bool) {
        self.start = val;
    }

    pub fn set_const_volume(&mut self, val: bool) {
        self.const_volume = val;
    }

    pub fn set_volume(&mut self, val: usize) {
        self.volume = val;
    }

    pub fn set_loop_flag(&mut self, val: bool) {
        self.loop_flag = val;
    }
}

pub struct Sweeper {
    
}

pub struct PulseChannel {
    channel_type: NesChannel,

    // Basic channel registers
    pub timer_reload: usize,

    pub freq: f64,
    pub enabled: bool,
    pub duty_cycle_percent: f64,

    // Sweeper registers
    pub sweeper_divider: usize,
    pub sweeper_divider_period: usize,
    pub sweeper_enabled: bool,
    pub sweeper_negate: bool,
    pub sweeper_shift: usize,
    pub sweeper_reload: bool,
    target_period: usize,
    sweeper_mute: bool,

    pub length_counter: LengthCounter,
    pub envelope: VolumeEnvelope
}

impl PulseChannel {
    pub fn new(channel_type: NesChannel) -> Self {
        Self {
            channel_type,
            timer_reload: 0,
            freq: 0.0,
            enabled: false,
            duty_cycle_percent: 0.0,
            sweeper_divider: 0,
            sweeper_divider_period: 0,
            sweeper_enabled: false,
            sweeper_negate: false,
            sweeper_shift: 0,
            sweeper_reload: false,
            target_period: 0,
            sweeper_mute: false,
            length_counter: LengthCounter::default(),
            envelope: VolumeEnvelope::default(),
        }
    }

    pub fn sample(&mut self, total_clocks: u64) -> f32 {
        let mut sample = 0.0;

        self.sweep_continuous_update();

        if self.enabled && !self.sweeper_mute {
            if !self.length_counter.silence_channel() {
                let time = total_clocks as f64 * CPU_CYCLE_PERIOD;
    
                let remainder = (time * self.freq).fract();
    
                if remainder < self.duty_cycle_percent {
                    sample = 1.0;
                } else {
                    sample = 0.0;
                }
            }
        }

        // The sample is scaled by the envelope to be a value in the range [0.0, 15.0]
        sample * self.envelope.output() as f32
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

    pub fn update_length_counter(&mut self) {
        self.length_counter.update_count()
    }


    pub fn set_timer_reload(&mut self, val: usize) {
        self.timer_reload = val;
        self.freq = CPU_FREQ / (16.0 * (val + 1) as f64);
    }

    pub fn set_enable(&mut self, val: bool) {
        self.enabled = val;

        self.length_counter.channel_enabled = val;
    }
}



#[derive(Default)]
pub struct TriangleChannel {
    // Basic channel registers
    // pub timer: usize,
    pub timer_reload: usize,
    pub freq: f64,
    pub enabled: bool,

    pub length_counter: LengthCounter,
    pub linear_counter: LinearCounter,
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
        // This is not how the NES works, but some games set the triangle timer
        // to 0 to "silence" the channel. This doesn't actually silence it, however,
        // and instead an extremely high frequency wave is produced. At the cost
        // of accuracy, we eliminate these frequencies for the sake of the player's 
        // eardrums :)
        if self.timer_reload <= 2 || !self.enabled {
            return 0.0;
        }

        if !self.linear_counter.silence_channel() && 
           !self.length_counter.silence_channel() {

            let time = total_clocks as f64 * CPU_CYCLE_PERIOD;

            let remainder = (time * self.freq).fract();

            let sequencer_idx = (32.0 * remainder) as usize;

            return TriangleChannel::SEQUENCER_LOOKUP[sequencer_idx];
        }

        0.0
    }

    pub fn update_linear_counter(&mut self) {
        self.linear_counter.update_count();
    }

    pub fn update_length_counter(&mut self) {
        self.length_counter.update_count();
    }

    pub fn set_timer_reload(&mut self, val: usize) {
        self.timer_reload = val;
        self.freq = CPU_FREQ / (32.0 * (val + 1) as f64);
    }

    pub fn set_enable(&mut self, val: bool) {
        self.enabled = val;

        self.length_counter.channel_enabled = val;
    }
}



pub struct NoiseChannel {
    // Basic channel registers
    rand_shifter: u16,

    pub period_reload: usize,
    pub period: usize,

    pub enabled: bool,
    pub mode: bool,

    pub length_counter: LengthCounter,
    pub envelope: VolumeEnvelope,
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
            length_counter: LengthCounter::default(),
            envelope: VolumeEnvelope::default(),
        }
    }

    pub fn sample(&mut self) -> f32 {
        let mut sample = 0.0;

        if !self.length_counter.silence_channel() {
            if self.rand_shifter & 1 == 0 {
                // The sample is in the range [0.0, 15.0]
                // Just outputs the envelope volume or 0
                sample = self.envelope.output() as f32;
            }
        }

        sample
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

    pub fn set_period(&mut self, data: u8) {
        self.period_reload = Self::PERIOD_LOOKUP[data as usize];
        self.period = self.period_reload;
    }

    pub fn set_enable(&mut self, val: bool) {
        self.enabled = val;

        self.length_counter.channel_enabled = val;
    }
}



#[derive(Default)]
pub struct DmcChannel {
    // Basic channel registers
    pub timer: usize,
    pub timer_reload: usize,
    pub enabled: bool,

    pub length_counter: LengthCounter,

    pub bytes_remaining: usize,
    pub sample_len: usize,
    pub sample_addr: u16,

    pub bits_remaining: usize,
    pub dmc_shifter: u8,
    pub silenced: bool,

    pub output: u8,
}

impl DmcChannel {
    const RATE_LOOKUP: [usize; 16] = [
        428, 380, 340, 320, 286, 254, 226, 214, 
        190, 160, 142, 128, 106,  84,  72,  54
    ];

    pub fn new() -> Self {
        Self::default()
    }

    pub fn sample(&mut self) -> f32 {
        self.output as f32
    }

    pub fn set_timer_reload(&mut self, data: u8) {
        self.timer_reload = DmcChannel::RATE_LOOKUP[data as usize];
        self.timer = self.timer_reload;
    } 

    pub fn update_timer(&mut self) -> bool {
        let mut need_new_sample_byte = false;

        if self.timer == 0 {
            if self.dmc_shifter & 1 == 0 {
                self.output = self.output.wrapping_sub(1);
            } else {
                self.output = self.output.wrapping_add(1);
            }

            self.dmc_shifter >>= 1;
            self.bits_remaining -= 1;

            if self.bits_remaining == 0 {
                need_new_sample_byte = true;
            }
        } else {
            self.timer -= 1;
        }

        need_new_sample_byte
    }
}
