use std::time::Instant;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::app::draw::chars::S;

use super::apu_util::{
    DmcChannel, NesChannel, NoiseChannel, PulseChannel, TriangleChannel
};

pub const NES_AUDIO_FREQUENCY: u32 = 44100; // 44.1 KiHz
pub const CPU_FREQ: f64 = 1_789_773f64; // For NTSC systems
pub const CPU_CYCLE_PERIOD: f64 = 1.0 / CPU_FREQ;

const SAMPLE_PERIOD: f64 = 1.0 / NES_AUDIO_FREQUENCY as f64;
const SAMPLE_BATCH_SIZE: usize = 2048;
// The number of clocks in each denomination of a frame (in CPU clocks)
const QUARTER_FRAME_CLOCKS: usize = 3729;
const HALF_FRAME_CLOCKS: usize = 7457;
const THREE_QUARTER_FRAME_CLOCKS: usize = 11185;
const WHOLE_FRAME_CLOCKS: usize = 14916;

pub struct Apu2A03 {
    sample_queue: Arc<Mutex<VecDeque<f32>>>,
    sample_batch: Vec<f32>,
    
    clocks: u64,
    frame_clocks: usize,
    clocks_since_sampled: usize,

    pulse1_channel: PulseChannel,
    pulse2_channel: PulseChannel,
    triangle_channel: TriangleChannel,
    noise_channel: NoiseChannel,
    dmc_channel: DmcChannel,

    frame_sequence: bool,
    disable_frame_interrupt: bool,

    frame_update_counter: usize,
    frame_update_mode1: bool,

    last_output: Instant,
    avg_rates: Vec<f64>,
    batches_sent: usize,

    irq_request_flag: bool,
    trigger_irq: bool,
}

impl Apu2A03 {
    pub fn new(sample_queue: Arc<Mutex<VecDeque<f32>>>) -> Self {
        Self {
            sample_queue,
            sample_batch: Vec::with_capacity(SAMPLE_BATCH_SIZE),

            clocks: 0,
            frame_clocks: 0,
            clocks_since_sampled: 0,

            pulse1_channel: PulseChannel::new(NesChannel::Pulse1),
            pulse2_channel: PulseChannel::new(NesChannel::Pulse2),
            triangle_channel: TriangleChannel::default(),
            noise_channel: NoiseChannel::new(),
            dmc_channel: DmcChannel::new(),

            frame_sequence: false,
            disable_frame_interrupt: false,

            frame_update_counter: 0,
            frame_update_mode1: false,

            last_output: Instant::now(),
            avg_rates: Vec::new(),
            batches_sent: 0,

            irq_request_flag: false,
            trigger_irq: false,
        }
    }

    pub fn cycle(&mut self) {
        self.clocks += 1;
        self.frame_clocks += 1;
        self.clocks_since_sampled += 1;

        // Noise channel updates period every CPU clock
        self.noise_channel.update_period();

        if self.frame_clocks == QUARTER_FRAME_CLOCKS
            || self.frame_clocks == HALF_FRAME_CLOCKS
            || self.frame_clocks == THREE_QUARTER_FRAME_CLOCKS
            || self.frame_clocks == WHOLE_FRAME_CLOCKS {
            
            self.frame_update();
        
            if self.frame_clocks == WHOLE_FRAME_CLOCKS {
                self.frame_clocks = 0;
            }
        }

        let time_since_sampled = self.clocks_since_sampled as f64 * CPU_CYCLE_PERIOD;

        if time_since_sampled > SAMPLE_PERIOD {
            let sample = self.generate_sample();

            self.push_sample(sample);

            self.clocks_since_sampled = 0;
        }
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x4015 => {
                // DMC interrupt (I), frame interrupt (F), DMC active (D), length counter > 0 (N/T/2/1) 
                let d = if self.dmc_channel.enabled { 1 } else { 0 };
                let n = if self.pulse1_channel.length_counter.is_zero() { 0 } else { 1 };
                let t = if self.pulse1_channel.length_counter.is_zero() { 0 } else { 1 };
                let p1 = if self.pulse1_channel.length_counter.is_zero() { 0 } else { 1 };
                let p2 = if self.pulse1_channel.length_counter.is_zero() { 0 } else { 1 };

                let data = (d << 4) | (n << 3) | (t << 2) | (p2 << 1) | (p1 << 0);

                data as u8
            },

            _ => 0
        }
    }

    pub fn cpu_write(&mut self, address: u16, data: u8) {
        match address {
            // Pulse 1 Registers
            0x4000 => {
                let new_duty_cycle = (data >> 6) & 3;
                let new_halt = (data & 0x20) != 0; // Also envelope's loop flag
                let new_const_volume = (data & 0x10) != 0;
                let new_volume = (data & 0x0F) as usize;

                self.pulse1_channel.duty_cycle_percent = match new_duty_cycle {
                    0 => 0.125,
                    1 => 0.25,
                    2 => 0.50,
                    3 => 0.75,
                    _ => { unreachable!("Holy wack unlyrical lyrics, Batman!"); }
                };
                self.pulse1_channel.length_counter.set_halted(new_halt);
                self.pulse1_channel.envelope.set_loop_flag(new_halt);
                self.pulse1_channel.envelope.set_const_volume(new_const_volume);
                self.pulse1_channel.envelope.set_volume(new_volume);
                self.pulse1_channel.envelope.set_start_flag(true);
            }

            // Pulse 1 Sweeper
            0x4001 => {
                let new_enable = (data & 0x80) != 0;
                let new_reload_val = ((data >> 4) & 7) as usize;
                let new_negate = (data & 0x08) != 0;
                let new_shift = (data & 7) as usize;

                self.pulse1_channel.set_sweep_enable(new_enable);
                self.pulse1_channel.set_sweep_period(new_reload_val);
                self.pulse1_channel.set_sweep_negate_flag(new_negate);
                self.pulse1_channel.set_sweep_shift(new_shift);
                self.pulse1_channel.set_sweep_reload_flag(true);
                self.pulse1_channel.update_target_period();
            }

            // Pulse 1 Timer Low
            0x4002 => {
                self.pulse1_channel.set_timer_reload(
                    (self.pulse1_channel.timer_reload() & 0x700) | data as usize
                );

                self.pulse1_channel.envelope.set_start_flag(true);
            }

            // Pulse 1 Timer High & Length Counter
            0x4003 => {
                let new_counter_load = (data >> 3) as usize;
                let new_timer_hi = (data & 0x7) as usize;
                let timer_lo = self.pulse1_channel.timer_reload() & 0xFF;
                let new_timer = (new_timer_hi << 8) | timer_lo;

                self.pulse1_channel.length_counter.set_counter(new_counter_load);
                self.pulse1_channel.set_timer_reload(new_timer);
                self.pulse1_channel.envelope.set_start_flag(true);
            }

            // Pulse 2 Registers
            0x4004 => {
                let new_duty_cycle = (data >> 6) & 3;
                let new_halt = (data & 0x20) != 0;
                let new_const_volume = (data & 0x10) != 0;
                let new_volume = (data & 0x0F) as usize;

                self.pulse2_channel.duty_cycle_percent = match new_duty_cycle {
                    0 => 0.125,
                    1 => 0.25,
                    2 => 0.50,
                    3 => 0.75,
                    _ => { unreachable!("Holy wack unlyrical lyrics, Batman!"); }
                };
                self.pulse2_channel.length_counter.set_halted(new_halt);
                self.pulse2_channel.envelope.set_loop_flag(new_halt);
                self.pulse2_channel.envelope.set_const_volume(new_const_volume);
                self.pulse2_channel.envelope.set_volume(new_volume);
                self.pulse2_channel.envelope.set_start_flag(true);
            }

            // Pulse 2 Sweeper
            0x4005 => {
                let new_enable = (data & 0x80) != 0;
                let new_reload_val = ((data >> 4) & 7) as usize;
                let new_negate = (data & 0x08) != 0;
                let new_shift = (data & 7) as usize;

                self.pulse2_channel.set_sweep_enable(new_enable);
                self.pulse2_channel.set_sweep_period(new_reload_val);
                self.pulse2_channel.set_sweep_negate_flag(new_negate);
                self.pulse2_channel.set_sweep_shift(new_shift);
                self.pulse2_channel.set_sweep_reload_flag(true);
                self.pulse2_channel.update_target_period();
            }

            // Pulse 2 Timer Low
            0x4006 => {
                self.pulse2_channel.set_timer_reload(
                    (self.pulse2_channel.timer_reload() & 0x700) | data as usize
                );

                self.pulse2_channel.envelope.set_start_flag(true);
            }

            // Pulse 2 Timer High & Length Counter
            0x4007 => {
                let new_counter_load = (data >> 3) as usize;
                let new_timer_hi = (data & 0x7) as usize;
                let timer_lo = self.pulse2_channel.timer_reload() & 0xFF;
                let new_timer = (new_timer_hi << 8) | timer_lo;

                self.pulse2_channel.length_counter.set_counter(new_counter_load);
                self.pulse2_channel.set_timer_reload(new_timer);
                self.pulse2_channel.envelope.set_start_flag(true);
            }

            // Triangle Linear counter
            0x4008 => {
                let new_control = (data & 0x80) != 0;
                let new_reload = (data & 0x7F) as usize;

                self.triangle_channel.length_counter.set_halted(new_control);
                self.triangle_channel.linear_counter.set_control_flag(new_control);
                self.triangle_channel.linear_counter.set_reload_value(new_reload);
            }

            // Triangle Timer Low
            0x400A => {
                self.triangle_channel.set_timer_reload(
                    (self.triangle_channel.timer_reload & 0x700) | data as usize
                );
            }

            // Triangle Length counter & Timer High
            0x400B => {
                let new_timer_hi = (data & 0x7) as usize;
                let timer_lo = self.triangle_channel.timer_reload & 0xFF;
                let new_timer = (new_timer_hi << 8) | timer_lo;
                
                self.triangle_channel.set_timer_reload(new_timer);

                self.triangle_channel.length_counter.set_counter((data >> 3) as usize);
                self.triangle_channel.linear_counter.set_reload_flag(true);
            }

            // Noise Length Counter & Volume Envelope
            0x400C => {
                let new_halt = (data & 0x20) != 0;
                let new_const_volume = (data & 0x10) != 0;
                let new_volume = (data & 0x0F) as usize;

                // new_enable is also envelope_loop, idk if it needs to be flipped or no

                self.noise_channel.length_counter.set_halted(new_halt);
                self.noise_channel.envelope.set_loop_flag(new_halt);
                self.noise_channel.envelope.set_const_volume(new_const_volume);
                self.noise_channel.envelope.set_volume(new_volume);
            }

            // Noise Channel Mode & Period
            0x400E => {
                let new_mode = (data & 0x80) != 0;
                let new_period = data & 0x0F;

                self.noise_channel.mode = new_mode;
                self.noise_channel.set_period(new_period);
            }

            // Noise Channel Length counter
            0x400F => {
                let new_counter_load = (data >> 3) as usize;

                self.noise_channel.length_counter.set_counter(new_counter_load);
                self.noise_channel.envelope.set_start_flag(true);
            }

            0x4011 => {
                self.dmc_channel.output = data & 0x7F;
            }

            // Channel enable register
            0x4015 => {
                let pulse1_enabled = (data & 0x01) != 0;
                let pulse2_enabled = (data & 0x02) != 0;
                let triangle_enabled = (data & 0x04) != 0;
                let noise_enabled = (data & 0x08) != 0;
                let dmc_enabled = (data & 0x10) != 0;

                self.pulse1_channel.set_enable(pulse1_enabled);
                self.pulse2_channel.set_enable(pulse2_enabled);
                self.triangle_channel.set_enable(triangle_enabled);
                self.noise_channel.set_enable(noise_enabled);
                self.dmc_channel.set_enable(dmc_enabled);
            }

            // Frame update mode & frame interrupt register
            0x4017 => {
                let new_mode1 = data & 0x80 != 0;
                let new_irq_flag = data & 0x40 == 0;

                self.frame_update_mode1 = new_mode1;
                self.frame_update_counter = 0;
                if new_mode1 {
                    self.frame_update()
                }
                self.irq_request_flag = new_irq_flag;
            }

            _ => {}
        }
    }

    fn generate_sample(&mut self) -> f32 {
        let pulse1_sample = self.pulse1_channel.sample(self.clocks);
        let pulse2_sample = self.pulse2_channel.sample(self.clocks);
        let triangle_sample = self.triangle_channel.sample(self.clocks);
        let noise_sample = self.noise_channel.sample();
        let dmc_sample = self.dmc_channel.sample();

        // There are a lot of magic numbers in this calculation. They are pulled 
        // from the nesdev wiki formulas on this page:
        // https://www.nesdev.org/wiki/APU_Mixer
        // The magic numbers here are different because the formulas have been
        // rearranged to minimize division operations. Essensially the main trick
        // is to take a fraction like 1 / (1/a + 100) and rearrange it to the form
        // a / (1 + 100a). This trick works on both the pulse_out and tnd_out
        // formulas, and result in the following equations.
        const ABRA: f32 = 1.0 / 8227.0;
        const KADABRA: f32 = 1.0 / 12241.0;
        const ALAKAZAM: f32 = 1.0 / 22638.0;

        const BIPPITY: f32 = 95.88;
        const BOPPITY: f32 = 159.79;
        const BOO: f32 = 8128.0;

        let pulse_sum = pulse1_sample + pulse2_sample;
        let pulse_out = (BIPPITY * pulse_sum) / (BOO + 100.0 * pulse_sum);

        let magic_sample = ABRA * triangle_sample + KADABRA * noise_sample;// + ALAKAZAM * dmc_sample;

        let tnd_out = BOPPITY * magic_sample / (1.0 + 100.0 * magic_sample);

        let output = pulse_out + tnd_out;

        // Output is on a scale from 0.0 to 1.0, so we put it from -1 to 1
        let sample = output * 1.95 - 1.0;

        sample
    }

    fn push_sample(&mut self, sample: f32) {
        self.sample_batch.push(sample);

        if self.sample_batch.len() >= SAMPLE_BATCH_SIZE {
            self.sample_queue.lock().unwrap()
                .extend(self.sample_batch.drain(..));

            self.last_output = Instant::now();
            self.batches_sent += 1;
        }
    }

    pub fn audio_samples_queued(&self) -> usize {
        self.sample_queue.lock().unwrap().len()
    }

    fn frame_update(&mut self) {
        if self.frame_update_mode1 {
            match self.frame_update_counter {
                0 => {
                    self.update_envelopes();
                    self.clock_linear_counters();
                },
                1 => {
                    self.update_envelopes();
                    self.clock_linear_counters();
                    self.update_length_counters();
                    self.update_sweepers();
                },
                2 => {
                    self.update_envelopes();
                    self.clock_linear_counters();
                },
                3 => {},
                4 => {
                    self.update_envelopes();
                    self.clock_linear_counters();
                    self.update_length_counters();
                    self.update_sweepers();
                },
                _ => {},
            }

            self.frame_update_counter = (self.frame_update_counter + 1) % 5;
        } 
        else {
            match self.frame_update_counter {
                0 => {
                    self.update_envelopes();
                    self.clock_linear_counters();
                },
                1 => {
                    self.update_envelopes();
                    self.clock_linear_counters();
                    self.update_length_counters();
                    self.update_sweepers();
                },
                2 => {
                    self.update_envelopes();
                    self.clock_linear_counters();
                },
                3 => {
                    self.update_envelopes();
                    self.clock_linear_counters();
                    self.update_length_counters();
                    self.update_sweepers();

                    self.trigger_irq = self.irq_request_flag;
                },
                _ => {},
            }

            self.frame_update_counter = (self.frame_update_counter + 1) % 4;
        }
    }

    fn clock_linear_counters(&mut self) {
        self.triangle_channel.update_linear_counter();
    }

    fn update_length_counters(&mut self) {
        self.pulse1_channel.update_length_counter();
        self.pulse2_channel.update_length_counter();
        self.triangle_channel.update_length_counter();
        self.noise_channel.update_length_counter();
    }

    fn update_sweepers(&mut self) {
        self.pulse1_channel.update_sweep();
        self.pulse2_channel.update_sweep();
    }

    fn update_envelopes(&mut self) {
        self.pulse1_channel.update_envelope();
        self.pulse2_channel.update_envelope();
        self.noise_channel.update_envelope();
    }

    pub fn trigger_irq(&self) -> bool {
        self.trigger_irq
    }

    pub fn set_trigger_irq(&mut self, val: bool) {
        self.trigger_irq = val;
    }
}
