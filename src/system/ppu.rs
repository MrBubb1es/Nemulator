/*
The double increment problem is caused by the following:
    During CPU cycles, instructions go through a fetch, decode, and execute
    stage. During the decode stage, we read the address that the instruction
    will access. The instruction itself may want to read or write to that
    address, we don't know during the decode stage. For convinience, we always
    read the data that the CPU might use during the execute stage. This means
    that if we have a writing instruction, such as STA, the data is read from
    the address but ultimatly is discarded. If the CPU were to execute an
    instrution like "STA $2007", PPUADDR would be read, even though the value
    would be discarded. This would cause two problems:
        1) The internal write latch inside the PPU would be left in the wrong state.
        2) The internal v register would be incremented 2 times. First for the 
           read from $2007 during the execute stage, and second for the write
           done during the instruction execution phase. 
*/

use std::rc::Rc;

use crate::cartridge::{mapper::NametableMirror, Mapper};

use super::{mem::Memory, nes_graphics::{NesColor, DEFAULT_PALETTE}, ppu_util::{PpuCtrl, PpuMask, PpuScrollReg, PpuStatus}};

const PRIMARY_OAM_SIZE: usize = 256;
const SECONDARY_OAM_SIZE: usize = 32;

/// Representation of the NES Picture Processing Unit. Details on how the PPU
/// works can be found here: https://www.nesdev.org/wiki/PPU_registers
pub struct Ppu2C02 {
    // To keep track of scanline rendering
    dot: usize,
    scanline: usize,

    // Internal PPU registers (the CPU is able to affect these through reads
    // and writes through $2000-$3FFF in CPU addressing space)
    ctrl: PpuCtrl,
    mask: PpuMask,
    status: PpuStatus,
    oam_address: u8,
    oam_data: u8,

    oam_dma: u8,

    // Current VRAM Address (15 least significant bits)
    v_reg: PpuScrollReg,
    // Temporary VRAM Address (15 least significant bits)
    t_reg: PpuScrollReg,
    // Fine X scroll (3 least significant bits)
    fine_x: u8,
    // CPU reads from the PPU are delayed by 1 read, instead placing the read
    // data into a read buffer to be returned next time the CPU reads data
    read_buffer: u8,
    // First or second write toggle (least significant bit)
    write_latch: u8,


    // Memories accessable only by the PPU
    vram: Memory,
    palette_mem: Memory,
    chr_rom: Memory,
    primary_oam: Memory,
    secondary_oam: Memory,

    // Mapper used by the cartridge, Rc because both the CPU and PPU access to it
    mapper: Rc<dyn Mapper>,

    // Flag to signal the NES to trigger a CPU NMI
    cpu_nmi_flag: bool,

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

    /// Frame buffer used to render NES screen
    screen_buf: Box<[u8; 256*240*4]>,

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
    pub fn new(chr_rom: Memory, mapper: Rc<dyn Mapper>) -> Self {
        let mut ppu = Ppu2C02 {
            dot: 0,
            scanline: 0,

            cpu_nmi_flag: false,

            ctrl: 0.into(),
            mask: 0.into(),
            status: 0.into(),
            oam_address: 0,
            oam_data: 0,
            oam_dma: 0,

            v_reg: 0.into(),
            t_reg: 0.into(),
            fine_x: 0,
            read_buffer: 0,
            write_latch: 0,

            vram: Memory::new(0x800), // 2KiB ppu ram
            palette_mem: Memory::new(0x20),
            chr_rom: chr_rom,
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

            screen_buf: Box::new([0; 256*240*4]),

            frame_finished: false,
        };

        // Read pagetable memories into arrays for debug view
        for i in 0..0x1000 {
            ppu.pgtbl1[i as usize] = ppu.ppu_read(i);
            ppu.pgtbl2[i as usize] = ppu.ppu_read(i | 0x1000);
        }


        // for addr in 0x2000..=0x3EFF {
        //     ppu.write(addr, addr as u8);

        //     assert_eq!(ppu.read(addr), addr as u8);
        // }

        // for addr in 0x3F00..=0x3FFF {
        //     ppu.write(addr, addr as u8 & 0x1F);
        // }

        ppu
    }

    /// Cycle the PPU through the rendering/execution of a single pixel/dot.
    pub fn cycle(&mut self) {        
        self.frame_finished = false;

        match self.scanline {
            0..=239 => { // Visible cycles
                self.visible_scanline_cycle();
            }
            240 => {}, // Idle scanline (technically the start of vblank, but 
                       // the vblank flag isn't set until dot 1 of scanline 241)
            241 => { // Start of vblank
                if self.dot == 1 {
                    self.status.set_in_vblank(1);
                
                    if self.ctrl.vblank_nmi() == 1 {
                        self.trigger_nmi();
                    }
                }
            }
            242..=260 => {}, // Idle cycles
            261 => { // Pre-Render scanline
                if self.dot == 1 {
                    self.status.set_in_vblank(0); // end of vblank
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

        if self.scanline < 240 && 0 < self.dot && self.dot <= 256 {
            self.draw_dot();
        }

        self.dot += 1;
        if self.dot > 340 {
            self.dot = 0;

            self.scanline += 1;
            if self.scanline > 261 {
                self.scanline = 0;
                self.frame_finished = true;

                // for addr in 0x3F00..0x3FFF {
                //     println!("${addr:04X}: 0x{:02X}", self.read(addr));
                // }
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
                self.update_shift_regs();
                self.load_nt_buffer(); // fixed: wasn't loading nt buffer here
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
        self.bg_tile_attrib_hi = (self.bg_tile_attrib_hi & 0xFF00) | (if self.bg_next_tile_attrib & 2 == 0 { 0x00 } else { 0xFF });
        self.bg_tile_attrib_lo = (self.bg_tile_attrib_lo & 0xFF00) | (if self.bg_next_tile_attrib & 1 == 0 { 0x00 } else { 0xFF });
    }
    /// Shift all of the shift registers to the left by one bit
    fn update_shift_regs(&mut self) {
        // If bg rendering enabled
        if self.mask.draw_bg() == 1 {
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
        DEFAULT_PALETTE[self.ppu_read(0x3F00 | (palette << 2) | pixel) as usize & 0x3F]
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
    pub fn ppu_read(&self, address: u16) -> u8 {
        let mapped_addr = self.mapper
            .get_ppu_read_addr(address)
            .unwrap_or(address);
        
        match mapped_addr & 0x3FFF {
            0x0000..=0x1FFF => {
                self.chr_rom.read(mapped_addr & 0x1FFF)
            },
            0x2000..=0x3EFF => {
                let mirrored_addr1 = mapped_addr & 0x2FFF; // mirror $3XXX addresses to $2XXX addresses
                let mirrored_addr2 = match self.mapper.get_nt_mirror_type() {
                    NametableMirror::Horizontal => { mirrored_addr1 & 0x07FF },
                    NametableMirror::Vertical => { (mirrored_addr1 & 0x03FF) | (if mirrored_addr1 > 0x2800 { 0x400 } else { 0 }) }
                    // NametableMirror::Horizontal => { (mirrored_addr1 & 0x03FF) | (if mirrored_addr1 > 0x2800 { 0x400 } else { 0 }) },
                    // NametableMirror::Vertical => { mirrored_addr1 & 0x07FF },
                };
                self.vram.read(mirrored_addr2)
            },
            0x3F00..=0x3FFF => {
                let mirrored_addr = mapped_addr & 0x1F;
                
                let mut data = self.palette_mem.read(mirrored_addr);

                if self.mask.greyscale() == 1 {
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
    pub fn ppu_write(&self, address: u16, data: u8) {
        // println!("Writing to ppu w/ addr 0x{address:02X} and data: 0x{data:02X}");
        
        let mapped_addr = self.mapper
            .get_ppu_write_addr(address, data)
            .unwrap_or(address);

        match mapped_addr & 0x3FFF {
            0x0000..=0x1FFF => {
                self.chr_rom.write(mapped_addr, data);
            },
            0x2000..=0x3EFF => {
                let mirrored_addr1 = mapped_addr & 0x2FFF; // mirror $3XXX addresses to $2XXX addresses
                let mirrored_addr2 = match self.mapper.get_nt_mirror_type() {
                    NametableMirror::Horizontal => { mirrored_addr1 & 0x07FF },
                    NametableMirror::Vertical => { (mirrored_addr1 & 0x03FF) | (if mirrored_addr1 > 0x2800 { 0x400 } else { 0 }) }
                };
                self.vram.write(mirrored_addr2, data);
            },
            0x3F00..=0x3FFF => {
                let mirrored_addr = mapped_addr & 0x1F;

                // println!("Writing data 0x{data:02X} to pal mem w/ addr 0x{mirrored_addr:02X}");

                // Dad's idea: Since we are writing much less than we are reading
                // this data, we can actually mirror it to 2 different addresses
                // and avoid doing any extra bitwise stuff in the read function,
                // we simply read the actual address we are given. (We still have
                // to do bitwise stuff in read, so the performance boost is 
                // almost nothing I think, but there's a lesson to be learned
                // from this idea anyways. I wouldn't have thought of this myself)
                match mirrored_addr {
                    0x00 => { self.palette_mem.write(0x10, data); },
                    0x04 => { self.palette_mem.write(0x14, data); },
                    0x08 => { self.palette_mem.write(0x18, data); },
                    0x0C => { self.palette_mem.write(0x1C, data); },

                    0x10 => { self.palette_mem.write(0x00, data); },
                    0x14 => { self.palette_mem.write(0x04, data); },
                    0x18 => { self.palette_mem.write(0x08, data); },
                    0x1C => { self.palette_mem.write(0x0C, data); },

                    _ => {},
                }

                self.palette_mem.write(mirrored_addr, data);

            },
            _ => {unreachable!("I never thought I'd live to see a Resonance Cascade, let alone create one...");}
        }
    }

    // READ / WRITE FUNCTIONS FOR CPU USE

    /// Takes an address in CPU address space and reads the value of a PPU 
    /// register as a u8. Some registers cannot be read.
    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address & 0x0007 {
            // PPUCTRL
            0 => 0xEE, // Can't read PPUCTRL

            // PPUMASK
            1 => 0xEE, // Can't read PPUMASK

            // PPUSTATUS
            2 => {
                let data = self.status_val();
                // Reads from $2002 reset write latch and vblank flag (after the read occurs)
                self.write_latch = 0;
                self.status.set_in_vblank(0);

                data
            },

            // OAMADDR
            3 => 0xEE, // Can't read OAMADDR

            // OAMDATA
            4 => self.oam_data,

            // PPUSCROLL
            5 => 0xEE, // Can't read PPUSCROLL

            // PPUADDR
            6 => 0xEE, // Can't read PPUADDR

            // PPUDATA
            7 => {
                // Reads are too slow for the PPU to respond immediatly, so they
                // go to a read buffer. With the exception (because of course
                // there's an exception) of palette memory, which responds
                // immediatly and still updates the read buffer, discarding the
                // old read buffer data.
                let mut data = self.read_buffer;
                self.read_buffer = self.ppu_read(self.v_val());

                if self.v_val() >= 0x3F00 {
                    data = self.read_buffer;
                }

                // if self.status().in_vblank() == 1 {
                //     self.inc_coarse_x();
                //     self.inc_coarse_y();
                // } else {
                self.set_v_reg(self.v_val() + if self.ctrl.vram_addr_inc() == 0 { 1 } else { 32 });
                // }

                data
            },
            _ => {
                unreachable!("If the laws of physics no longer apply in the future, God help you.")
            }
        }
    }

    // READS/WRITES FROM CPU:

    /// Write a single byte to the PPU Registers. Internal Registers cannot be
    /// written to, and some registers depend on the internal write latch to
    /// determine which byte is being written.
    pub fn cpu_write(&mut self, address: u16, data: u8) {
        // println!("Writing 0x{data:02X} to PPU Register at addr {address}");

        match address & 0x0007 {
            // PPUCTRL
            0 => {
                // t: ...GH.. ........ <- d: ......GH
                //    <used elsewhere> <- d: ABCDEF..
                self.set_ctrl(data);
                self.t_reg.set_nt_select((data & 3) as usize);
            },

            // PPUMASK
            1 => self.set_mask(data),

            // PPUSTATUS
            2 => {}, // Cannot write PPUSTATUS

            // OAMADDR
            3 => { self.oam_address = data },

            // OAMDATA
            4 => { self.oam_data = data },

            // PPUSCROLL
            5 => {
                if self.write_latch == 0 {
                    // 1st Write => write to low byte
                    // Update internal regs
                    // t: ....... ...ABCDE <- d: ABCDE...
                    // x:              FGH <- d: .....FGH
                    // w:                  <- 1

                    // NOTE: There is no dedicated PPUSCROLL register separate
                    //       from the v/t registers. The scroll information is
                    //       contained entirely within the v/t registers and the
                    //       fine_x register.
                    self.t_reg.set_coarse_x((data >> 3) as usize);
                    self.fine_x = data & 7;

                    self.write_latch = 1;
                } else {
                    // 2nd Write => Write to high byte
                    // Update internal regs
                    // t: FGH..AB CDE..... <- d: ABCDEFGH
                    // w:                  <- 0
                    self.t_reg.set_coarse_y((data >> 3) as usize);
                    self.t_reg.set_fine_y((data & 7) as usize);

                    self.write_latch = 0;
                }
            },

            // PPUADDR
            6 => {
                if self.write_latch == 0 {
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
                    self.write_latch = 1;
                } else {
                    // 2nd Write => Write to high byte
                    // Update internal regs
                    // t: ....... ABCDEFGH <- d: ABCDEFGH
                    // v: <...all bits...> <- t: <...all bits...>
                    // w:                  <- 0
                    self.set_t_reg((self.t_val() & 0xFF00) | (data as u16));
                    self.set_v_reg(self.t_val());
                    self.write_latch = 0;
                }
            },

            // PPUDATA
            7 => {
                // println!("Writing 0x{data:02} to PPUDATA");

                self.ppu_write(self.v_val(), data);

                // if self.status().in_vblank() == 1 {
                //     self.inc_coarse_x();
                //     self.inc_coarse_y();
                // } else {
                self.set_v_reg(self.v_val() + if self.ctrl.vram_addr_inc() == 0 { 1 } else { 32 });
                // }
            },
            _ => unreachable!("Well done. Here are the test results: \"You are a horrible person.\" I'm serious, that's what it says: \"A horrible person.\" We weren't even testing for that."),
        };
    }

    /// Increment the coarse x value in the v register. Also handles wrap around
    /// cases when the value of coarse x overflows. For more details, visit
    /// https://www.nesdev.org/wiki/PPU_scrolling#Wrapping_around
    fn inc_coarse_x(&mut self) {
        if self.rendering_enabled() {
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
    }

    /// Increment the coarse y value in the v register. Also handles wrap around
    /// cases when the value of coarse y overflows. For more details, visit
    /// https://www.nesdev.org/wiki/PPU_scrolling#Wrapping_around
    fn inc_coarse_y(&mut self) {
        if self.rendering_enabled() {
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
    }

    /// Transfer horizontal data (coarse x and nametable x) from the t register
    /// to the v register in preperation for the rendering phase. 
    fn transfer_x_data(&mut self) {
        if self.rendering_enabled() {
            self.v_reg.set_coarse_x(self.t_reg.coarse_x());
            self.v_reg.set_nt_x(self.t_reg.nt_x());
        }
    }

    /// Transfer vertical data (coarse y, nametable y, and fine y) from the t
    /// register to the v register in preperation for the rendering phase.
    fn transfer_y_data(&mut self) {
        if self.rendering_enabled() {
            self.v_reg.set_coarse_y(self.t_reg.coarse_y());
            self.v_reg.set_nt_y(self.t_reg.nt_y());
            self.v_reg.set_fine_y(self.t_reg.fine_y());
        }
    }

    /// Fetches the next byte of background tile nametable ids and stores it in
    /// an internal buffer.
    fn load_nt_buffer(&mut self) {
        self.bg_next_tile_nt_addr = self.ppu_read(0x2000 | (self.v_val() & 0x0FFF));
    }
    /// Fetches the next byte of background tile attributes and stores it in
    /// an internal buffer.
    fn load_attrib_buffer(&mut self) {
        // Read byte containing 4 tile attribute values from mem
        self.bg_next_tile_attrib = self.ppu_read(0x23C0 
                                            | ((self.v_reg.nt_select() as u16) << 10)  // fixed: was nt_y << 11 (ignored nt_x)
                                            | (((self.v_reg.coarse_y() as u16) >> 2) << 3) 
                                            | ((self.v_reg.coarse_x() as u16) >> 2));

        // Shift to obtain the correct tile attribute value
        if self.v_reg.coarse_y() & 0x02 == 1 {
            self.bg_next_tile_attrib >>= 4;
        }

        if self.v_reg.coarse_x() & 0x02 == 1 {
            self.bg_next_tile_attrib >>= 2;
        }

        self.bg_next_tile_attrib &= 3;
    }
    /// Fetches the next low byte of background tile pixel data and stores it
    /// in an internal buffer.
    fn load_bg_lsb_buffer(&mut self) {
        let bg_ptrn = self.ctrl.bg_pattern_tbl();
        let fine_y = self.v_reg.fine_y();

        self.bg_next_tile_lsb = self.ppu_read(
            ((bg_ptrn as u16) << 12) // 0x0000 or 0x1000
            | ((self.bg_next_tile_nt_addr as u16) << 4)
            | (fine_y as u16 + 0));
    }
    /// Fetches the next high byte of background tile pixel data and stores it
    /// in an internal buffer.
    fn load_bg_msb_buffer(&mut self) {
        let bg_ptrn = self.ctrl.bg_pattern_tbl();
        let fine_y = self.v_reg.fine_y();

        self.bg_next_tile_msb = self.ppu_read(
            ((bg_ptrn as u16) << 12) // 0x0000 or 0x1000
            | ((self.bg_next_tile_nt_addr as u16) << 4)
            | (fine_y as u16 + 8)); // fixed: was setting tile_lsb instead of tile_msb
    }

    /// Trigger a NMI within the CPU
    fn trigger_nmi(&mut self) {
        self.cpu_nmi_flag = true;
    }

    /// Draw to the given frame buffer at the current scanline and dot. This function
    /// does not internally check to ensure the scanline and dot are within the bounds
    /// of the screen.
    fn draw_dot(&mut self) {
        let mut bg_pix = 0;
        let mut bg_pal = 0;

        if self.mask.draw_bg() == 1 {
            let bg_pix_hi = (self.bg_tile_nt_hi >> (15 - self.fine_x)) & 1;
            let bg_pix_lo = (self.bg_tile_nt_lo >> (15 - self.fine_x)) & 1;

            let bg_pal_hi = (self.bg_tile_attrib_hi >> (15 - self.fine_x)) & 1;
            let bg_pal_lo = (self.bg_tile_attrib_lo >> (15 - self.fine_x)) & 1;

            bg_pix = (bg_pix_hi << 1) | bg_pix_lo;
            bg_pal = (bg_pal_hi << 1) | bg_pal_lo;
        }

        let col = self.color_from_tile_data(bg_pal, bg_pix);
        let pix_idx = (self.scanline * 256 + self.dot-1)*4;

        let frame = self.screen_buf.as_mut_slice();

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
        self.mask.draw_bg() == 1 || self.mask.draw_sprites() == 1
    }
    /// Get the frame buffer as a slice
    pub fn frame_buf_slice(&self) -> &[u8] {
        self.screen_buf.as_slice()
    }
    pub fn cpu_nmi_flag(&self) -> bool {
        self.cpu_nmi_flag
    }
    
    /// Set the value of the frame finished flag
    pub fn set_frame_finished(&mut self, val: bool) {
        self.frame_finished = val;
    }
    pub fn set_cpu_nmi_flag(&mut self, val: bool) {
        self.cpu_nmi_flag = val;
    }
}

// Getters & Setters (more helper functionality)
impl Ppu2C02 {
    // GETTER / SETTER FUNCTIONS FOR PPU USE

    /// Get a copy of the value of PPUCTRL as a u8
    pub fn ctrl_val(&self) -> u8 {
        self.ctrl.clone().into_bits()
    }
    /// Get a copy of the value of PPUSTATUS as a u8
    pub fn status_val(&self) -> u8 {
        self.status.clone().into_bits()
    }
    /// Get a copy of the value of the v register as a u16
    pub fn v_val(&self) -> u16 {
        self.v_reg.clone().into_bits()
    }
    /// Get a copy of the value of the t register as a u16
    pub fn t_val(&self) -> u16 {
        self.t_reg.clone().into_bits()
    }

    /// Set the value of PPUCTRL
    pub fn set_ctrl(&mut self, val: u8) {
        self.ctrl = val.into();
    }
    /// Set the value of the PPUMASK register
    pub fn set_mask(&mut self, val: u8) {
        self.mask = val.into();
    }
    /// Set the value of the PPUSTATUS register
    pub fn set_status(&mut self, val: u8) {
        self.status = val.into();
    }
    /// Set the value of the v register
    pub fn set_v_reg(&mut self, val: u16) {
        self.v_reg = val.into();
    }
    /// Set the value of the t register
    pub fn set_t_reg(&mut self, val: u16) {
        self.t_reg = val.into();
    }
}