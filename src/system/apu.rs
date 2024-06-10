use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use super::apu_util::{ApuControl, ApuStatus, DmcRegisters, NoiseRegisters, PulseRegisters, TriangleRegisters};

pub struct Apu2A03 {
    sample_output_channel: Arc<Sender<f32>>,
    clocks: u64,

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
    pub fn cycle(&self) {
        // self.sample_output_channel.send(value)
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
}