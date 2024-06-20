use std::{collections::VecDeque, sync::{Arc, Mutex}, time::Duration};

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

#[derive(Default)]
pub enum NesChannel {
    #[default]
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
        } else {
            if !self.halted { 
                if self.counter > 0 {
                    self.counter -= 1;
                }
            }
        }

        self.silence_channel = self.is_zero();
    }

    pub fn set_channel_enabled(&mut self, val: bool) {
        self.channel_enabled = val;
        if !self.channel_enabled {
            self.counter = 0;
        }
    }

    pub fn set_counter(&mut self, data: usize) {
        if self.channel_enabled {
            self.counter = Self::LENGTH_LOOKUP[data];
        }
    }

    pub fn set_halted(&mut self, val: bool) {
        self.halted = val;
    }

    pub fn is_silencing_channel(&self) -> bool {
        self.silence_channel
    }

    pub fn is_zero(&self) -> bool {
        self.counter == 0
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

    pub fn set_reload_value(&mut self, data: usize) {
        self.reload_value = data;
    }

    pub fn set_reload_flag(&mut self, val: bool) {
        self.reload_flag = val;
    }

    pub fn is_silencing_channel(&self) -> bool {
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

    pub fn set_volume(&mut self, data: usize) {
        self.volume = data;
    }

    pub fn set_loop_flag(&mut self, val: bool) {
        self.loop_flag = val;
    }
}


#[derive(Default)]
pub struct PulseChannel {
    sweep_negate_offset: isize,

    // Basic channel registers
    timer_reload: usize,

    enabled: bool,
    pub freq: f64,
    pub duty_cycle_percent: f64,

    // Sweeper registers
    sweep_enabled: bool,
    sweep_negate: bool,
    sweep_divider: usize,
    sweep_reload_flag: bool,
    sweep_reload_value: usize,
    sweep_shift: usize,
    sweep_target_period: usize,

    pub length_counter: LengthCounter,
    pub envelope: VolumeEnvelope,
}

impl PulseChannel {
    pub fn new(channel_type: NesChannel) -> Self {
        // Pulse channels 1 and 2 handle negation differently
        let sweep_negate_offset = match channel_type {
            NesChannel::Pulse1 => -1,
            NesChannel::Pulse2 =>  0,
            _ => { unreachable!("Only Pulse 1 & Pulse 2 should have a sweep unit"); }
        };

        Self {
            sweep_negate_offset,
            ..Default::default()
        }
    }

    pub fn sample(&mut self, total_clocks: u64) -> f32 {
        let mut sample = 0.0;

        if self.enabled {
            if !self.length_counter.is_silencing_channel() &&
               !self.sweep_is_muting_channel() {

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

    // https://www.nesdev.org/wiki/APU_Sweep
    pub fn update_sweep(&mut self) {
        if self.sweep_divider == 0 && self.sweep_enabled && self.sweep_shift > 0 {
            if !self.sweep_is_muting_channel() {
                self.set_timer_reload(self.sweep_target_period);
            }
        }

        if self.sweep_divider == 0 || self.sweep_reload_flag {
            self.sweep_divider = self.sweep_reload_value;
            self.sweep_reload_flag = false;
        } else {
            self.sweep_divider -= 1;
        }
    }

    pub fn update_target_period(&mut self) {
        let mut delta = (self.timer_reload as isize) >> self.sweep_shift;

        if self.sweep_negate {
            delta = -delta - self.sweep_negate_offset;
        }

        self.sweep_target_period = (self.timer_reload as isize + delta).max(0) as usize;
    }

    pub fn update_length_counter(&mut self) {
        self.length_counter.update_count();
    }

    pub fn update_envelope(&mut self) {
        self.envelope.update_output();
    }

    fn sweep_is_muting_channel(&self) -> bool {
        self.sweep_target_period > 0x7FF || self.timer_reload < 8
    }

    pub fn timer_reload(&self) -> usize {
        self.timer_reload
    }

    pub fn set_timer_reload(&mut self, data: usize) {
        self.timer_reload = data;
        self.freq = CPU_FREQ / (16.0 * (data + 1) as f64);

        self.update_target_period();
    }

    pub fn set_enable(&mut self, val: bool) {
        self.enabled = val;
        self.length_counter.set_channel_enabled(val);
    }

    pub fn set_sweep_enable(&mut self, val: bool) {
        self.sweep_enabled = val;
    }

    pub fn set_sweep_period(&mut self, data: usize) {
        self.sweep_reload_value = data;
    }

    pub fn set_sweep_shift(&mut self, data: usize) {
        self.sweep_shift = data;
    }

    pub fn set_sweep_negate_flag(&mut self, val: bool) {
        self.sweep_negate = val;
    }

    pub fn set_sweep_reload_flag(&mut self, val: bool) {
        self.sweep_reload_flag = val;
    }

}


#[derive(Default)]
pub struct TriangleChannel {
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

        if !self.linear_counter.is_silencing_channel() && 
           !self.length_counter.is_silencing_channel() {

            let time = total_clocks as f64 * CPU_CYCLE_PERIOD;

            let remainder = (time * self.freq).fract();

            let sequencer_idx = (32.0 * remainder) as usize;

            return Self::SEQUENCER_LOOKUP[sequencer_idx];
        }

        0.0
    }

    pub fn update_linear_counter(&mut self) {
        self.linear_counter.update_count();
    }

    pub fn update_length_counter(&mut self) {
        self.length_counter.update_count();
    }

    pub fn set_timer_reload(&mut self, data: usize) {
        self.timer_reload = data;
        self.freq = CPU_FREQ / (32.0 * (data + 1) as f64);
    }

    pub fn set_enable(&mut self, val: bool) {
        self.enabled = val;
        self.length_counter.set_channel_enabled(val);
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

        if !self.length_counter.is_silencing_channel() {
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

    pub fn update_length_counter(&mut self) {
        self.length_counter.update_count();
    }

    pub fn update_envelope(&mut self) {
        self.envelope.update_output();
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
        self.length_counter.set_channel_enabled(val);
    }
}


#[derive(Default)]
pub struct DmcChannel {
    divider: usize,
    divider_reload_value: usize,
    enabled: bool,
    loop_flag: bool,

    irq_enabled: bool,
    irq_requested: bool,

    // 1 byte buffer
    next_byte: u8,
    need_next_byte: bool,

    bytes_remaining: usize,
    sample_len: usize,
    sample_start_addr: u16,
    current_addr: u16,

    bits_remaining: usize,
    dmc_shifter: u8,
    silenced: bool,

    output: u8,
}

impl DmcChannel {
    const RATE_LOOKUP: [usize; 16] = [
        428, 380, 340, 320, 286, 254, 226, 214, 
        190, 160, 142, 128, 106,  84,  72,  54
    ];

    pub fn new() -> Self {
        Self {
            sample_start_addr: 0xC000,
            current_addr: 0xC000,
            ..Default::default()
        }
    }

    pub fn sample(&mut self) -> f32 {
        self.output as f32
    }

    pub fn update_timer(&mut self, next_clip_byte: Option<u8>) {
        if self.need_next_byte && self.bytes_remaining > 0 {
            self.next_byte = next_clip_byte.unwrap();
            self.need_next_byte = false;

            self.current_addr = self.current_addr.checked_add(1).unwrap_or(0x8000); // Address wraps to $8000 (start of cart rom)
        
            self.bytes_remaining -= 1;

            if self.bytes_remaining == 0 {
                if self.loop_flag {
                    self.start_sample();
                } else if self.irq_enabled {
                    self.irq_requested = true;
                }
            }
        }

        if self.divider == 0 {
            self.divider = self.divider_reload_value;

            if self.bits_remaining == 0 {
                self.bits_remaining = 8;
                // Silence channel if next byte hasn't been loaded
                self.silenced = self.need_next_byte;

                if !self.need_next_byte {
                    self.dmc_shifter = self.next_byte;
                    self.need_next_byte = true;
                }
            }

            if !self.silenced {
                let delta = if self.dmc_shifter & 1 == 0 {
                    -2
                } else {
                    2
                };

                let new_output = self.output as isize + delta;

                if 0 <= new_output && new_output <= 127 {
                    self.output = new_output as u8;
                }
            }

            self.dmc_shifter >>= 1;
            self.bits_remaining -= 1;
        } else {
            self.divider -= 1;
        }
    }

    fn start_sample(&mut self) {
        self.current_addr = self.sample_start_addr;
        self.bytes_remaining = self.sample_len;
    }

    pub fn need_next_clip_byte(&self) -> bool {
        self.need_next_byte
    }

    pub fn current_sample_addr(&self) -> u16 {
        self.current_addr
    }

    pub fn dmc_active(&self) -> bool {
        self.bytes_remaining > 0
    }

    pub fn irq_triggered(&self) -> bool {
        self.irq_requested
    }

    pub fn set_irq_flag(&mut self, val: bool) {
        self.irq_requested = val;
    }

    pub fn set_enable(&mut self, val: bool) {
        self.enabled = val;

        if self.enabled {
            if self.bytes_remaining == 0 {
                self.start_sample();
            }
        } else {
            self.bytes_remaining = 0;
        }
        
        self.irq_enabled = false;
    }

    pub fn set_irq_enable(&mut self, val: bool) {
        self.irq_enabled = val;
    }

    pub fn set_loop_flag(&mut self, val: bool) {
        self.loop_flag = val;
    }

    pub fn set_reload_value(&mut self, data: usize) {
        self.divider_reload_value = Self::RATE_LOOKUP[data as usize];
    }

    pub fn set_output_direct(&mut self, data: u8) {
        self.output = data;
    }

    pub fn set_clip_address(&mut self, addr: u16) {
        self.sample_start_addr = addr;
    }

    pub fn set_clip_length(&mut self, data: usize) {
        self.sample_len = data;
    }
}
