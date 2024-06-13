use std::{collections::VecDeque, f32::consts::TAU};
use std::sync::{Arc, Mutex};

use rodio::queue::SourcesQueueInput;

use super::apu_util::{
    ApuControl, ApuStatus, DmcRegisters, NoiseRegisters, PulseChannel, PulseRegisters, TriangleRegisters, NES_AUDIO_FREQUENCY
};

const SAMPLE_PERIOD: f64 = 1.0 / 44_100.0;
const CPU_FREQ: f64 = 1_789_773f64; // For NTSC systems
pub const CPU_CYCLE_PERIOD: f64 = 1.0 / CPU_FREQ;
const SAMPLE_BATCH_SIZE: usize = 2048;
const FRAME_COUNTER_FREQ: u64 = 7457;

pub struct Apu2A03 {
    sample_queue: Arc<Mutex<VecDeque<f32>>>,
    sample_batch: Vec<f32>,
    
    clocks: u64,
    clocks_since_sampled: usize,

    pulse1_regs: PulseRegisters,
    pulse2_regs: PulseRegisters,
    triangle_regs: TriangleRegisters,
    noise_regs: NoiseRegisters,
    dmc_regs: DmcRegisters,

    pulse1_channel: PulseChannel,

    control: ApuControl,
    status: ApuStatus,

    frame_sequence: bool,
    disable_frame_interrupt: bool,

    frame_update_counter: usize,
    frame_update_mode1: bool,
}

impl Apu2A03 {
    pub fn new(sample_queue: Arc<Mutex<VecDeque<f32>>>) -> Self {
        Self {
            sample_queue,
            sample_batch: Vec::with_capacity(SAMPLE_BATCH_SIZE),

            clocks: 0,
            clocks_since_sampled: 0,

            pulse1_regs: PulseRegisters::default(),
            pulse2_regs: PulseRegisters::default(),
            triangle_regs: TriangleRegisters::default(),
            noise_regs: NoiseRegisters::default(),
            dmc_regs: DmcRegisters::default(),

            pulse1_channel: PulseChannel::default(),


            control: ApuControl::default(),
            status: ApuStatus::default(),

            frame_sequence: false,
            disable_frame_interrupt: false,

            frame_update_counter: 0,
            frame_update_mode1: false,
        }
    }

    pub fn cycle(&mut self) {

        if self.clocks % FRAME_COUNTER_FREQ == 0 {
            self.frame_update();
        }

        let time_since_sampled = self.clocks_since_sampled as f64 * CPU_CYCLE_PERIOD;

        if time_since_sampled > SAMPLE_PERIOD {
            let sample = self.generate_sample();

            self.push_sample(sample);

            self.clocks_since_sampled = 0;
        }

        self.clocks += 1;
        self.clocks_since_sampled += 1;
    }

    pub fn cpu_write(&mut self, address: u16, data: u8) {
        match address {
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

                // println!("Enabled: {}", self.pulse1_channel.enabled);
                // println!("New duty cycle of {}%", (self.pulse1_channel.duty_cycle_percent * 100.0) as usize);

                // self.pulse1_regs.set_duty_cycle(new_duty_cycle);
                // self.pulse1_regs.set_disable(new_disable);
                // self.pulse1_regs.set_const_volume(new_const_volume);
                // self.pulse1_regs.set_envelope_volume(new_volume);
            }

            0x4001 => {
                let new_duty_cycle = (data >> 6) & 3;
                let new_disable = (data & 0x20) != 0;
                let new_const_volume = (data & 0x10) != 0;
                let new_volume = data & 0x0F;

                // self.pulse2_regs.set_duty_cycle(new_duty_cycle);
                // self.pulse2_regs.set_disable(new_disable);
                // self.pulse2_regs.set_const_volume(new_const_volume);
                // self.pulse2_regs.set_envelope_volume(new_volume);
            }

            // Pulse 1 Timer Low
            0x4002 => {
                self.pulse1_regs.set_timer_lo(data);

                self.update_pulse1_freq();
            }

            0x4003 => {
                self.pulse1_regs.set_timer_hi(data & 7);
                self.pulse1_channel.set_length_counter(data >> 3);

                self.update_pulse1_freq();
            }

            0x4015 => {
                let pulse1_enabled = (data & 0x01) != 0;

                self.pulse1_channel.enabled = pulse1_enabled;
            }

            _ => {}
        }
    }

    fn generate_sample(&mut self) -> f32 {

        let pulse1_sample = self.pulse1_channel.sample(self.clocks);

        let sample = pulse1_sample;

        sample
    }

    fn push_sample(&mut self, sample: f32) {
        self.sample_batch.push(sample);

        if self.sample_batch.len() >= SAMPLE_BATCH_SIZE {
            self.sample_queue.lock().unwrap()
                .extend(self.sample_batch.drain(..));
        }
    }

    pub fn audio_samples_queued(&self) -> usize {
        self.sample_queue.lock().unwrap().len()
    }

    fn update_pulse1_freq(&mut self) {
        let t_hi = self.pulse1_regs.timer_hi() as u16;
        let t_lo = self.pulse1_regs.timer_lo() as u16;

        let t = (t_hi << 8) | t_lo;

        let frequency = CPU_FREQ / (16.0 * (t + 1) as f64);

        self.pulse1_channel.freq = frequency;

        println!("New Freq: {}", self.pulse1_channel.freq);

        if t < 8 {
            self.pulse1_channel.enabled = false;
        }
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
    }
}
