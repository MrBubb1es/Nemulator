use crate::cartridge::{mapper::NametableMirror, Cartridge, Mapper};

const PRG_RAM_SIZE: usize = 0x2000;
const PRG_BANK_SIZE: usize = 0x2000;
const CHR_BANK_SIZE: usize = 0x400;

// Mapper 4 (AKA MMC3)
// Just about the most complicated mapper of them all. The PRG and CHR ROMs are
// split into 4 and 8 banks, respectively. Where each bank is mapped to is controlled
// by a total of 8 index registers, plus a control register. The details of this
// mapper are pretty complex, but its really a simple idea at a high level. I
// suggest the video by javidx9 on NES mappers.
// Games:
// - Super Mario Bros 2
// - Super Mario Bros 3
#[derive(Default)]
pub struct Mapper4 {
    irq_counter: usize,
    irq_latch: usize,
    irq_enabled: bool,
    irq_request_flag: bool,

    bank_select: u8,
    registers: [u8; 8],
    prg_banks: [usize; 4],
    chr_banks: [usize; 8],

    nt_mirror_type: NametableMirror,

    num_prg_banks: usize,

    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,

    prg_ram: Vec<u8>,
}

impl Mapper for Mapper4 {
    fn init(&mut self, cart: Cartridge) {
        if cart.header.alt_nametables {
            self.nt_mirror_type = NametableMirror::FourScreen
        }

        self.num_prg_banks = cart.prg_rom_banks();
        self.prg_rom = cart.get_prg_rom();
        self.chr_rom = cart.get_chr_rom();

        self.prg_ram = vec![0; PRG_RAM_SIZE];

        self.reset();
    }

    fn cpu_cart_read(&mut self, addr: u16) -> Option<u8> {
        match addr {
            0x6000..=0x7FFF => {
                let mapped_addr = (addr & 0x1FFF) as usize;

                Some( self.prg_ram[mapped_addr] )
            }

            0x8000..=0x9FFF => {
                let mapped_addr = self.prg_banks[0] + (addr & 0x1FFF) as usize;
                
                Some( self.prg_rom[mapped_addr] )
            }

            0xA000..=0xBFFF => {
                let mapped_addr = self.prg_banks[1] + (addr & 0x1FFF) as usize;

                Some( self.prg_rom[mapped_addr] )
            }

            0xC000..=0xDFFF => {
                let mapped_addr = self.prg_banks[2] + (addr & 0x1FFF) as usize;
                                
                Some( self.prg_rom[mapped_addr] )
            }

            0xE000..=0xFFFF => {
                let mapped_addr = self.prg_banks[3] + (addr & 0x1FFF) as usize;
                
                Some( self.prg_rom[mapped_addr] )
            }
        
            _ => None,
        }
    }

    fn ppu_cart_read(&mut self, addr: u16) -> Option<u8> {
        match addr {
            0x0000..=0x03FF => {
                let mapped_addr = self.chr_banks[0] + (addr & 0x3FF) as usize;
                
                Some( self.chr_rom[mapped_addr] )
            }

            0x0400..=0x07FF => {
                let mapped_addr = self.chr_banks[1] + (addr & 0x3FF) as usize;
                
                Some( self.chr_rom[mapped_addr] )
            }

            0x0800..=0x0BFF => {
                let mapped_addr = self.chr_banks[2] + (addr & 0x3FF) as usize;
                
                Some( self.chr_rom[mapped_addr] )
            }

            0x0C00..=0x0FFF => {
                let mapped_addr = self.chr_banks[3] + (addr & 0x3FF) as usize;
                
                Some( self.chr_rom[mapped_addr] )
            }

            0x1000..=0x13FF => {
                let mapped_addr = self.chr_banks[4] + (addr & 0x3FF) as usize;
                
                Some( self.chr_rom[mapped_addr] )
            }

            0x1400..=0x17FF => {
                let mapped_addr = self.chr_banks[5] + (addr & 0x3FF) as usize;
                
                Some( self.chr_rom[mapped_addr] )
            }

            0x1800..=0x1BFF => {
                let mapped_addr = self.chr_banks[6] + (addr & 0x3FF) as usize;
                
                Some( self.chr_rom[mapped_addr] )
            }

            0x1C00..=0x1FFF => {
                let mapped_addr = self.chr_banks[7] + (addr & 0x3FF) as usize;
                
                Some( self.chr_rom[mapped_addr] )
            }

            _ => None,
        }
    }

    fn cpu_cart_write(&mut self, addr: u16, data: u8) -> bool {
        match addr {
            // PRG RAM
            0x6000..=0x7FFF => {
                let mapped_addr = (addr & 0x1FFF) as usize;

                self.prg_ram[mapped_addr] = data;

                true
            }

            // Bank Select (Even), Bank Data (Odd)
            0x8000..=0x9FFF => {
                // Bank Select
                if addr & 0x01 == 0 {
                    self.bank_select = data;
                }
                // Bank Data
                else {
                    let reg_idx = (self.bank_select & 0x07) as usize;

                    if reg_idx == 0 || reg_idx == 1 {
                        self.registers[reg_idx] = data & 0xFE;
                    } else {
                        self.registers[reg_idx] = data;
                    }

                    self.update_banks();
                }

                false
            },

            // Mirror (Even), PRG ROM Protect (Odd - unimplemented)
            0xA000..=0xBFFF => {
                // New Mirror
                if addr & 0x01 == 0 {
                    if self.nt_mirror_type != NametableMirror::FourScreen {
                        self.nt_mirror_type = if data & 1 == 0 {
                            NametableMirror::Vertical
                        } else {
                            NametableMirror::Horizontal
                        };
                    }
                }
                // Prg Rom Protect
                else {
                    // unimplemented
                }

                false
            },

            // IRQ Latch (Even), IRQ Clear (Odd)
            0xC000..=0xDFFF => {
                // IRQ Latch
                if addr & 1 == 0 {
                    self.irq_latch = data as usize;
                }
                // IRQ Reload
                else {
                    self.irq_counter = 0;
                }

                false
            }

            // IRQ Disable (Even), IRQ Enable (Odd)
            0xE000..=0xFFFF => {
                // IRQ Disable
                if addr & 1 == 0 {
                    self.irq_enabled = false;
                    self.irq_request_flag = false;
                }
                // IRQ Enable
                else {
                    self.irq_enabled = true;
                }

                false
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

    fn reset(&mut self) {
        self.bank_select = 0;

        if self.nt_mirror_type != NametableMirror::FourScreen {
            self.nt_mirror_type = NametableMirror::Horizontal;
        }

        self.irq_request_flag = false;
        self.irq_enabled = false;
        self.irq_counter = 0;
        self.irq_latch = 0;

        self.registers.fill(0);
        self.chr_banks.fill(0);

        self.prg_banks[0] = 0 * PRG_BANK_SIZE;
        self.prg_banks[1] = 1 * PRG_BANK_SIZE;
        self.prg_banks[2] = (self.num_prg_banks * 2 - 2) * PRG_BANK_SIZE;
        self.prg_banks[3] = (self.num_prg_banks * 2 - 1) * PRG_BANK_SIZE;
    }

    fn scanline_finished(&mut self) {
        if self.irq_counter == 0 {
            self.irq_counter = self.irq_latch;
        } else {
            self.irq_counter -= 1;
        }

        if self.irq_counter == 0 && self.irq_enabled {
            self.irq_request_flag = true;
        }
    }

    fn irq_requested(&self) -> bool {
        self.irq_request_flag
    }

    fn irq_handled(&mut self) {
        self.irq_request_flag = false;
    }
}


impl Mapper4 {
    fn prg_banks_swapped(&self) -> bool {
        self.bank_select & 0x40 != 0
    }

    fn chr_banks_swapped(&self) -> bool {
        self.bank_select & 0x80 != 0
    }

    fn update_banks(&mut self) {
        if self.prg_banks_swapped() {
            self.prg_banks[0] = (self.num_prg_banks * 2 - 2) * PRG_BANK_SIZE;
            self.prg_banks[1] = (self.registers[7] & 0x3F) as usize * PRG_BANK_SIZE;
            self.prg_banks[2] = (self.registers[6] & 0x3F) as usize * PRG_BANK_SIZE;
            self.prg_banks[3] = (self.num_prg_banks * 2 - 1) * PRG_BANK_SIZE;
        } else {
            self.prg_banks[0] = (self.registers[6] & 0x3F) as usize * PRG_BANK_SIZE;
            self.prg_banks[1] = (self.registers[7] & 0x3F) as usize * PRG_BANK_SIZE;
            self.prg_banks[2] = (self.num_prg_banks * 2 - 2) * PRG_BANK_SIZE;
            self.prg_banks[3] = (self.num_prg_banks * 2 - 1) * PRG_BANK_SIZE;
        }

        if self.chr_banks_swapped() {
            self.chr_banks[0] = self.registers[2] as usize * CHR_BANK_SIZE;
            self.chr_banks[1] = self.registers[3] as usize * CHR_BANK_SIZE;
            self.chr_banks[2] = self.registers[4] as usize * CHR_BANK_SIZE;
            self.chr_banks[3] = self.registers[5] as usize * CHR_BANK_SIZE;
            self.chr_banks[4] = (self.registers[0] as usize + 0) * CHR_BANK_SIZE;
            self.chr_banks[5] = (self.registers[0] as usize + 1) * CHR_BANK_SIZE;
            self.chr_banks[6] = (self.registers[1] as usize + 0) * CHR_BANK_SIZE;
            self.chr_banks[7] = (self.registers[1] as usize + 1) * CHR_BANK_SIZE;
        } else {
            self.chr_banks[0] = (self.registers[0] as usize + 0) * CHR_BANK_SIZE;
            self.chr_banks[1] = (self.registers[0] as usize + 1) * CHR_BANK_SIZE;
            self.chr_banks[2] = (self.registers[1] as usize + 0) * CHR_BANK_SIZE;
            self.chr_banks[3] = (self.registers[1] as usize + 1) * CHR_BANK_SIZE;
            self.chr_banks[4] = self.registers[2] as usize * CHR_BANK_SIZE;
            self.chr_banks[5] = self.registers[3] as usize * CHR_BANK_SIZE;
            self.chr_banks[6] = self.registers[4] as usize * CHR_BANK_SIZE;
            self.chr_banks[7] = self.registers[5] as usize * CHR_BANK_SIZE;
        }
    }
}