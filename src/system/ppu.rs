use std::rc::Rc;

use crate::cartridge::cartridge::Cartridge;

use super::mem::Memory;
use super::nes_graphics;

/// Representation of the NES Picture Processing Unit. Details on how the PPU
/// works can be found here: https://www.nesdev.org/wiki/PPU_registers
pub struct PPU {
    // Registers

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
    ppu_ctrl: u8,
    
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
    ppu_mask: u8,

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
    ppu_status: u8,

    // OAMADDRESS Register (write only)
    oam_address: u8,

    // OAMDATA Register (read and write)
    oam_data: u8,

    // PPUSCROLL Register ()
    ppu_scroll: u8,
    ppu_address: u8,
    ppu_data: u8,

    oam_dma: u8,

    cart: Rc<Cartridge>,
    vram: Memory,

    palette: nes_graphics::NESPalette,
}

impl PPU {
    /// Create a new PPU 
    ///  * `cart` - The cartridge to attatch to the PPU bus
    pub fn new(cart: Rc<Cartridge>) -> Self {
        PPU {
            ppu_ctrl: 0, ppu_mask: 0, ppu_status: 0, oam_address: 0, 
            oam_data: 0, ppu_scroll: 0, ppu_address: 0, ppu_data: 0, 
            
            oam_dma: 0,

            cart: cart,

            vram: Memory::new(0x800), // 2KiB ppu ram
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
        match address {
            0x0000..=0x1FFF => self.cart.ppu_read(address),
            0x2000..=0x3EFF => self.read_vram(address & 0x0FFF),
            0x3F00..=0x3FFF => self.read_palette(address & 0x00FF),
            _ => 0,
        }
    }

    fn read_vram(&self, address: u16) -> u8 {
        0
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

    }

    pub fn render_to_arr(dest: &mut [u8; 256*240*3]) {

    }
}