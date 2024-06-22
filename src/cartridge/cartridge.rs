


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
    /// Cartridge Identifier String - For iNES and NES 2.0 file formats, this
    /// should be "NES\x1A"
    pub identifier: String,
    // Byte 4
    /// Number of 16KiB chunks of the Program ROM the cartridge contains 
    pub prg_rom_size: u16,
    // Byte 5
    /// Number of 8KiB chunks of the Program ROM the cartridge contains
    pub chr_rom_size: u16,
    // Byte 6
    /// Mapper identifier number. See https://www.nesdev.org/wiki/Mapper for an
    /// extensive list.
    pub mapper_num: u16,
    /// Determines whether horizontal/vertical mirroring is present for the ppu
    /// nametables.
    ///  * `false` - horizontal/vertical mirroring present
    ///  * `true` - alternative mirroring present (horizontal/vertical not present)
    pub alt_nametables: bool,
    /// Whether the cartridge includes trainer data. Unused in this emulator
    pub has_trainer: bool, // shouldn't matter for our purposes
    /// Whether there is a battery present. Unused for this emulator
    pub battery_present: bool,
    /// Determines the kind of mirroring the cartridge has for nametables. See
    /// https://www.nesdev.org/wiki/PPU_nametables for more information.
    /// 
    ///  * `false` - Veritical mirroring
    ///  * `true` - Horizontal mirroring
    pub hardwired_nametable: bool,
    // Byte 7
    /// Type of console cartridge is meant for. Unused in this emulator
    pub console_type: u8,
    // Byte 8
    pub submapper_num: u8,
    // Byte 9
    //   more prg/chr rom size
    // Byte 10
    pub has_prg_ram: bool,
    pub prg_ram_shift: u8,
    pub has_prg_nv_ram: bool,
    pub prg_nv_ram_shift: u8,
    // Byte 11
    pub has_chr_ram: bool,
    pub chr_ram_shift: u8,
    pub has_chr_nv_ram: bool,
    pub chr_nv_ram_shift: u8,
    // Byte 12
    pub timing_mode: u8,
    // Byte 13
    pub vs_hardware_type: u8,
    pub vs_ppu_type: u8,
    pub extended_console_type: u8,
    // Byte 14
    pub misc_roms_count: u8,
    // Byte 15
    pub default_expansion_device: u8,
}

/// Representation of a standard NES Cartridge.
pub struct Cartridge {
    pub header: Header,
    
    format: CartFormat,

    // trainer_area: Option<[u8; 512]>,
    prg_rom_banks: usize,
    prg_rom: Vec<u8>,

    chr_rom_banks: usize,
    chr_rom: Vec<u8>,

    misc_rom: Vec<u8>,
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
        
        let prg_rom = if header.prg_rom_size > 0 {
            data[prg_rom_start..prg_rom_end].to_vec()
        } else {
            vec![0; PRG_ROM_BANK_SIZE] // If no prg rom allocated by program, give cpu 1 bank so it doesn't freak out
        };

        let chr_rom_start = prg_rom_end;
        let chr_rom_banks = Cartridge::rom_size(header.chr_rom_size);
        let chr_rom_end = chr_rom_start + chr_rom_banks * CHR_ROM_BANK_SIZE;

        let chr_rom = if header.chr_rom_size > 0 {
            data[chr_rom_start..chr_rom_end].to_vec()
        } else {
            vec![0; CHR_ROM_BANK_SIZE] // If no chr rom allocated by program, give ppu 1 bank so it doesn't freak out
        };

        let misc_rom = data[chr_rom_end..].to_vec();

        println!("Prg ROM Size: {} (given by {})", prg_rom.len(), header.prg_rom_size);
        println!("Chr ROM Size: {} (given by {})", chr_rom.len(), header.chr_rom_size);
        println!("Misc ROM Size: {}", misc_rom.len());

        Ok(Cartridge {
            format,
            header,
            // trainer_area: None,
            prg_rom_banks,
            prg_rom,

            chr_rom_banks,
            chr_rom,

            misc_rom,
        })
    }

    /// Translates from the prg/chr ROM size specified by the header to the
    /// actual number of bytes to read in from the cart file.
    pub fn rom_size(rom_size_bytes: u16) -> usize {
        if (rom_size_bytes & 0xF00) == 0xF00 {
            let mm = (rom_size_bytes & 0x3) as usize;
            let exp = (rom_size_bytes & 0xFC) >> 2;

            return (1 << exp) * (2 * mm + 1);
        }

        rom_size_bytes as usize
    }

    pub fn get_prg_rom(&self) -> Vec<u8> {
        self.prg_rom.clone()
    }

    pub fn prg_rom_banks(&self) -> usize {
        self.prg_rom_banks
    }

    pub fn get_chr_rom(&self) -> Vec<u8> {
        self.chr_rom.clone()
    }

    pub fn chr_rom_banks(&self) -> usize {
        self.chr_rom_banks
    }
}
