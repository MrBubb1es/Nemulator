use std::time::Instant;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::apu_util::{
    ApuControl, ApuStatus, DmcRegisters, NoiseRegisters, 
    PulseChannel, PulseRegisters, TriangleRegisters
};

pub const NES_AUDIO_FREQUENCY: u32 = 44100; // 44.1 KiHz
pub const CPU_CYCLE_PERIOD: f64 = 1.0 / CPU_FREQ;

const SAMPLE_PERIOD: f64 = 1.0 / NES_AUDIO_FREQUENCY as f64;
const CPU_FREQ: f64 = 1_789_773f64; // For NTSC systems
const SAMPLE_BATCH_SIZE: usize = 2048;
// The number of clocks in each denomination of a frame (in CPU clocks)
const QUARTER_FRAME_CLOCKS: usize = 3729;
const HALF_FRAME_CLOCKS: usize = 7457;
const THREE_QUARTER_FRAME_CLOCKS: usize = 7457;
const WHOLE_FRAME_CLOCKS: usize = 14916;

pub struct Apu2A03 {
    sample_queue: Arc<Mutex<VecDeque<f32>>>,
    sample_batch: Vec<f32>,
    
    clocks: u64,
    frame_clocks: usize,
    clocks_since_sampled: usize,

    triangle_regs: TriangleRegisters,
    noise_regs: NoiseRegisters,
    dmc_regs: DmcRegisters,

    pulse1_channel: PulseChannel,
    pulse2_channel: PulseChannel,

    control: ApuControl,
    status: ApuStatus,

    frame_sequence: bool,
    disable_frame_interrupt: bool,

    frame_update_counter: usize,
    frame_update_mode1: bool,

    last_output: Instant,
    avg_rates: Vec<f64>,
    batches_sent: usize,
}

impl Apu2A03 {
    pub fn new(sample_queue: Arc<Mutex<VecDeque<f32>>>) -> Self {
        Self {
            sample_queue,
            sample_batch: Vec::with_capacity(SAMPLE_BATCH_SIZE),

            clocks: 0,
            frame_clocks: 0,
            clocks_since_sampled: 0,

            triangle_regs: TriangleRegisters::default(),
            noise_regs: NoiseRegisters::default(),
            dmc_regs: DmcRegisters::default(),

            pulse1_channel: PulseChannel::default(),
            pulse2_channel: PulseChannel::default(),

            control: ApuControl::default(),
            status: ApuStatus::default(),

            frame_sequence: false,
            disable_frame_interrupt: false,

            frame_update_counter: 0,
            frame_update_mode1: false,

            last_output: Instant::now(),
            avg_rates: Vec::new(),
            batches_sent: 0,
        }
    }

    pub fn cycle(&mut self) {
        self.clocks += 1;
        self.frame_clocks += 1;
        self.clocks_since_sampled += 1;

        if self.frame_clocks == QUARTER_FRAME_CLOCKS
            || self.frame_clocks == HALF_FRAME_CLOCKS
            || self.frame_clocks == THREE_QUARTER_FRAME_CLOCKS
            || self.frame_clocks == WHOLE_FRAME_CLOCKS {
            
            self.frame_update();
        }

        if self.frame_clocks == WHOLE_FRAME_CLOCKS {
            self.frame_clocks = 0;
        }

        let time_since_sampled = self.clocks_since_sampled as f64 * CPU_CYCLE_PERIOD;

        if time_since_sampled > SAMPLE_PERIOD {
            let sample = self.generate_sample();

            self.push_sample(sample);

            self.clocks_since_sampled = 0;
        }
    }

    pub fn cpu_write(&mut self, address: u16, data: u8) {
        match address {
            // Pulse 1 Registers
            0x4000 => {
                let new_duty_cycle = (data >> 6) & 3;
                let new_enable = (data & 0x20) == 0;
                let new_const_volume = (data & 0x10) != 0;
                let new_volume = data & 0x0F;

                self.pulse1_channel.counter_enabled = new_enable;
                self.pulse1_channel.duty_cycle_percent = match new_duty_cycle {
                    0 => 0.125,
                    1 => 0.25,
                    2 => 0.50,
                    3 => 0.75,
                    _ => { unreachable!("Holy wack unlyrical lyrics, Batman!"); }
                };

                // self.pulse1_regs.set_duty_cycle(new_duty_cycle);
                // self.pulse1_regs.set_disable(new_disable);
                // self.pulse1_regs.set_const_volume(new_const_volume);
                // self.pulse1_regs.set_envelope_volume(new_volume);
            }

            // Pulse 1 Timer Low
            0x4002 => {
                self.pulse1_channel.timer &= 0x700;
                self.pulse1_channel.timer |= data as usize;

                self.update_pulse1_freq();
            }

            // Pulse 1 Timer High & Length Counter
            0x4003 => {
                self.pulse1_channel.timer &= 0xFF;
                self.pulse1_channel.timer |= ((data & 7) as usize) << 8;

                self.pulse1_channel.set_length_counter(data >> 3);

                self.update_pulse1_freq();
            }

            // Pulse 2 Registers
            0x4004 => {
                let new_duty_cycle = (data >> 6) & 3;
                let new_enable = (data & 0x20) == 0;
                let new_const_volume = (data & 0x10) != 0;
                let new_volume = data & 0x0F;

                self.pulse2_channel.counter_enabled = new_enable;
                self.pulse2_channel.duty_cycle_percent = match new_duty_cycle {
                    0 => 0.125,
                    1 => 0.25,
                    2 => 0.50,
                    3 => 0.75,
                    _ => { unreachable!("Holy wack unlyrical lyrics, Batman!"); }
                };

                // println!("Writing to Pulse 2, Enabled: {}", self.pulse2_channel.enabled);

                // self.pulse1_regs.set_duty_cycle(new_duty_cycle);
                // self.pulse1_regs.set_disable(new_disable);
                // self.pulse1_regs.set_const_volume(new_const_volume);
                // self.pulse1_regs.set_envelope_volume(new_volume);
            }

            // Pulse 2 Timer Low
            0x4006 => {
                self.pulse2_channel.timer &= 0x700;
                self.pulse2_channel.timer |= data as usize;

                self.update_pulse1_freq();
            }

            // Pulse 2 Timer High & Length Counter
            0x4007 => {
                self.pulse2_channel.timer &= 0xFF;
                self.pulse2_channel.timer |= ((data & 7) as usize) << 8;

                self.pulse2_channel.set_length_counter(data >> 3);

                self.update_pulse2_freq();
            }



            0x4015 => {
                let pulse1_enabled = (data & 0x01) != 0;
                let pulse2_enabled = (data & 0x02) != 0;

                self.pulse1_channel.enabled = pulse1_enabled;
                self.pulse2_channel.enabled = pulse2_enabled;
            }

            _ => {}
        }
    }

    fn generate_sample(&mut self) -> f32 {

        let pulse1_sample = self.pulse1_channel.sample(self.clocks);
        let pulse2_sample = self.pulse2_channel.sample(self.clocks);

        let amplitude = 0.25;

        let sample = (pulse1_sample + pulse2_sample) * amplitude;

        sample
    }

    fn push_sample(&mut self, sample: f32) {
        self.sample_batch.push(sample);

        if self.sample_batch.len() >= SAMPLE_BATCH_SIZE {
            self.sample_queue.lock().unwrap()
                .extend(self.sample_batch.drain(..));

            // let batch_avg_sample_rate = 2048.0 / self.last_output.elapsed().as_secs_f64();

            // println!("Avg. {} samples / sec", batch_avg_sample_rate);

            // self.avg_rates.push(batch_avg_sample_rate);

            // let total_avg = self.avg_rates.iter().sum::<f64>() / self.batches_sent as f64;

            // if self.batches_sent % 100 == 0 {
            //     println!("Running Average Sample Rate: {} ==================", total_avg);
            // }

            self.last_output = Instant::now();
            self.batches_sent += 1;
        }
    }

    pub fn audio_samples_queued(&self) -> usize {
        self.sample_queue.lock().unwrap().len()
    }

    fn update_pulse1_freq(&mut self) {
        let t = self.pulse1_channel.timer;

        let frequency = CPU_FREQ / (16.0 * (t + 1) as f64);

        self.pulse1_channel.freq = frequency;
    }

    fn update_pulse2_freq(&mut self) {
        let t = self.pulse2_channel.timer;

        let frequency = CPU_FREQ / (16.0 * (t + 1) as f64);

        self.pulse2_channel.freq = frequency;
    }

    fn frame_update(&mut self) {
        if self.frame_update_mode1 {
            match self.frame_update_counter {
                0 => {},
                1 => { self.update_length_counters() },
                2 => {},
                3 => {},
                4 => { self.update_length_counters() },
                _ => {},
            }

            self.frame_update_counter = (self.frame_update_counter + 1) % 5;
        } 
        else {
            match self.frame_update_counter {
                0 => {},
                1 => { self.update_length_counters() },
                2 => {},
                3 => { self.update_length_counters() },
                _ => {},
            }

            self.frame_update_counter = (self.frame_update_counter + 1) % 4;
        }
    }

    fn update_length_counters(&mut self) {
        self.pulse1_channel.update_counter();
        self.pulse2_channel.update_counter();
    }
}
