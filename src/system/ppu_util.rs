use std::cell::Cell;

use bitfield_struct::bitfield;

// use super::ppu::PpuCtrl;

/// Struct containing all of the PPUs registers, internal and external. 
/// Encapsulates most of the state of the PPU itself.
#[derive(Default, Debug)]
pub struct PpuRegisters {
    ppu_ctrl: Cell<PpuCtrl>,
    ppu_mask: Cell<PpuMask>,
    ppu_status: Cell<PpuStatus>,
    oam_address: Cell<u8>,
    oam_data: Cell<u8>,
    ppu_scroll: Cell<u16>,
    ppu_address: Cell<u16>,
    ppu_data: Cell<u8>,
    read_buffer: Cell<u8>,

    oam_dma: Cell<u8>,

    // Current VRAM Address (15 least significant bits)
    v_reg: Cell<PpuScrollPos>,
    // Temporary VRAM Address (15 least significant bits)
    t_reg: Cell<PpuScrollPos>,
    // Fine X scroll (3 least significant bits)
    fine_x: Cell<u8>,
    // First or second write toggle (least significant bit)
    write_latch: Cell<u8>,
}

impl PpuRegisters {

    // READ / WRITE FUNCTIONS FOR CPU USE

    /// Takes an address in CPU address space and reads the value of a PPU 
    /// register as a u8. Some registers cannot be read.
    pub fn read(&self, address: u16) -> u8 {
        match address & 0x0007 {
            0 => 0x00, // Can't read PPUCTRL
            1 => 0x00, // Can't read PPUMASK
            2 => {
                // Reads from $2002 reset write latch
                self.write_latch.set(0);
                self.ppu_status.get().0
            },
            3 => 0x00, // Can't read OAMADDR
            4 => self.oam_data.get(),
            5 => 0x00, // Can't read PPUSCROLL
            6 => 0x00, // Can't read PPUADDR
            7 => {
                // Reads are too slow for the PPU to respond immediatly, so they
                // go to a read buffer. With the exception (because of coarse
                // there's an exception) of palette memory, which responds
                // immediatly and still updates the read buffer, discarding the
                // old read buffer data.
                let mut val = self.read_buffer.get();
                self.read_buffer.set(self.ppu_data.get());

                if self.v_reg.get().0 >= 0x3F00 {
                    val = self.read_buffer.get();
                }

                self.v_reg.set(PpuScrollPos(
                    self.v_reg.get().0 + if self.ppu_ctrl.get().vram_addr_inc() == 0 { 1 } else { 32 }
                ));

                val
            },
            _ => {
                unreachable!("If the laws of physics no longer apply in the future, God help you.")
            }
        }
    }

    /// Write a single byte to the PPU Registers. Internal Registers cannot be
    /// written to, and some registers depend on the internal write latch to
    /// determine which byte is being written.
    pub fn write(&self, address: u16, data: u8) {
        match address & 0x0007 {
            0 => {
                self.ppu_ctrl.set(PpuCtrl::from_bits(data));
                self.t_reg.set(self.t_reg.get()
                    .with_nt_select((data & 3) as usize));
            },
            1 => self.ppu_mask.set(PpuMask::from_bits(data)),
            2 => {}, // Cannot write PPUSTATUS
            3 => self.oam_address.set(data),
            4 => self.oam_data.set(data),
            5 => {
                if self.write_latch.get() == 0 {
                    // 1st Write => write to low byte
                    self.ppu_scroll.set((self.ppu_scroll.get() & 0xFF00) | data as u16);
                    // Update internal regs
                    // t: ....... ...ABCDE <- d: ABCDE...
                    // x:              FGH <- d: .....FGH
                    // w:                  <- 1
                    self.t_reg.set(self.t_reg.get()
                        .with_coarse_x((data >> 3) as usize));
                    self.fine_x.set(data & 7);
                    self.write_latch.set(1);
                } else {
                    // 2nd Write => Write to high byte
                    self.ppu_scroll.set((self.ppu_scroll.get() & 0x00FF) | ((data as u16) << 8));
                    // Update internal regs
                    // t: FGH..AB CDE..... <- d: ABCDEFGH
                    // w:                  <- 0
                    self.t_reg.set(self.t_reg.get()
                        .with_coarse_y((data >> 3) as usize)
                        .with_fine_y((data & 7) as usize));
                    self.write_latch.set(0);
                }
            },
            6 => {
                if self.write_latch.get() == 0 {
                    // 1st Write => write to low byte
                    self.ppu_address.set((self.ppu_address.get() & 0xFF00) | data as u16);
                    // Update internal regs
                    // t: .CDEFGH ........ <- d: ..CDEFGH
                    //     <unused>     <- d: AB......
                    // t: Z...... ........ <- 0 (bit Z is cleared)
                    // w:                  <- 1
                    self.t_reg.set(PpuScrollPos(
                        (self.t_reg.get().0 & 0xC0FF) | (((data & 0x3F) as u16) << 8)
                    ));
                    self.write_latch.set(1);
                } else {
                    // 2nd Write => Write to high byte
                    self.ppu_address.set((self.ppu_address.get() & 0x00FF) | ((data as u16) << 8));
                    // Update internal regs
                    // t: ....... ABCDEFGH <- d: ABCDEFGH
                    // v: <...all bits...> <- t: <...all bits...>
                    // w:                  <- 0
                    self.t_reg.set(PpuScrollPos(
                        (self.t_reg.get().0 & 0xFF00) | (data as u16)
                    ));
                    self.v_reg.set(self.t_reg.get());
                    self.write_latch.set(0);
                }
            },
            7 => {
                self.ppu_data.set(data);

                self.v_reg.set(PpuScrollPos(
                    self.v_reg.get().0 + if self.ppu_ctrl.get().vram_addr_inc() == 0 { 1 } else { 32 }
                ));
            },
            _ => unreachable!("Well done. Here are the test results: \"You are a horrible person.\" I'm serious, that's what it says: \"A horrible person.\" We weren't even testing for that.")
        };
    }

    // GETTER / SETTER FUNCTIONS FOR PPU USE

    /// Get a copy of the value of PPUCTRL as a PpuCtrl struct
    pub fn ctrl(&self) -> PpuCtrl {
        self.ppu_ctrl.get()
    }
    /// Get a copy of the value of PPUCTRL as a u8
    pub fn ctrl_val(&self) -> u8 {
        self.ppu_ctrl.get().0
    }
    /// Get a copy of the value of PPUMASK as a PpuMask struct
    pub fn mask(&self) -> PpuMask {
        self.ppu_mask.get()
    }
    /// Get a copy of the value of PPUMASK as a u8
    pub fn mask_val(&self) -> u8 {
        self.ppu_mask.get().0
    }
    /// Get a copy of the value of PPUSTATUS as a PpuStatus struct
    pub fn status(&self) -> PpuStatus {
        self.ppu_status.get()
    }
    /// Get a copy of the value of PPUSTATUS as a u8
    pub fn status_val(&self) -> u8 {
        self.ppu_status.get().0
    }
    /// Get a copy of the value of OAMADDR
    pub fn oam_address(&self) -> u8 {
        self.oam_address.get()
    }
    /// Get a copy of the value of OAMDATA
    pub fn oam_data(&self) -> u8 {
        self.oam_data.get()
    }
    /// Get a copy of the value of PPUSCROLL
    pub fn scroll(&self) -> u16 {
        self.ppu_scroll.get()
    }
    /// Get a copy of the value of PPUADDR
    pub fn address(&self) -> u16 {
        self.ppu_address.get()
    }
    /// Get a copy of the value of PPUDATA
    pub fn data(&self) -> u8 {
        self.ppu_data.get()
    }
    /// Get a copy of the value of OAMDMA
    pub fn oam_dma(&self) -> u8 {
        self.oam_dma.get()
    }
    /// Get a copy of the value of the v register as a PpuScrollPos struct
    pub fn v_reg(&self) -> PpuScrollPos {
        self.v_reg.get()
    }
    /// Get a copy of the value of the v register as a u16
    pub fn v_val(&self) -> u16 {
        self.v_reg.get().0
    }
    /// Get a copy of the value of the t register as a PpuScrollPos struct
    pub fn t_reg(&self) -> PpuScrollPos {
        self.t_reg.get()
    }
    /// Get a copy of the value of the t register as a u16
    pub fn t_val(&self) -> u16 {
        self.t_reg.get().0
    }
    /// Get a copy of the value of the fine x offset
    pub fn fine_x(&self) -> u8 {
        self.fine_x.get()
    }
    /// Get a copy of the value of the write latch register
    pub fn get_write_latch(&self) -> u8 {
        self.write_latch.get()
    }


    /// Set the value of PPUCTRL
    pub fn set_ctrl(&self, val: u8) {
        self.ppu_ctrl.set(PpuCtrl(val));
    }
    /// Set the nametable address bits (0..=1) of PPUCTRL
    pub fn set_nt_addr(&self, val: usize) {
        self.ppu_ctrl.set(self.ppu_ctrl.get().with_nametable(val));
    }
    /// Set the VRAM address increment bit (2) of PPUCTRL
    pub fn set_vram_addr_inc(&self, val: usize) {
        self.ppu_ctrl.set(self.ppu_ctrl.get().with_vram_addr_inc(val));
    }
    /// Set the sprite pattern table address bits (3..=4) of PPUCTRL
    pub fn set_spr_pt_addr(&self, val: usize) {
        self.ppu_ctrl.set(self.ppu_ctrl.get().with_spr_pattern_tbl(val));
    }
    /// Set the background pattern table address bit (5) of PPU CTRL
    pub fn set_bg_pt_addr(&self, val: usize) {
        self.ppu_ctrl.set(self.ppu_ctrl.get().with_bg_pattern_tbl(val));
    }
    /// Set the sprite size bit (6) of PPUCTRL
    pub fn set_spr_size(&self, val: usize) {
        self.ppu_ctrl.set(self.ppu_ctrl.get().with_spr_size(val));
    }
    /// Set the master/slave select bit (7) of PPUCTRL [bit is unused in this emulation]
    pub fn set_mstr_slave(&self, val: usize) {
        self.ppu_ctrl.set(self.ppu_ctrl.get().with_mstr_slave(val));
    }
    /// Set the vertical blank NMI trigger bit (7) of PPUCTRL
    pub fn set_vblank_nmi(&self, val: usize) {
        self.ppu_ctrl.set(self.ppu_ctrl.get().with_vblank_nmi(val));
    }

    /// Set the value of the PPUMASK register
    pub fn set_mask(&self, val: u8) {
        self.ppu_mask.set(PpuMask(val));
    }
    /// Set the greyscale bit (0) of PPUMASK
    pub fn set_grayscale(&self, val: usize) {
        self.ppu_mask.set(self.ppu_mask.get().with_greyscale(val));
    }
    /// Set the show background left bit (1) of PPUMASK
    pub fn set_draw_bg_left(&self, val: usize) {
        self.ppu_mask.set(self.ppu_mask.get().with_draw_bg_left(val));
    }
    pub fn set_draw_spr_left(&self, val: usize) {
        self.ppu_mask.set(self.ppu_mask.get().with_draw_spr_left(val));
    }
    /// Set the show background bit (3) of PPUMASK
    pub fn set_draw_bg(&self, val: usize) {
        self.ppu_mask.set(self.ppu_mask.get().with_draw_bg(val));
    }
    /// Set the show sprites bit (4) of PPUMASK
    pub fn set_draw_sprites(&self, val: usize) {
        self.ppu_mask.set(self.ppu_mask.get().with_draw_sprites(val));
    }
    /// Set the emphasize red bit (5) of PPUMASK
    pub fn set_emph_red(&self, val: usize) {
        self.ppu_mask.set(self.ppu_mask.get().with_emph_red(val));
    }
    /// Set the emphasize green bit (5) of PPUMASK
    pub fn set_emph_grn(&self, val: usize) {
        self.ppu_mask.set(self.ppu_mask.get().with_emph_grn(val));
    }
    /// Set the emphasize blu bit (5) of PPUMASK
    pub fn set_emph_blu(&self, val: usize) {
        self.ppu_mask.set(self.ppu_mask.get().with_emph_blu(val));
    }

    /// Set the value of the PPUSTATUS register
    pub fn set_status(&self, val: u8) {
        self.ppu_status.set(PpuStatus(val));
    }
    /// Set the open bus bits (0..=4) of PPUSTATUS
    pub fn set_open_bus(&self, val: usize) {
        self.ppu_status.set(self.ppu_status.get().with_open_bus(val));
    }
    /// Set the sprite overflow bit (5) of PPUSTATUS
    pub fn set_spr_overflow(&self, val: usize) {
        self.ppu_status.set(self.ppu_status.get().with_spr_overflow(val));
    }
    /// Set the sprite 0 hit bit (6) of PPUSTATUS
    pub fn set_spr_0_hit(&self, val: usize) {
        self.ppu_status.set(self.ppu_status.get().with_spr_0_hit(val));
    }
    /// Set the in vertical blank bit (7) of PPUSTATUS
    pub fn set_in_vblank(&self, val: usize) {
        self.ppu_status.set(self.ppu_status.get().with_in_vblank(val));
    }

    /// Set the value of OAMADDR
    pub fn set_oam_address(&self, val: u8) {
        self.oam_address.set(val);
    }
    /// Set the value of OAMDATA
    pub fn set_oam_data(&self, val: u8) {
        self.oam_data.set(val);
    }
    /// Set the value of PPUSCROLL
    pub fn set_scroll(&self, val: u16) {
        self.ppu_scroll.set(val);
    }
    /// Set the value of PPUADDR
    pub fn set_address(&self, val: u16) {
        self.ppu_address.set(val);
    }
    /// Set the value of PPUDATA
    pub fn set_data(&self, val: u8) {
        self.ppu_data.set(val);
    }
    /// Set the value of OAMDMA
    pub fn set_oam_dma(&self, val: u8) {
        self.oam_dma.set(val);
    }

    /// Set the value of the v register
    pub fn set_v_reg(&self, val: u16) {
        self.v_reg.set(PpuScrollPos(val));
    }
    /// Set the coarse x bits (0..=4) of the v register
    pub fn set_v_coarse_x(&self, val: usize) {
        self.v_reg.set(self.v_reg.get().with_coarse_x(val));
    }
    /// Set the coarse y bits (5..=9) of the v register
    pub fn set_v_coarse_y(&self, val: usize) {
        self.v_reg.set(self.v_reg.get().with_coarse_y(val));
    }
    /// Set the nametavle select bits (10..=11) of the v register
    pub fn set_v_nt_select(&self, val: usize) {
        self.v_reg.set(self.v_reg.get().with_nt_select(val));
    }
    /// Set only the nametable x bit (10) of the v register
    pub fn set_v_nt_x(&self, val: usize) {
        self.v_reg.set(PpuScrollPos((self.v_reg.get().0 & 0xFBFF) | ((val as u16) << 10)));
    }
    /// Set only the nametable y bit (11) of the v register
    pub fn set_v_nt_y(&self, val: usize) {
        self.v_reg.set(PpuScrollPos((self.v_reg.get().0 & 0xF7FF) | ((val as u16) << 11)));
    }
    /// Set the fine y bits (12..=14) of the v register
    pub fn set_v_fine_y(&self, val: usize) {
        self.v_reg.set(self.v_reg.get().with_fine_y(val));
    }

    /// Set the value of the t register
    pub fn set_t_reg(&self, val: u16) {
        self.t_reg.set(PpuScrollPos(val));
    }
    /// Set the coarse x bits (0..=4) of the t register
    pub fn set_t_coarse_x(&self, val: usize) {
        self.t_reg.set(self.t_reg.get().with_coarse_x(val));
    }
    /// Set the coarse y bits (5..=9) of the t register
    pub fn set_t_coarse_y(&self, val: usize) {
        self.t_reg.set(self.t_reg.get().with_coarse_y(val));
    }
    /// Set the nametavle select bits (10..=11) of the t register
    pub fn set_t_nt_select(&self, val: usize) {
        self.t_reg.set(self.t_reg.get().with_nt_select(val));
    }
    /// Set only the nametable x bit (10) of the t register
    pub fn set_t_nt_x(&self, val: usize) {
        self.t_reg.set(PpuScrollPos((self.t_reg.get().0 & 0xFBFF) | ((val as u16) << 10)));
    }
    /// Set only the nametable y bit (11) of the t register
    pub fn set_t_nt_y(&self, val: usize) {
        self.t_reg.set(PpuScrollPos((self.t_reg.get().0 & 0xF7FF) | ((val as u16) << 11)));
    }
    /// Set the fine y bits (12..=14) of the t register
    pub fn set_t_fine_y(&self, val: usize) {
        self.t_reg.set(self.t_reg.get().with_fine_y(val));
    }

    /// Set the value of the fine x offset
    pub fn set_fine_x(&self, val: u8) {
        self.fine_x.set(val);
    }
    /// Set the value of the write latch register
    pub fn set_write_latch(&self, val: u8) {
        self.write_latch.set(val);
    }
}

// PPUCTRL Register (write only)
//     7  bit  0
//     ---- ----
//     VPHB SINN
//     |||| ||||
//     |||| ||++- Base nametable address
//     |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
//     |||| |+--- VRAM address increment per CPU read/write of PPUDATA
//     |||| |     (0: add 1, going across; 1: add 32, going down)
//     |||| +---- Sprite pattern table address for 8x8 sprites
//     ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
//     |||+------ Background pattern table address (0: $0000; 1: $1000)
//     ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels – see PPU OAM#Byte 1)
//     |+-------- PPU master/slave select
//     |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
//     +--------- Generate an NMI at the start of the
//             vertical blanking interval (0: off; 1: on)
#[bitfield(u8)]
pub struct PpuCtrl {
    #[bits(2)]
    pub nametable: usize,
    #[bits(1)]
    pub vram_addr_inc: usize,
    #[bits(1)]
    pub spr_pattern_tbl: usize,
    #[bits(1)]
    pub bg_pattern_tbl: usize,
    #[bits(1)]
    pub spr_size: usize,
    #[bits(1)]
    pub mstr_slave: usize,
    #[bits(1)]
    pub vblank_nmi: usize,
}

// PPUMASK Register (write only)
//     7  bit  0
//     ---- ----
//     BGRs bMmG
//     |||| ||||
//     |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
//     |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
//     |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
//     |||| +---- 1: Show background
//     |||+------ 1: Show sprites
//     ||+------- Emphasize red (green on PAL/Dendy)
//     |+-------- Emphasize green (red on PAL/Dendy)
//     +--------- Emphasize blue
#[bitfield(u8)]
pub struct PpuMask {
    #[bits(1)]
    pub greyscale: usize,
    #[bits(1)]
    pub draw_bg_left: usize,
    #[bits(1)]
    pub draw_spr_left: usize,
    #[bits(1)]
    pub draw_bg: usize,
    #[bits(1)]
    pub draw_sprites: usize,
    #[bits(1)]
    pub emph_red: usize,
    #[bits(1)]
    pub emph_grn: usize,
    #[bits(1)]
    pub emph_blu: usize,
}

// PPUSTATUS Register (read only)
//     7  bit  0
//     ---- ----
//     VSO. ....
//     |||| ||||
//     |||+-++++- PPU open bus. Returns stale PPU bus contents.
//     ||+------- Sprite overflow. The intent was for this flag to be set
//     ||         whenever more than eight sprites appear on a scanline, but a
//     ||         hardware bug causes the actual behavior to be more complicated
//     ||         and generate false positives as well as false negatives; see
//     ||         PPU sprite evaluation. This flag is set during sprite
//     ||         evaluation and cleared at dot 1 (the second dot) of the
//     ||         pre-render line.
//     |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
//     |          a nonzero background pixel; cleared at dot 1 of the pre-render
//     |          line.  Used for raster timing.
//     +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
//             Set at dot 1 of line 241 (the line *after* the post-render
//             line); cleared after reading $2002 and at dot 1 of the
//             pre-render line.
#[bitfield(u8)]
pub struct PpuStatus {
    #[bits(5)]
    pub open_bus: usize,
    #[bits(1)]
    pub spr_overflow: usize,
    #[bits(1)]
    pub spr_0_hit: usize,
    #[bits(1)]
    pub in_vblank: usize,
}

// PPU Scroll Position
//     yyy NN YYYYY XXXXX
//     ||| || ||||| +++++-- coarse X scroll
//     ||| || +++++-------- coarse Y scroll
//     ||| ++-------------- nametable select (bit 0 x, bit 1 y)
//     +++----------------- fine Y scroll
#[bitfield(u16)]
pub struct PpuScrollPos {
    #[bits(5)]
    pub coarse_x: usize,
    #[bits(5)]
    pub coarse_y: usize,
    #[bits(2)]
    pub nt_select: usize,
    #[bits(3)]
    pub fine_y: usize,
    // fine_x is taken from
    _unused: bool,
}

impl PpuScrollPos {
    /// Bits 10..11
    pub fn nt_x(&self) -> usize { self.nt_select() & 1 }
    /// Bits 10..11
    pub fn set_nt_x(&mut self, value: usize) {
        self.set_nt_select((self.nt_select() & 2) | (value & 1));
    }
    /// Bits 10..11
    pub fn with_nt_x(self, value: usize) -> PpuScrollPos {
        PpuScrollPos(((self.0 as usize & 0xFBFF) | ((value & 1) << 10)) as u16)
    }

    /// Bits 11..12
    pub fn nt_y(&self) -> usize { self.nt_select() >> 1 }
    /// Bits 11..12
    pub fn set_nt_y(&mut self, value: usize) {
        self.set_nt_select((self.nt_select() & 1) | ((value & 1) << 1) );
    }
    /// Bits 11..12
    pub fn with_nt_y(self, value: usize) -> PpuScrollPos {
        PpuScrollPos(((self.0 as usize & 0xF7FF) | ((value & 1) << 11)) as u16)
    }
}