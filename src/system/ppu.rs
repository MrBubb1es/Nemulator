use std::cell::Cell;
use std::rc::Rc;

use bitfield_struct::bitfield;

use crate::cartridge::cartridge::Cartridge;
use crate::cartridge::mapper::Mapper;

use super::mem::Memory;
use super::nes_graphics;

// NOTE: We may be misunderstanding scanline rendering behavior, in which case these constants may
// be off by one.
const SCANWIDTH: usize = 340;
const SCANLINES: usize = 262;
const PRIMARY_OAM_SIZE: usize = 256;
const SECONDARY_OAM_SIZE: usize = 32;

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
struct PpuCtrl {
    #[bits(2)]
    nametable: usize,
    #[bits(1)]
    vram_addr_inc: usize,
    #[bits(1)]
    spr_pattern_tbl: usize,
    #[bits(1)]
    bg_pattern_tbl: usize,
    #[bits(1)]
    spr_size: usize,
    #[bits(1)]
    mstr_slave: usize,
    #[bits(1)]
    create_nmi: usize,
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
struct PpuMask {
    #[bits(1)]
    greyscale: usize,
    #[bits(1)]
    show_bg_left: usize,
    #[bits(1)]
    show_spr_left: usize,
    #[bits(1)]
    show_background: usize,
    #[bits(1)]
    show_sprite: usize,
    #[bits(1)]
    emph_red: usize,
    #[bits(1)]
    emph_grn: usize,
    #[bits(1)]
    emph_blu: usize,
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
struct PpuStatus {
    #[bits(5)]
    open_bus: usize,
    #[bits(1)]
    spr_overflow: usize,
    #[bits(1)]
    spr_0_hit: usize,
    #[bits(1)]
    in_vblank: usize,
}

#[derive(Default)]
pub struct PpuRegisters {
    pub ppu_ctrl: Cell<PpuCtrl>,
    pub ppu_mask: Cell<PpuMask>,
    pub ppu_status: Cell<PpuStatus>,
    pub oam_address: Cell<u8>,
    pub oam_data: Cell<u8>,
    pub ppu_scroll: Cell<u16>,
    pub ppu_address: Cell<u16>,
    pub ppu_data: Cell<u8>,

    pub oam_dma: Cell<u8>,

    // Current VRAM Address (15 least significant bits)
    v: Cell<u16>,
    // Temporary VRAM Address (15 least significant bits)
    t: Cell<u16>,
    // Fine X scroll (3 least significant bits)
    x: Cell<u8>,
    // First or second write toggle (least significant bit)
    w: Cell<u8>,
}

impl PpuRegisters {
    /// Read the value of a PPU Register as a u8
    pub fn read(&self, address: u16) -> u8 {
        match address & 0x0007 {
            0 => 0x00, // Can't read PPUCTRL
            1 => 0x00, // Can't read PPUMASK
            2 => self.ppu_status.get().0,
            3 => 0x00, // Can't read OAMADDR
            4 => self.oam_data.get(),
            5 => 0x00, // Can't read PPUSCROLL
            6 => 0x00, // Can't read PPUADDR
            7 => self.ppu_data.get(),
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
            0 => self.ppu_ctrl.set(PpuCtrl::from_bits(data)),
            1 => self.ppu_mask.set(PpuMask::from_bits(data)),
            2 => {}, // Cannot write PPUSTATUS
            3 => self.oam_address.set(data),
            4 => self.oam_data.set(data),
            5 => {
                if self.w.get() == 0 {
                    // 1st Write => write to low byte
                    self.ppu_scroll.set((self.ppu_scroll.get() & 0xFF00) | data as u16);
                } else {
                    // 2nd Write => Write to high byte
                    self.ppu_scroll.set((self.ppu_scroll.get() & 0x00FF) | ((data as u16) << 8));
                }
            },
            6 => {
                if self.w.get() == 0 {
                    // 1st Write => write to low byte
                    self.ppu_address.set((self.ppu_address.get() & 0xFF00) | data as u16);
                } else {
                    // 2nd Write => Write to high byte
                    self.ppu_address.set((self.ppu_address.get() & 0x00FF) | ((data as u16) << 8));
                }
            },
            7 => self.ppu_data.set(data),
            _ => unreachable!("Well done. Here are the test results: \"You are a horrible person.\" I'm serious, that's what it says: \"A horrible person.\" We weren't even testing for that.")
        };
    }
}

/// Representation of the NES Picture Processing Unit. Details on how the PPU
/// works can be found here: https://www.nesdev.org/wiki/PPU_registers
pub struct PPU {
    // To keep track of scanline rendering
    dot: usize,
    scanline: usize,

    // Registers
    vram: Memory,
    chr_rom: Memory,
    registers: Rc<PpuRegisters>,
    primary_oam: Memory,
    secondary_oam: Memory,
    mapper: Rc<dyn Mapper>,

    palette: nes_graphics::NESPalette,
}

impl PPU {
    /// Create a new PPU
    ///  * `cart` - The cartridge to attatch to the PPU bus
    pub fn new(chr_rom: Memory, ppu_regs: Rc<PpuRegisters>, mapper: Rc<dyn Mapper>) -> Self {
        PPU {
            dot: 0,
            scanline: 0,
            vram: Memory::new(0x800), // 2KiB ppu ram
            chr_rom: chr_rom,
            registers: Rc::clone(&ppu_regs),
            primary_oam: Memory::new(PRIMARY_OAM_SIZE),
            secondary_oam: Memory::new(SECONDARY_OAM_SIZE),
            mapper: Rc::clone(&mapper),

            palette: nes_graphics::DEFAULT_PALETTE,
        }
    }

    // GETTER / SETTER FUNCTIONS

    /// Reads a single byte from a given address. The ram/rom accessed depends
    /// on the address.
    ///
    /// 0x0000-0x1FFF: Cartridge CHR ROM
    ///
    /// 0x2000-0x2FFF: VRAM
    ///
    /// 0x3000-0x3EFF: VRAM (Mirror of 0x2000-0x2EFF)
    ///
    /// 0x3F00-0x3FFF: palette
    ///
    ///  * `address` - 16 bit address used to access data
    pub fn read(&self, address: u16) -> u8 {
        let mapped_address = self.mapper.get_ppu_read_addr(address);
        let mapped_addr = mapped_address.unwrap_or(address);

        match mapped_addr {
            0x0000..=0x1FFF => self.chr_rom.read(mapped_addr),
            0x2000..=0x2FFF => self.vram.read(mapped_addr),
            0x3000..=0x3EFF => self.vram.read(mapped_addr - 0x1000),
            0x3F00..=0x3FFF => self.read_palette(address & 0x00FF),
            _ => 0xEE,
        }
    }

    fn read_palette(&self, address: u16) -> u8 {
        0
    }

    /// Write a single byte of data to a given address. The ram accessed depends
    /// on the address.
    ///
    /// 0x0000-0x1FFF: Cartridge CHR ROM
    ///
    /// 0x2000-0x3EFF: VRAM
    ///
    /// 0x3F00-0x3FFF: palettee
    ///
    ///  * `address` - 16 bit address used to access data
    pub fn write(&self, address: u16, data: u8) {
        let mapped_address = self.mapper.get_ppu_write_addr(address, data);
        let mapped_addr = mapped_address.unwrap_or(address);

        match mapped_addr {
            0x0000..=0x1FFF => {
                self.chr_rom.write(mapped_addr, data);
            }
            0x2000..=0x2FFF => {
                self.vram.write(mapped_addr, data);
            }
            0x3000..=0x3EFF => {
                self.vram.write(mapped_addr - 0x1000, data);
            }
            0x3F00..=0x3FFF => self.write_palette(address & 0x001F, data),
            _ => {}
        }
    }

    fn write_palette(&self, address: u16, data: u8) {}

    pub fn render_to_arr(dest: &mut [u8; 256 * 240 * 3]) {}

    pub fn cycle(&mut self) {
        self.dot += 1;
        if self.dot > SCANWIDTH {
            self.scanline += 1;
        }

        if self.scanline > SCANLINES {
            self.vblank_begin();
        }
    }

    fn vblank_begin(&mut self) {
        // TODO: write code here
    }

    /// Called every scanline.
    /// Performs sprite evaluation as detailed here:
    /// https://www.nesdev.org/wiki/PPU_sprite_evaluation. Scans through primary OAM to determine
    /// which sprites to draw, and stores those sprites (normally up to 8) in the secondary OAM.
    /// There are quirks about what happens when there are more than 8 sprites in a scanline - see
    /// the link for details.
    fn sprite_evaluation(&mut self) {
        // Part 1: cycles 1-64
        for n in 0..SECONDARY_OAM_SIZE {
            self.secondary_oam.write(n as u16, 0xFF); // The default, meaning "no sprite".
        }

        // Part 2: cycles 65-256
        let mut secondary_full = false;
        let mut sprites_found: usize = 0;
        for n in 0..64 {
            let y_coord = self.primary_oam.read(n * 4); // First byte of the nth sprite
            if !secondary_full {
                self.secondary_oam
                    .write((sprites_found * 4) as u16, y_coord);

                // Check if y coordinate is within range of this scanline.
                let y_diff = self.scanline as isize - y_coord as isize;
                if y_diff >= 0 && y_diff < 8 {
                    // TODO: Ensure this works for 8-pixel and 16-pixel.
                    sprites_found += 1;
                    if sprites_found == 8 {
                        secondary_full = true;
                    }
                    // Copy the rest of the sprite data to secondary OAM.
                    self.secondary_oam.write(
                        (sprites_found * 4 + 1) as u16,
                        self.primary_oam.read(n * 4 + 1),
                    );
                    self.secondary_oam.write(
                        (sprites_found * 4 + 2) as u16,
                        self.primary_oam.read(n * 4 + 2),
                    );
                    self.secondary_oam.write(
                        (sprites_found * 4 + 3) as u16,
                        self.primary_oam.read(n * 4 + 3),
                    );
                }
            }
        }
        // Part 3: cycles 257-320
        //   ¯\_(ツ)_/¯

        // Part 4: cycles 321-340
        //   ¯\_(ツ)_/¯
    }

    /// Renders a single scanline. Returns the rendered line as a vector of bytes corresponding to
    /// NES palette color codes (see https://www.nesdev.org/wiki/PPU_palettes for details), each
    /// byte corresponding to a single pixel to be rendered.
    fn render_scanline(&mut self) -> Vec<u8> {
        let mut rendered_line: Vec<u8> = Vec::new();
        for v in 0..33 {
            // Fetch the nametable and attribute table entry for the next background sprite.
            let nt_entry = self.read((0x2000 + v + (self.scanline / 8) * 32) as u16);
            let at_entry = self.read(((0x23C0 + (v & 0xFC)) >> (2 * (v & 3))) as u16); // Trust, bro.

            // Use nametable entry to fetch background sprite data.
            let pt_address = nt_entry as u16 * 16 + (self.scanline as u16) & 7;
            let pt_entry_lo = self.read(pt_address);
            let pt_entry_hi = self.read(pt_address + 8);

            // TODO: Sprite evaluation. For now we just render the background.
            for i in 0..8 {
                let palette_lookup = at_entry << 2
                    | ((pt_entry_hi >> (7 - i)) & 1) << 1
                    | (pt_entry_lo >> (7 - i)) & 1;
                if palette_lookup & 0x11 == 0 {
                    rendered_line.push(self.read(0x3F00)); // transparent pixel, render bg color
                } else {
                    rendered_line.push(self.read(0x3F00 | palette_lookup as u16));
                    // get bg sprite color
                }
            }
        }

        rendered_line
    }
}
