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
    ppu_data: Cell<u8>,
    read_buffer: Cell<u8>,

    oam_dma: Cell<u8>,

    // Current VRAM Address (15 least significant bits)
    v_reg: Cell<PpuScrollReg>,
    // Temporary VRAM Address (15 least significant bits)
    t_reg: Cell<PpuScrollReg>,
    // Fine X scroll (3 least significant bits)
    fine_x: Cell<u8>,
    // First or second write toggle (least significant bit)
    write_latch: Cell<u8>,

    // Set every time the CPU writes data to the PPUDATA register
    write_ppu_data: Cell<bool>,
    // Stores the address the CPU is trying to write to until the write is 
    // performed. This is required because PpuRegisters doesn't have access to
    // the PPU itself, and thus cannot perform the write here. The solution is
    // to have the PPU check the write_ppu_data flag for when a CPU write
    // occurs, and use this value as the address, which is internally set to the
    // VRAM address the CPU would be using.
    write_address: Cell<u16>,
    // Contains data the CPU *might* read this cycle
    pub cpu_read_data: Cell<u8>,
}

impl PpuRegisters {

    // READ / WRITE FUNCTIONS FOR CPU USE

    /// Takes an address in CPU address space and reads the value of a PPU 
    /// register as a u8. Some registers cannot be read.
    pub fn read(&self, address: u16) -> u8 {
        match address & 0x0007 {
            // PPUCTRL
            0 => 0x00, // Can't read PPUCTRL

            // PPUMASK
            1 => 0x00, // Can't read PPUMASK

            // PPUSTATUS
            2 => {
                let data = self.status().0;
                // Reads from $2002 reset write latch and vblank flag (after the read occurs)
                self.set_write_latch(0);
                self.set_in_vblank(0);

                data
            },

            // OAMADDR
            3 => 0x00, // Can't read OAMADDR

            // OAMDATA
            4 => self.oam_data(),

            // PPUSCROLL
            5 => 0x00, // Can't read PPUSCROLL

            // PPUADDR
            6 => 0x00, // Can't read PPUADDR

            // PPUDATA
            7 => {
                // Reads are too slow for the PPU to respond immediatly, so they
                // go to a read buffer. With the exception (because of course
                // there's an exception) of palette memory, which responds
                // immediatly and still updates the read buffer, discarding the
                // old read buffer data.
                let mut data = self.read_buffer.get();
                self.read_buffer.set(self.cpu_read_data.get()); // this is wrong

                if self.v_val() >= 0x3F00 {
                    data = self.read_buffer.get();
                }

                // if self.status().in_vblank() == 1 {
                //     self.inc_coarse_x();
                //     self.inc_coarse_y();
                // } else {
                // self.set_v_reg(self.v_val() + if self.ctrl().vram_addr_inc() == 0 { 1 } else { 32 });
                // }

                data
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
        // println!("Writing 0x{data:02X} to PPU Register at addr {address}");

        match address & 0x0007 {
            // PPUCTRL
            0 => {
                // t: ...GH.. ........ <- d: ......GH
                //    <used elsewhere> <- d: ABCDEF..
                self.set_ctrl(data);
                self.set_t_nt_select((data & 3) as usize);
            },

            // PPUMASK
            1 => self.set_mask(data),

            // PPUSTATUS
            2 => {}, // Cannot write PPUSTATUS

            // OAMADDR
            3 => self.set_oam_address(data),

            // OAMDATA
            4 => self.set_oam_data(data),

            // PPUSCROLL
            5 => {
                if self.write_latch.get() == 0 {
                    // 1st Write => write to low byte
                    // Update internal regs
                    // t: ....... ...ABCDE <- d: ABCDE...
                    // x:              FGH <- d: .....FGH
                    // w:                  <- 1

                    // NOTE: There is no dedicated PPUSCROLL register separate
                    //       from the v/t registers. The scroll information is
                    //       contained entirely within the v/t registers and the
                    //       fine_x register.
                    self.set_t_coarse_x((data >> 3) as usize);
                    self.set_fine_x(data & 7);

                    self.write_latch.set(1);
                } else {
                    // 2nd Write => Write to high byte
                    // Update internal regs
                    // t: FGH..AB CDE..... <- d: ABCDEFGH
                    // w:                  <- 0
                    self.set_t_coarse_y((data >> 3) as usize);
                    self.set_t_fine_y((data & 7) as usize);

                    self.write_latch.set(0);
                }
            },

            // PPUADDR
            6 => {
                if self.write_latch.get() == 0 {
                    // 1st Write => write to low byte
                    // Update internal regs
                    // t: .CDEFGH ........ <- d: ..CDEFGH
                    //            <unused> <- d: AB......
                    // t: Z...... ........ <- 0 (bit Z is cleared)
                    // w:                  <- 1

                    // NOTE: Only the t register is updated on the 1st write of 
                    //       PPUADDR. Then on the 2nd write, after the high byte
                    //       is also written to, it is copied to the v register.
                    self.set_t_reg((self.t_val() & 0x00FF) | (((data & 0x3F) as u16) << 8));
                    self.write_latch.set(1);
                } else {
                    // 2nd Write => Write to high byte
                    // Update internal regs
                    // t: ....... ABCDEFGH <- d: ABCDEFGH
                    // v: <...all bits...> <- t: <...all bits...>
                    // w:                  <- 0
                    self.set_t_reg((self.t_val() & 0xFF00) | (data as u16));
                    self.set_v_reg(self.t_val());
                    self.write_latch.set(0);
                }
            },

            // PPUDATA
            7 => {
                // println!("Writing 0x{data:02} to PPUDATA");

                self.set_data(data);

                // THIS IS THE +1 PROBLEM!!!
                // The PPU won't actually see this write signal until the next
                // cycle, but we are incrementing v_reg here. Thus the address
                // used by the PPU to write the data is different than the one
                // the CPU expects it to be by either 1 or 32. See the comment
                // above where write_address is declared as a field.
                self.write_ppu_data.set(true);

                self.write_address.set(self.v_val());

                // if self.status().in_vblank() == 1 {
                //     self.inc_coarse_x();
                //     self.inc_coarse_y();
                // } else {
                self.set_v_reg(self.v_val() + if self.ctrl().vram_addr_inc() == 0 { 1 } else { 32 });
                // }
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
    /// Get a copy of the value of the v register when the CPU last wrote data
    /// to PPUDATA (i.e. the address that CPU was trying to write to)
    pub fn address(&self) -> u16 {
        self.write_address.get()
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
    pub fn v_reg(&self) -> PpuScrollReg {
        self.v_reg.get()
    }
    /// Get a copy of the value of the v register as a u16
    pub fn v_val(&self) -> u16 {
        self.v_reg.get().0
    }
    /// Get a copy of the value of the t register as a PpuScrollPos struct
    pub fn t_reg(&self) -> PpuScrollReg {
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
    pub fn write_latch(&self) -> u8 {
        self.write_latch.get()
    }
    /// Get a copy of the flag keeping track of when the CPU writes to PPUDATA
    pub fn write_ppu_data(&self) -> bool {
        self.write_ppu_data.get()
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
        self.v_reg.set(PpuScrollReg(val));
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
        self.v_reg.set(PpuScrollReg((self.v_reg.get().0 & 0xFBFF) | ((val as u16) << 10)));
    }
    /// Set only the nametable y bit (11) of the v register
    pub fn set_v_nt_y(&self, val: usize) {
        self.v_reg.set(PpuScrollReg((self.v_reg.get().0 & 0xF7FF) | ((val as u16) << 11)));
    }
    /// Set the fine y bits (12..=14) of the v register
    pub fn set_v_fine_y(&self, val: usize) {
        self.v_reg.set(self.v_reg.get().with_fine_y(val));
    }

    /// Set the value of the t register
    pub fn set_t_reg(&self, val: u16) {
        self.t_reg.set(PpuScrollReg(val));
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
        self.t_reg.set(PpuScrollReg((self.t_reg.get().0 & 0xFBFF) | ((val as u16) << 10)));
    }
    /// Set only the nametable y bit (11) of the t register
    pub fn set_t_nt_y(&self, val: usize) {
        self.t_reg.set(PpuScrollReg((self.t_reg.get().0 & 0xF7FF) | ((val as u16) << 11)));
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
    /// Set the value of the write ppu data flag
    pub fn set_write_ppu_data(&self, val: bool) {
        self.write_ppu_data.set(val);
    }

    // PPU RENDERING RENDERING HELPERS

    /// Increment the coarse x value in the v register. Also handles wrap around
    /// cases when the value of coarse x overflows. For more details, visit
    /// https://www.nesdev.org/wiki/PPU_scrolling#Wrapping_around
    pub fn inc_coarse_x(&self) {
        let mut v = self.v_val();

        // This occurs when the coarse_x is crossing into the next nametable
        if (v & 0x001F) == 0x1F { // if coarse x would wrap on add
            v &= !0x001F;  // coarse X = 0 (wrap)
            v ^= 0x0400;   // switch horizontal nametable
        } else {
            v += 1;  // increment coarse X
        }

        self.set_v_reg(v);
    }

    /// Increment the coarse y value in the v register. Also handles wrap around
    /// cases when the value of coarse y overflows. For more details, visit
    /// https://www.nesdev.org/wiki/PPU_scrolling#Wrapping_around
    pub fn inc_coarse_y(&self) {
        let mut v = self.v_val();

        if (v & 0x7000) != 0x7000 {
            v += 0x1000;
        } else {
            v &= !0x7000;
            let mut y = (v & 0x03E0) >> 5;
            if y == 0x1D {
                y = 0;
                v ^= 0x0800;
            } else if y == 0x1F {
                y = 0;
            } else {
                y += 1;
            }
            v = (v & !0x03E0) | (y << 5);
        }

        self.set_v_reg(v);
    }

    /// Transfer horizontal data (coarse x and nametable x) from the t register
    /// to the v register in preperation for the rendering phase. 
    pub fn transfer_x_data(&self) {
        let t_reg = self.t_reg();

        self.set_v_coarse_x(t_reg.coarse_x());
        self.set_v_nt_x(t_reg.nt_x());
    }

    /// Transfer vertical data (coarse y, nametable y, and fine y) from the t 
    /// register to the v register in preperation for the rendering phase.
    pub fn transfer_y_data(&self) {
        let t_reg = self.t_reg();

        self.set_v_coarse_y(t_reg.coarse_y());
        self.set_v_nt_y(t_reg.nt_y());
        self.set_v_fine_y(t_reg.fine_y());
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
    _unused: bool,
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