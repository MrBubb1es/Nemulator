use std::rc::Rc;

use crate::cartridge::Mapper;

use super::{mem::Memory, nes_graphics::{NesColor, DEFAULT_PALETTE}, ppu_util::PpuRegisters};

const PRIMARY_OAM_SIZE: usize = 256;
const SECONDARY_OAM_SIZE: usize = 32;

/// Representation of the NES Picture Processing Unit. Details on how the PPU
/// works can be found here: https://www.nesdev.org/wiki/PPU_registers
pub struct Ppu2C02 {
    // To keep track of scanline rendering
    dot: usize,
    scanline: usize,

    vram: Memory,
    palette_mem: Memory,
    chr_rom: Memory,
    registers: Rc<PpuRegisters>,
    primary_oam: Memory,
    secondary_oam: Memory,
    mapper: Rc<dyn Mapper>,

    // Information for the next background tile to render. These essensially work
    // as data buffers for the next data to be read into the shift registers.
    bg_next_tile_nt_addr: u8,
    bg_next_tile_attrib: u8,
    bg_next_tile_lsb: u8,
    bg_next_tile_msb: u8,

    // Data for this bg tile and the next bg tile in 16 bit shift registers
    bg_tile_nt_hi: u16,
    bg_tile_nt_lo: u16,
    bg_tile_attrib_hi: u16,
    bg_tile_attrib_lo: u16,

    /// Pagetable 1 & 2 memory (for debugging purposes)
    pub pgtbl1: Box<[u8; 0x1000]>,
    pub pgtbl2: Box<[u8; 0x1000]>,

    // flag set when the ppu finishes rendering a frame
    frame_finished: bool,
}

// Main functionality
impl Ppu2C02 {
    /// Create a new PPU
    ///  * `chr_rom` - Character ROM read in from cartridge.
    ///  * `ppu_regs` - Pointer to this PPUs registers. This allows both the PPU
    ///                 and CPU to access the PPU registers, as the CPU needs to
    ///                 read and write to some of them.
    ///  * `mapper` - Pointer the the mapper being used by the cartridge.
    pub fn new(chr_rom: Memory, ppu_regs: Rc<PpuRegisters>, mapper: Rc<dyn Mapper>) -> Self {
        let mut ppu = Ppu2C02 {
            dot: 0,
            scanline: 0,
            vram: Memory::new(0x800), // 2KiB ppu ram
            palette_mem: Memory::new(0x20),
            chr_rom: chr_rom,
            registers: Rc::clone(&ppu_regs),
            primary_oam: Memory::new(PRIMARY_OAM_SIZE),
            secondary_oam: Memory::new(SECONDARY_OAM_SIZE),
            mapper: Rc::clone(&mapper),

            bg_next_tile_nt_addr: 0,
            bg_next_tile_attrib: 0,
            bg_next_tile_lsb: 0,
            bg_next_tile_msb: 0,

            bg_tile_nt_hi: 0,
            bg_tile_nt_lo: 0,
            bg_tile_attrib_hi: 0,
            bg_tile_attrib_lo: 0,

            pgtbl1: Box::new([0; 0x1000]),
            pgtbl2: Box::new([0; 0x1000]),

            frame_finished: false,
        };

        // Read pagetable memories into arrays for debug view
        for i in 0..0x1000 {
            ppu.pgtbl1[i as usize] = ppu.read(i);
            ppu.pgtbl2[i as usize] = ppu.read(i | 0x1000);
        }

        // let data = 0x2C;
        // for addr in 0x2000..=0x3EFF {
        //     ppu.write(addr, data);

        //     assert_eq!(ppu.read(addr), data);
        // }

        // for addr in 0x3F00..=0x3FFF {
        //     ppu.write(addr, addr as u8 & 0x1F);
        // }

        ppu
    }

    /// Cycle the PPU through the rendering/execution of a single pixel/dot.
    pub fn cycle(&mut self, frame: &mut [u8]) {
        self.frame_finished = false;

        if self.registers.write_ppu_data() {
            self.write(self.registers.v_val(), self.registers.data());
            self.registers.set_write_ppu_data(false);
        }

        match self.scanline {
            0..=239 => { // Visible cycles
                self.visible_scanline_cycle();
            }
            240 => {}, // Idle scanline (technically the start of vblank, but 
                       // the vblank flag isn't set until dot 1 of scanline 241)
            241 => { // Start of vblank
                if self.dot == 1 {
                    self.registers.set_in_vblank(1);
                
                    if self.registers.ctrl().vblank_nmi() == 1 {
                        self.trigger_nmi();
                    }
                }
            }
            242..=260 => {}, // Idle cycles
            261 => { // Pre-Render scanline
                if self.dot == 1 {
                    self.registers.set_in_vblank(0); // end of vblank
                }

                self.visible_scanline_cycle(); // even though nothing is rendered
                                               // during this scanline, the ppu
                                               // still performs all of the same
                                               // memory reads as on a visible 
                                               // scanline.
                
                if self.dot >= 280 && self.dot <= 304 {
                    self.transfer_y_data();
                }
            },
            _ => {},
        }

        if self.scanline < 240 && self.dot < 256 {
            self.draw_dot(frame);
        }

        self.dot += 1;
        if self.dot > 340 {
            self.dot = 0;

            self.scanline += 1;
            if self.scanline > 261 {
                self.scanline = 0;
                self.frame_finished = true;
            }
        }
    }
    
    /// Helper function to handle the memory accesses/internal register changes
    /// that occur during a visible scanline (and the pre-render scanline).
    fn visible_scanline_cycle(&mut self) {
        // NEED TO HANDLE ODD FRAME SKIP

        match self.dot {
            0 => {}, // Idle dot
            2..=255 | 321..=337 => {
                self.update_shift_regs();

                // Update internal buffers & registers
                match (self.dot - 1) & 7 {
                    0 => { 
                        self.load_shift_regs();
                        self.load_nt_buffer(); 
                    },
                    2 => { self.load_attrib_buffer(); },
                    4 => { self.load_bg_lsb_buffer(); },
                    6 => { self.load_bg_msb_buffer(); },
                    7 => { self.inc_coarse_x(); },
                    _ => {},
                }
            },
            256 => {
                self.update_shift_regs();
                self.inc_coarse_x();
                self.inc_coarse_y();
            },
            257 => {
                self.load_shift_regs();
                self.transfer_x_data();  
            },
            338 | 340 => {
                self.load_nt_buffer();
            },
            _ => {}, // All other cycles are idle
        }
    }

}

// Internal & Helper functionality
impl Ppu2C02 {
    /// Put the background buffer registers into the least significant byte of the shift registers
    fn load_shift_regs(&mut self) {
        self.bg_tile_nt_hi = (self.bg_tile_nt_hi & 0xFF00) | self.bg_next_tile_msb as u16;
        self.bg_tile_nt_lo = (self.bg_tile_nt_lo & 0xFF00) | self.bg_next_tile_lsb as u16;
        self.bg_tile_attrib_hi = (self.bg_tile_attrib_hi & 0xFF00) | (0xFF * (((self.bg_next_tile_attrib >> 1) & 1) as u16));
        self.bg_tile_attrib_lo = (self.bg_tile_attrib_lo & 0xFF00) | (0xFF * (((self.bg_next_tile_attrib >> 0) & 1) as u16));
    }
    /// Shift all of the shift registers to the left by one bit
    fn update_shift_regs(&mut self) {
        // If bg rendering enabled
        if self.registers.mask().draw_bg() == 1 {
            self.bg_tile_nt_hi <<= 1;
            self.bg_tile_nt_lo <<= 1;
            self.bg_tile_attrib_hi <<= 1;
            self.bg_tile_attrib_lo <<= 1;
        }
    }

    // HELPER FUNCTIONS

    /// Takes in a 2 bit palette value and 2 bit pixel value and returns the
    /// color of the pixel as a NesColor
    fn color_from_tile_data(&self, palette: u16, pixel: u16) -> NesColor {
        DEFAULT_PALETTE[self.read(0x3F00 | (palette << 2) | pixel) as usize & 0x3F]
    }

    /// PPU reads a single byte from a given address. The ram/rom accessed 
    /// depends on the address.
    ///
    /// 0x0000-0x1FFF: Cartridge CHR ROM, mapper deals with this
    ///
    /// 0x2000-0x2FFF: VRAM, mapper may rerout this to CHR ROM or somewhere in vram
    ///
    /// 0x3000-0x3EFF: VRAM (Mirror of 0x2000-0x2EFF)
    ///
    /// 0x3F00-0x3FFF: palette, mapper cannot change address
    ///
    ///  * `address` - 16 bit address used to access data
    pub fn read(&self, address: u16) -> u8 {
        let mapped_addr = self.mapper
            .get_ppu_read_addr(address)
            .unwrap_or(address);
        
        match mapped_addr & 0x3FFF {
            0x0000..=0x1FFF => {
                self.chr_rom.read(mapped_addr)
            },
            0x2000..=0x3EFF => {
                self.vram.read(mapped_addr & 0x7FF)
            },
            0x3F00..=0x3FFF => {
                let mut data = if mapped_addr & 3 == 0 {
                    self.palette_mem.read(address & 0x0C) // Does all of the mapping above
                } else {
                    self.palette_mem.read(address & 0x1F)
                };

                if self.registers.mask().greyscale() == 1 {
                    data &= 0x30;
                } else {
                    data &= 0x3F;
                }

                // println!("Reading data 0x{data:02X} from pal mem w/ addr 0x{mapped_addr:02X}");

                data
            },
            _ => {unreachable!("By Becquerel's Ghost!");}
        }
    }

    /// PPU writes a single byte of data to a given address. The ram accessed 
    /// depends on the address.
    ///
    /// 0x0000-0x1FFF: Cartridge CHR ROM
    ///
    /// 0x2000-0x3EFF: VRAM
    ///
    /// 0x3F00-0x3FFF: palette
    ///
    ///  * `address` - 16 bit address used to access data
    ///  * `data` - Single byte of data to write
    pub fn write(&self, address: u16, data: u8) {
        let mapped_addr = self.mapper
            .get_ppu_write_addr(address, data)
            .unwrap_or(address);

        match mapped_addr & 0x3FFF {
            0x0000..=0x1FFF => {
                self.chr_rom.write(mapped_addr, data);
            },
            0x2000..=0x3EFF => {
                self.vram.write(mapped_addr & 0x7FF, data);
            },
            0x3F00..=0x3FFF => {
                println!("Writing to pal mem w/ addr 0x{mapped_addr:02X} & data: 0x{data:02X}");

                if mapped_addr & 3 == 0 {
                    self.palette_mem.write(address & 0x0C, data); // Does all of the mapping needed
                } else {
                    self.palette_mem.write(address & 0x1F, data);
                };
            },
            _ => {unreachable!("I never thought I'd live to see a Resonance Cascade, let alone create one...");}
        }
    }


    /// Increment the coarse x value in the v register. Also handles wrap around
    /// cases when the value of coarse x overflows. For more details, visit
    /// https://www.nesdev.org/wiki/PPU_scrolling#Wrapping_around
    fn inc_coarse_x(&self) {
        if !self.rendering_enabled() {
            return;
        }

        let mut v = self.registers.v_val();

        // This occurs when the coarse_x is crossing into the next nametable
        if (v & 0x001F) == 0x1F { // if coarse x would wrap on add
            v &= !0x001F;  // coarse X = 0 (wrap)
            v ^= 0x0400;   // switch horizontal nametable
        } else {
            v += 1;  // increment coarse X
        }

        self.registers.set_v_reg(v);
    }

    /// Increment the coarse y value in the v register. Also handles wrap around
    /// cases when the value of coarse y overflows. For more details, visit
    /// https://www.nesdev.org/wiki/PPU_scrolling#Wrapping_around
    fn inc_coarse_y(&self) {
        if !self.rendering_enabled() {
            return;
        }

        let mut v = self.registers.v_val();

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

        self.registers.set_v_reg(v);
    }

    /// Transfer horizontal data (coarse x and nametable x) from the t register
    /// to the v register in preperation for the rendering phase. 
    fn transfer_x_data(&self) {
        if !self.rendering_enabled() {
            return;
        }

        let t_reg = self.registers.t_reg();

        self.registers.set_v_coarse_x(t_reg.coarse_x());
        self.registers.set_v_nt_x(t_reg.nt_x());
    }

    /// Transfer vertical data (coarse y, nametable y, and fine y) from the t 
    /// register to the v register in preperation for the rendering phase.
    fn transfer_y_data(&self) {
        if !self.rendering_enabled() {
            return;
        }

        let t_reg = self.registers.t_reg();

        self.registers.set_v_coarse_y(t_reg.coarse_y());
        self.registers.set_v_nt_y(t_reg.nt_y());
        self.registers.set_v_fine_y(t_reg.fine_y());
    }

    /// Fetches the next byte of background tile nametable ids and stores it in
    /// an internal buffer.
    fn load_nt_buffer(&mut self) {
        self.bg_next_tile_nt_addr = self.read(0x2000 | (self.registers.v_val() & 0x0FFF));
    }
    /// Fetches the next byte of background tile attributes and stores it in
    /// an internal buffer.
    fn load_attrib_buffer(&mut self) {
        let v_reg = self.registers.v_reg();
        // Read byte containing 4 tile attribute values from mem
        self.bg_next_tile_attrib = self.read((0x23C0 
                                            | (v_reg.nt_y() << 11) 
                                            | ((v_reg.coarse_y() >> 2) << 3) 
                                            | (v_reg.coarse_x() >> 2)) as u16);
        
        // Shift to obtain the correct tile attribute value
        if v_reg.coarse_y() & 0x02 == 1 {
            self.bg_next_tile_attrib >>= 4;
        }

        if v_reg.coarse_x() & 0x02 == 1 {
            self.bg_next_tile_attrib >>= 2;
        }

        self.bg_next_tile_attrib &= 3;
    }
    /// Fetches the next low byte of background tile pixel data and stores it
    /// in an internal buffer.
    fn load_bg_lsb_buffer(&mut self) {
        let bg_ptrn = self.registers.ctrl().bg_pattern_tbl();
        let fine_y = self.registers.v_reg().fine_y();

        self.bg_next_tile_lsb = self.read(
            ((bg_ptrn as u16) << 12) // 0x0000 or 0x1000
            | ((self.bg_next_tile_nt_addr as u16) << 4)
            | (fine_y as u16 + 0));
    }
    /// Fetches the next high byte of background tile pixel data and stores it
    /// in an internal buffer.
    fn load_bg_msb_buffer(&mut self) {
        let bg_ptrn = self.registers.ctrl().bg_pattern_tbl();
        let fine_y = self.registers.v_reg().fine_y();

        self.bg_next_tile_lsb = self.read(
            ((bg_ptrn as u16) << 12) // 0x0000 or 0x1000
            | ((self.bg_next_tile_nt_addr as u16) << 4)
            | (fine_y as u16 + 8));
    }

    /// Trigger a NMI within the CPU
    fn trigger_nmi(&self) {
        // TODO: write code here
    }

    /// Draw to the given frame buffer at the current scanline and dot. This function
    /// does not internally check to ensure the scanline and dot are within the bounds
    /// of the screen.
    fn draw_dot(&self, frame: &mut [u8]) {
        let mut bg_pix = 0;
        let mut bg_pal = 0;

        if self.registers.mask().draw_bg() == 1 {
            let bg_pix_hi = self.bg_tile_nt_hi >> (15 - self.registers.fine_x());
            let bg_pix_lo = self.bg_tile_nt_lo >> (15 - self.registers.fine_x());

            let bg_pal_hi = self.bg_tile_attrib_hi >> (15 - self.registers.fine_x());
            let bg_pal_lo = self.bg_tile_attrib_lo >> (15 - self.registers.fine_x());

            bg_pix = (bg_pix_hi << 1) | bg_pix_lo;
            bg_pal = (bg_pal_hi << 1) | bg_pal_lo;
        }

        // dbg!(bg_pal, bg_pix);

        let col = self.color_from_tile_data(bg_pal, bg_pix);
        let pix_idx = (self.scanline * 256 + self.dot)*4;

        frame[pix_idx + 0] = col.r;
        frame[pix_idx + 1] = col.g;
        frame[pix_idx + 2] = col.b;
        frame[pix_idx + 3] = 0xFF;
    }
}

// Public functionality
impl Ppu2C02 {

    // GETTER / SETTER FUNCTIONS

    /// Get the current dot of the PPU
    pub fn get_dot(&self) -> usize {
        self.dot
    }
    pub fn get_scanline(&self) -> usize {
        self.scanline
    }
    /// Get the current state of the frame finished flag
    pub fn frame_finished(&self) -> bool {
        self.frame_finished
    }
    /// Return a bool reporting whether either sprites or background tiles are
    /// currently being rendered by the PPU.
    pub fn rendering_enabled(&self) -> bool {
        self.registers.mask().draw_bg() == 1 || self.registers.mask().draw_sprites() == 1
    }

    /// Set the value of the frame finished flag
    pub fn set_frame_finished(&mut self, val: bool) {
        self.frame_finished = val;
    }
}