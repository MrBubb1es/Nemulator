use bitfield_struct::bitfield;

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
pub struct PpuScrollReg {
    #[bits(5)]
    pub coarse_x: usize,
    #[bits(5)]
    pub coarse_y: usize,
    #[bits(2)]
    pub nt_select: usize,
    #[bits(3)]
    pub fine_y: usize,
    // fine_x is taken from
    pub _unused: bool,
}

impl PpuScrollReg {
    /// Bits 10..11
    pub fn nt_x(&self) -> usize { self.nt_select() & 1 }
    /// Bits 10..11
    pub fn set_nt_x(&mut self, value: usize) {
        self.set_nt_select((self.nt_select() & 2) | (value & 1));
    }
    /// Bits 10..11
    pub fn with_nt_x(self, value: usize) -> PpuScrollReg {
        PpuScrollReg(((self.0 as usize & 0xFBFF) | ((value & 1) << 10)) as u16)
    }

    /// Bits 11..12
    pub fn nt_y(&self) -> usize { self.nt_select() >> 1 }
    /// Bits 11..12
    pub fn set_nt_y(&mut self, value: usize) {
        self.set_nt_select((self.nt_select() & 1) | ((value & 1) << 1) );
    }
    /// Bits 11..12
    pub fn with_nt_y(self, value: usize) -> PpuScrollReg {
        PpuScrollReg(((self.0 as usize & 0xF7FF) | ((value & 1) << 11)) as u16)
    }
}