use crate::cartridge::Cartridge;
use crate::cartridge::mapper::{Mapper, NametableMirror};

const PRG_RAM_SIZE: usize = 0x2000;
const PRG_BANK_SIZE: usize = 0x2000;
const CHR_BANK_SIZE: usize = 0x1000;

// Mapper 9 (AKA MMC2)
// Games:
// - Punch Out
#[derive(Debug, Default)]
pub struct Mapper9 {
    nt_mirror_type: NametableMirror,
    num_prg_banks: usize,
    num_chr_banks: usize,

    prg_bank_select_lo: usize,
    prg_bank_select_hi: usize,
    chr_bank_lo_latch_off: usize, // FD
    chr_bank_lo_latch_on: usize,  // FE
    chr_bank_hi_latch_off: usize,
    chr_bank_hi_latch_on: usize,

    // True == FE, False == FD
    chr_latch_lo: bool,
    chr_latch_hi: bool,

    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,

    prg_ram: Vec<u8>,
}

impl Mapper for Mapper9 {
    fn init(&mut self, cart: Cartridge) {
        self.nt_mirror_type = if cart.header.hardwired_nametable {
            NametableMirror::Vertical
        } else {
            NametableMirror::Horizontal
        };


        self.num_prg_banks = cart.prg_rom_banks();
        self.num_chr_banks = cart.chr_rom_banks();
        self.prg_rom = cart.get_prg_rom();
        self.chr_rom = cart.get_chr_rom();

        self.prg_ram = vec![0; PRG_RAM_SIZE];

        // The PRG ROM is split into 4 8KiB chunks. The highest 3 are fixed to the
        // last 3 banks in the cartridge.
        self.prg_bank_select_hi = self.num_prg_banks * 2 - 3;
    }

    fn cpu_cart_read(&mut self, addr: u16) -> Option<u8> {
        match addr {
            // PRG RAM
            0x6000..=0x7FFF => {
                let mapped_addr = (addr & 0x1FFF) as usize;

                Some( self.prg_ram[mapped_addr] )
            }

            // 1st PRG ROM Bank (Switchable)
            0x8000..=0x9FFF => {
                let mapped_addr = self.prg_bank_select_lo * PRG_BANK_SIZE + (addr & 0x1FFF) as usize;

                Some( self.prg_rom[mapped_addr] )
            }

            // 2nd, 3rd, & 4th PRG ROM Banks (Fixed to last 3 banks)
            0xA000..=0xFFFF => {
                let mapped_addr = self.prg_bank_select_hi * PRG_BANK_SIZE + (addr - 0xA000) as usize;

                Some( self.prg_rom[mapped_addr] )
            }

            _ => None,
        }
    }

    fn ppu_cart_read(&mut self, addr: u16) -> Option<u8> {
        match addr {
            // CHR ROM Bank Low (One of two banks depending on state of the low latch)
            0x0000..=0x0FFF => {
                let chr_bank_select_lo = if self.chr_latch_lo { 
                    self.chr_bank_lo_latch_on 
                } else { 
                    self.chr_bank_lo_latch_off 
                };

                let mapped_addr = chr_bank_select_lo * CHR_BANK_SIZE + addr as usize;

                let data = Some( self.chr_rom[mapped_addr] );

                // PPU reads from these addresses set the low latch
                if addr == 0x0FD8 {
                    self.chr_latch_lo = false;
                } else if addr == 0x0FE8 {
                    self.chr_latch_lo = true;
                }

                data
            }

            // CHR ROM Bank High (One of two banks depending on state of the high latch)
            0x1000..=0x1FFF => {
                let chr_bank_select_hi = if self.chr_latch_hi { 
                    self.chr_bank_hi_latch_on
                } else { 
                    self.chr_bank_hi_latch_off 
                };

                let mapped_addr = chr_bank_select_hi * CHR_BANK_SIZE + (addr - 0x1000) as usize;

                let data = Some( self.chr_rom[mapped_addr] );

                // PPU reads from these addresses set the high latch
                if 0x1FD8 <= addr && addr <= 0x1FDF {
                    self.chr_latch_hi = false;
                } else if 0x1FE8 <= addr && addr <= 0x1FEF {
                    self.chr_latch_hi = true;
                }

                data
            }
            
            _ => None,
        }
    }

    fn cpu_cart_write(&mut self, addr: u16, data: u8) -> bool {
        match addr {
            0x6000..=0x7FFF => {
                let mapped_addr = (addr - 0x6000) as usize;

                self.prg_ram[mapped_addr] = data;

                true
            }
            // PRG ROM Bank Select Low
            0xA000..=0xAFFF => {
                self.prg_bank_select_lo = (data & 0x0F) as usize;

                true
            }

            // CHR ROM Low Bank Select (Latch off)
            0xB000..=0xBFFF => {
                self.chr_bank_lo_latch_off = (data & 0x1F) as usize;
            
                true
            }

            // CHR ROM Low Bank Select (Latch on)
            0xC000..=0xCFFF => {
                self.chr_bank_lo_latch_on = (data & 0x1F) as usize;
            
                true
            }

            // CHR ROM High Bank Select (Latch off)
            0xD000..=0xDFFF => {
                self.chr_bank_hi_latch_off = (data & 0x1F) as usize;
            
                true
            }

            // CHR ROM High Bank Select (Latch on)
            0xE000..=0xEFFF => {
                self.chr_bank_hi_latch_on = (data & 0x1F) as usize;
            
                true
            }

            // Mirror
            0xF000..=0xFFFF => {
                self.nt_mirror_type = if data & 1 == 0 {
                    NametableMirror::Vertical
                } else {
                    NametableMirror::Horizontal
                };

                true
            }
            
            _ => false,
        }
    }

    fn ppu_cart_write(&mut self, _addr: u16, _data: u8) -> bool {
        false
    }

    fn get_nt_mirror_type(&self) -> NametableMirror {
        self.nt_mirror_type
    }
}
