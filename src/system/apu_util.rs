use bitfield_struct::bitfield;

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



