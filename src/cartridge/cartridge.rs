use crate::cartridge::mapper;
use crate::system::mem::Memory;

use super::mapper::Mapper;

// Identifier for NES 2.0 and INES formats
pub const NES_2V0_IDENT: [u8; 4] = [b'N', b'E', b'S', 0x1A];

#[derive(PartialEq, Debug)]
pub enum CartFormat {
    Unknown,
    INES,
    V2NES,
}

/// Struct containing information given by the 16 byte header of the cartridge
#[derive(Default)]
pub struct Header {
    // Bytes 0-3
    identifier: String,
    // Byte 4
    prg_rom_size: u16,
    // Byte 5
    chr_rom_size: u16,
    // Byte 6
    mapper_num: u16,
    alt_nametables: bool,
    has_trainer: bool, // shouldn't matter for our purposes
    battery_present: bool,
    hardwired_nametable: bool,
    // Byte 7
    console_type: u8,
    // Byte 8
    submapper_num: u8,
    // Byte 9
    //   more prg/chr rom size
    // Byte 10
    has_prg_ram: bool,
    prg_ram_shift: u8,
    has_prg_nv_ram: bool,
    prg_nv_ram_shift: u8,
    // Byte 11
    has_chr_ram: bool,
    chr_ram_shift: u8,
    has_chr_nv_ram: bool,
    chr_nv_ram_shift: u8,
    // Byte 12
    timing_mode: u8,
    // Byte 13
    vs_hardware_type: u8,
    vs_ppu_type: u8,
    extended_console_type: u8,
    // Byte 14
    misc_roms_count: u8,
    // Byte 15
    default_expansion_device: u8,
}

/// Representation of a standard NES Cartridge.
pub struct Cartridge {
    _format: CartFormat,

    _header: Header,

    // trainer_area: Option<[u8; 512]>,
    prg_rom: Memory,
    chr_rom: Memory,
    _misc_rom: Memory,
}

impl Cartridge {
    pub const HEADER_LEN: usize = 16;
    pub const TRAINER_LEN: usize = 512;

    /// Attempts to parse the header section of the provided data. If the slice
    /// of bytes isn't in the NES 2.0 or iNES format, an error is returned. Else
    /// it reads the header and constructs and returns the cartridge. The layout
    /// of the header is described in detail here:
    /// https://www.nesdev.org/wiki/NES_2.0#Header
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < Self::HEADER_LEN {
            return Err(String::from("Cartridge file not NES 2.0 (or INES) format"));
        }

        // Identifier should be 'NES<EOF>'
        let mut format: CartFormat;
        if data[..4] == NES_2V0_IDENT {
            format = CartFormat::INES;
            // Identifier flags are NES 2.0
            if (data[7] & 0x0C) == 0x08 {
                format = CartFormat::V2NES;
            }
        } else {
            format = CartFormat::Unknown;
        }

        // If file is of unknown format, the identifier still could be looked at to determine file
        // type. Mostly a debugging tool.
        let ident = match std::str::from_utf8(&data[..4]) {
            Ok(v) => v,
            Err(..) => return Err(String::from("Could not read cart file identifier")),
        };

        let mut header = Header::default();

        header.identifier = ident.to_string();

        if format == CartFormat::Unknown {
            return Err(format!("Unknown cartridge format '{0}'", header.identifier));
        }

        header.prg_rom_size |= data[4] as u16;
        header.chr_rom_size |= data[5] as u16;

        header.mapper_num |= (data[6] & 0xF0) as u16 >> 4;
        header.alt_nametables = (data[6] & 0x08) != 0;
        header.has_trainer = (data[6] & 0x04) != 0;
        header.battery_present = (data[6] & 0x02) != 0;
        header.hardwired_nametable = (data[6] & 0x01) != 0;

        header.mapper_num |= (data[7] & 0xF0) as u16;
        header.console_type = data[7] & 0x03;

        header.submapper_num = (data[8] & 0xF0) >> 4;
        header.mapper_num |= ((data[8] & 0x0F) as u16) << 8;

        header.chr_rom_size |= ((data[9] & 0xF0) as u16) << 4;
        header.prg_rom_size |= ((data[9] & 0x0F) as u16) << 8;

        header.has_prg_nv_ram = data[10] & 0xF0 != 0;
        header.prg_nv_ram_shift = (data[10] & 0xF0) >> 4;
        header.has_prg_ram = data[10] & 0x0F != 0;
        header.prg_ram_shift = data[10] & 0x0F;

        header.has_chr_nv_ram = data[11] & 0xF0 != 0;
        header.chr_nv_ram_shift = (data[11] & 0xF0) >> 4;
        header.has_chr_ram = data[11] & 0x0F != 0;
        header.chr_ram_shift = data[11] & 0x0F;

        header.timing_mode = data[12] & 0x03;

        if header.console_type == 0b01 {
            header.vs_hardware_type = (data[13] & 0xF0) >> 4;
            header.vs_ppu_type = data[13] & 0x0F;
        } else if header.console_type == 0b11 {
            header.extended_console_type = data[13] & 0x0F;
        }

        header.misc_roms_count = data[14] & 0x03;

        header.default_expansion_device = data[15] & 0x3F;

        const CHR_ROM_BANK_SIZE: usize = 0x2000; // 8KiB
        const PRG_ROM_BANK_SIZE: usize = 0x4000; // 16KiB

        let prg_rom_start = 0x10 + if header.has_trainer { 0x200 } else { 0 };
        let prg_rom_banks = Cartridge::rom_size(header.prg_rom_size);
        let prg_rom_end = prg_rom_start + prg_rom_banks * PRG_ROM_BANK_SIZE;
        let prg_rom_vec = data[prg_rom_start..prg_rom_end].to_vec();

        let chr_rom_start = prg_rom_end;
        let chr_rom_banks = Cartridge::rom_size(header.chr_rom_size);
        let chr_rom_end = chr_rom_start + chr_rom_banks * CHR_ROM_BANK_SIZE;
        let chr_rom_vec = data[chr_rom_start..chr_rom_end].to_vec();

        let misc_rom_vec = data[chr_rom_end..].to_vec();

        println!("Prg ROM Size: {} (given by {})", prg_rom_vec.len(), header.prg_rom_size);
        println!("Chr ROM Size: {} (given by {})", chr_rom_vec.len(), header.chr_rom_size);
        println!("Misc ROM Size: {}", misc_rom_vec.len());

        Ok(Cartridge {
            _format: format,
            _header: header,
            // trainer_area: None,
            prg_rom: Memory::from_vec(prg_rom_vec),
            chr_rom: Memory::from_vec(chr_rom_vec),
            _misc_rom: Memory::from_vec(misc_rom_vec),
        })
    }

    /// Translates from the prg/chr ROM size specified by the header to the
    /// actual number of bytes to read in from the cart file.
    fn rom_size(rom_size_bytes: u16) -> usize {
        if (rom_size_bytes & 0xF00) == 0xF00 {
            let mm = (rom_size_bytes & 0x3) as usize;
            let exp = (rom_size_bytes & 0xFC) >> 2;

            return (1 << exp) * (2 * mm + 1);
        }

        rom_size_bytes as usize
    }

    pub fn get_prg_rom(&self) -> Memory {
        self.prg_rom.clone()
    }

    pub fn get_chr_rom(&self) -> Memory {
        self.chr_rom.clone()
    }

    pub fn get_mapper(&self) -> impl Mapper {
        mapper::get_mapper(self._header.mapper_num)
    }
}
