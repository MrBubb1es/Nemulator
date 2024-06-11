use std::f32::consts::TAU;
use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use super::apu_util::{
    ApuControl, ApuStatus, DmcRegisters, NoiseRegisters, PulseRegisters, TriangleRegisters,
};

const SAMPLE_PERIOD: f32 = 1f32 / 44_100f32;
const CPU_FREQ: f32 = 1_789_773f32; // For NTSC systems
const CPU_CYCLE_PERIOD: f32 = 1.0 / CPU_FREQ;

pub struct Apu2A03 {
    sample_output_channel: Arc<Sender<f32>>,
    clocks: u64,
    emulator_time: f32,

    pulse1_regs: PulseRegisters,
    pulse2_regs: PulseRegisters,
    triangle_regs: TriangleRegisters,
    noise_regs: NoiseRegisters,
    dmc_regs: DmcRegisters,

    control: ApuControl,
    status: ApuStatus,

    frame_sequence: bool,
    disable_frame_interrupt: bool,
}

impl Apu2A03 {
    pub fn new(sample_output_channel: Arc<tokio::sync::mpsc::Sender<f32>>) -> Self {
        Self {
            sample_output_channel,
            clocks: 0,
            emulator_time: 0.0,

            pulse1_regs: PulseRegisters::default(),
            pulse2_regs: PulseRegisters::default(),
            triangle_regs: TriangleRegisters::default(),
            noise_regs: NoiseRegisters::default(),
            dmc_regs: DmcRegisters::default(),

            control: ApuControl::default(),
            status: ApuStatus::default(),

            frame_sequence: false,
            disable_frame_interrupt: false,
        }
    }

    pub fn cycle(&mut self) {
        self.clocks += 1;
        let emulated_time = self.clocks as f32 * CPU_CYCLE_PERIOD;
        if emulated_time > SAMPLE_PERIOD {
            self.send_sample(emulated_time);
        }
    }

    pub fn cpu_write(&mut self, address: u16, data: u8) {
        match address {
            0x4000 => {
                let new_duty_cycle = (data >> 6) & 3;
                let new_disable = (data & 0x20) != 0;
                let new_const_volume = (data & 0x10) != 0;
                let new_volume = data & 0x0F;

                self.pulse1_regs.set_duty_cycle(new_duty_cycle);
                self.pulse1_regs.set_disable(new_disable);
                self.pulse1_regs.set_const_volume(new_const_volume);
                self.pulse1_regs.set_envelope_volume(new_volume);
            }

            0x4001 => {
                let new_duty_cycle = (data >> 6) & 3;
                let new_disable = (data & 0x20) != 0;
                let new_const_volume = (data & 0x10) != 0;
                let new_volume = data & 0x0F;

                self.pulse2_regs.set_duty_cycle(new_duty_cycle);
                self.pulse2_regs.set_disable(new_disable);
                self.pulse2_regs.set_const_volume(new_const_volume);
                self.pulse2_regs.set_envelope_volume(new_volume);
            }

            _ => {}
        }
    }

    pub fn send_sample(&self, time: f32) {
        // let timer1 = (self.pulse1_regs.timer_hi() << 8) | self.pulse1_regs.timer_lo();
        // let freq: f32 = CPU_FREQ / (16 * (timer1 as f32 + 1));

        // NOTE: only for testing
        let freq: f32 = 440.0;

        let sample = f32::sin(TAU * freq * time);

        self.sample_output_channel.send(sample);
    }
}
