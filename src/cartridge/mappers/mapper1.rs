use crate::cartridge::{mapper::NametableMirror, Cartridge, Mapper};

const PRG_RAM_SIZE: usize = 0x2000;

#[derive(Default)]
pub struct Mapper1 {
    nt_mirror_type: NametableMirror,
    num_prg_banks: usize,
    num_chr_banks: usize,
    control: u8,

    write_count: usize,
    shift_reg: u8,

    chr_bank_select_lo: usize,
    chr_bank_select_hi: usize,
    chr_bank_select_full: usize,

    prg_bank_select_lo: usize,
    prg_bank_select_hi: usize,
    prg_bank_select_full: usize,

    prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    chr_mem: Vec<u8>,
}

impl Mapper for Mapper1 {
    fn init(&mut self, cart: Cartridge) {
        self.num_prg_banks = cart.prg_rom_banks();
        self.num_chr_banks = cart.chr_rom_banks();

        self.prg_rom = cart.get_prg_rom();
        self.chr_mem = cart.get_chr_rom();

        self.prg_ram = vec![0; PRG_RAM_SIZE];

        self.reset();
    }

    fn cpu_cart_read(&mut self, addr: u16) -> Option<u8> {
        match addr {
            // Internal PRG RAM
            0x6000..=0x7FFF => {
                let mapped_addr = (addr & 0x1FFF) as usize;

                Some( self.prg_ram[mapped_addr] )
            }

            // PRG ROM Low
            0x8000..=0xBFFF => {
                let mapped_addr = if self.split_prg_bank_mode() {
                    self.prg_bank_select_lo * 0x4000 + (addr & 0x3FFF) as usize
                } else {
                    self.prg_bank_select_full * 0x8000 + (addr & 0x7FFF) as usize
                };

                Some( self.prg_rom[mapped_addr] )
            }

            // PRG ROM High
            0xC000..=0xFFFF => {
                let mapped_addr = if self.split_prg_bank_mode() {
                    self.prg_bank_select_hi * 0x4000 + (addr & 0x3FFF) as usize
                } else {
                    self.prg_bank_select_full * 0x8000 + (addr & 0x7FFF) as usize
                };

                Some( self.prg_rom[mapped_addr] )
            }

            _ => None,
        }
    }

    fn ppu_cart_read(&mut self, addr: u16) -> Option<u8> {
        if addr <= 0x1FFF {
            // Only one CHR bank to access
            if self.num_chr_banks == 0 {
                return Some( self.chr_mem[addr as usize] )
            }

            let mapped_addr = if self.split_chr_bank_mode() {
                // CHR Bank Low
                if addr <= 0x0FFF {
                    self.chr_bank_select_lo * 0x1000 + addr as usize
                } 
                // CHR Bank High
                else {
                    self.chr_bank_select_hi * 0x1000 + (addr & 0x0FFF) as usize
                }
            } else {
                // CHR Bank Full
                self.chr_bank_select_full * 0x2000 + addr as usize
            };

            return Some( self.chr_mem[mapped_addr] );
        }

        None
    }

    fn cpu_cart_write(&mut self, addr: u16, data: u8) -> bool {
        match addr {
            // Internal PRG RAM
            0x6000..=0x7FFF => {
                let mapped_addr = (addr & 0x1FFF) as usize;
                
                self.prg_ram[mapped_addr] = data;

                true
            }

            // PRG ROM (Mapper Registers)
            0x8000..=0xFFFF => {
                if data & 0x80 == 0 {
                    self.cpu_write_regs(addr, data);
                } else {
                    self.shift_reg = 0;
                    self.write_count = 0;
                    self.control |= 0x0C;
                }

                // Even though the mapped updated the write, the roms aren't updated
                false
            }

            _ => false,
        }
    }

    fn ppu_cart_write(&mut self, addr: u16, data: u8) -> bool {
        if addr <= 0x1FFF {
            // If # CHR banks == 0, treat CHR ROM as CHR RAM
            if self.num_chr_banks == 0 {
                self.chr_mem[addr as usize] = data;
                return true;
            }
        }

        false
    }

    fn get_nt_mirror_type(&self) -> NametableMirror {
        self.nt_mirror_type
    }
}


impl Mapper1 {
    fn reset(&mut self) {
        self.control = 0x1C;
        
        self.write_count = 0;
        self.shift_reg = 0;

        self.chr_bank_select_lo = 0;
        self.chr_bank_select_hi = 0;
        self.chr_bank_select_full = 0;

        self.prg_bank_select_lo = 0;
        self.prg_bank_select_hi = self.num_prg_banks - 1;
        self.prg_bank_select_full = 0;
    }

    fn cpu_write_regs(&mut self, address: u16, data: u8) {
        self.shift_reg >>= 1;
        self.shift_reg |= (data & 1) << 4; // writes low bit of value first, then higher bit, etc.
        self.write_count += 1;

        if self.write_count == 5 {
            let shift_val = (self.shift_reg & 0x1F) as usize;

            // The register being accessed depends on bits 13 & 14 of the address.
            // addr: ----------------
            //       15 ... bit ... 0
            match (address >> 13) & 0x03 {
                // Control Register
                0 => {
                    self.control = shift_val as u8;

                    self.nt_mirror_type = match self.control & 0x03 {
                        0 => NametableMirror::SingleScreenLower,
                        1 => NametableMirror::SingleScreenUpper,
                        2 => NametableMirror::Vertical,
                        3 => NametableMirror::Horizontal,
                        _ => {unreachable!("Things are wrong :O")},
                    };
                }

                // CHR ROM Select Low OR Full
                1 => {
                    if self.split_chr_bank_mode() {
                        self.chr_bank_select_lo = shift_val;
                    } else {
                        // might need to shift here instead of just masking bit, keep an eye on it
                        self.chr_bank_select_full = shift_val & 0x1E; // Full mode ignores lowest bit
                    }
                }

                // CHR ROM Select High
                2 => {
                    if self.split_chr_bank_mode() {
                        self.chr_bank_select_hi = shift_val;
                    }
                }

                // PRG ROM Select Low/High/Full depending on control register
                3 => {
                    match (self.control >> 2) & 0x03 {
                        // Switch Full
                        0 | 1 => {
                            self.prg_bank_select_full = shift_val >> 1;
                        }

                        // Fix Low At Bank 0 & Switch High
                        2 => {
                            self.prg_bank_select_lo = 0;
                            self.prg_bank_select_hi = shift_val & 0x0F;
                        }

                        // Switch Low & Fix High At Last Bank
                        3 => {
                            self.prg_bank_select_lo = shift_val & 0x0F;
                            self.prg_bank_select_hi = self.num_prg_banks - 1;
                        }

                        _ => {}
                    }
                }

                _ => {unreachable!("Whatchu doin here?")}
            }
        
            self.shift_reg = 0;
            self.write_count = 0;
        }
    }

    pub fn split_prg_bank_mode(&self) -> bool {
        self.control & 0x08 != 0
    }

    pub fn split_chr_bank_mode(&self) -> bool {
        self.control & 0x10 != 0
    }
}