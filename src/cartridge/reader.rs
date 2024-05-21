/*
*  A reader for NES 2.0 file roms
*/

// use std::io::Read;
// use std::fs;
// use log::info;

// use super::cartridge::Cartridge;

// pub struct Reader {}

// impl Reader {
//     pub fn new() -> Self {
//         Reader {}
//     }

//     fn read_cart_to_buf(&self, cart_path: &str) -> Result<Vec<u8>, String> {
//         let mut cart_file = match fs::File::open(cart_path) {
//             Ok(v) => v,
//             Err(..) => return Err(String::from(format!("Could not find file '{}'", cart_path))),
//         };

//         let mut data = Vec::new();

//         match cart_file.read_to_end(&mut data) {
//             Ok(..) => Ok(data),
//             Err(..) => Err(String::from(format!("Failed to read cartridge from '{}' to buffer", cart_path))),
//         }
//     }

//     fn read_trainer_data_from_buf(&self, buf: &[u8], cart_pos: &mut usize) -> Result<[u8; Cartridge::TRAINER_LEN], String> {
//         let mut trainer = [0; Cartridge::TRAINER_LEN];

//         let start = *cart_pos;
//         let end = start + Cartridge::TRAINER_LEN;

//         if end > buf.len() {
//             return Err(String::from("Could not read trainer area even though trainer area expected"));
//         }

//         for i in start..end {
//             trainer[i - start] = buf[i];
//         }

//         *cart_pos = end;

//         Ok(trainer)
//     }

//     fn read_prg_rom_from_buf(&self, buf: &[u8], cart_pos: &mut usize, prg_rom_len: usize) -> Result<Vec<u8>, String> {
//         let mut prg_rom = Vec::new();

//         let start = *cart_pos;
//         let end = start + prg_rom_len;

//         if end > buf.len() {
//             return Err(String::from("Failed to read program ROM (hit EOF before finished reading)"));
//         }

//         for i in start..end {
//             prg_rom.push(buf[i]);
//         }

//         *cart_pos = end;

//         Ok(prg_rom)
//     }

//     fn read_chr_rom_from_buf(&self, buf: &[u8], cart_pos: &mut usize, chr_rom_len: usize) -> Result<Vec<u8>, String> {
//         let mut chr_rom = Vec::new();

//         let start = *cart_pos;
//         let end = start + chr_rom_len;

//         if end > buf.len() {
//             return Err(String::from("Failed to read program ROM (hit EOF before finished reading)"));
//         }

//         for i in start..end {
//             chr_rom.push(buf[i]);
//         }

//         *cart_pos = end;

//         Ok(chr_rom)
//     }

//     fn read_misc_rom_from_buf(&self, buf: &[u8], cart_pos: &mut usize) -> Vec<u8> {
//         let mut misc_rom = Vec::new();

//         let start = *cart_pos;
//         let end = buf.len();

//         for i in start..end {
//             misc_rom.push(buf[i]);
//         }

//         *cart_pos = end;

//         misc_rom
//     }

//     pub fn read_cart(&self, cart_path: &str) -> Result<Cartridge, String> {
//         info!("Attempting to read cartridge from {0}", cart_path);

//         let mut cart = Cartridge::new();
//         let cart_data = self.read_cart_to_buf(cart_path)?;

//         if let Err(err_message) = cart.parse_header(&cart_data) {
//             return Err(err_message);
//         }

//         info!("  [Succesfully read cartridge header]");

//         let mut cart_pos = Cartridge::HEADER_LEN;

//         if cart.has_trainer_area() {
//             let t_data = self.read_trainer_data_from_buf(&cart_data, &mut cart_pos)?;
//             cart.set_trainer_data(&t_data);

//             info!("  [Successfully read trainer area]");
//         }

//         let prg_rom_size = cart.get_prg_rom_size();
//         let prg_rom = self.read_prg_rom_from_buf(&cart_data, &mut cart_pos, prg_rom_size)?;
//         cart.set_prg_rom(prg_rom);

//         let chr_rom_size = cart.get_chr_rom_size();
//         let chr_rom = self.read_chr_rom_from_buf(&cart_data, &mut cart_pos, chr_rom_size)?;
//         cart.set_chr_rom(chr_rom);

//         let misc_rom = self.read_misc_rom_from_buf(&cart_data, &mut cart_pos);
//         cart.set_misc_rom(misc_rom);

//         Ok(cart)
//     }
// }


// #[cfg(test)]
// mod tests {
//     use crate::cartridge::cartridge::{CartFormat, NES_2V0_IDENT};
//     use super::*;

//     #[test]
//     fn test_header_parse() {
//         let header_data0: [u8; 16] = [0; 16]; // invalid format
//         let header_data1: [u8; 16] = [b'N', b'E', b'S', 0x1A, // INES format && extended console type
//                                       0x15, 0x54, 0x4E, 0x73,
//                                       0xB1, 0xAE, 0x04, 0xF0,
//                                       0xFE, 0x76, 0x22, 0xF9];
//         let header_data2: [u8; 16] = [b'N', b'E', b'S', 0x1A, // V2NES format && vs console type
//                                       0x15, 0x54, 0x4E, 0x79,
//                                       0xB1, 0xFF, 0x04, 0xF0,
//                                       0xFE, 0x76, 0x22, 0xF9];

//         let mut cart = Cartridge::new();
//         let result = cart.parse_header(&header_data0);

//         assert!(result.is_err());
//         assert_eq!(*cart.get_format(), CartFormat::Unknown);
//         assert_eq!(cart.get_ident(), String::from("Unknown Header"));

//         let mut cart = Cartridge::new();
//         let result = cart.parse_header(&header_data1);

//         assert!(result.is_ok());
//         assert_eq!(*cart.get_format(), CartFormat::INES); // not V2NES bc the identifier flags in byte 7 != 0b10 (required for v2NES, else iNES)
//         assert_eq!(cart.get_ident(), String::from("NES\u{1A}"));
//         assert_eq!(cart.get_prg_rom_size(), 0xE15);
//         assert_eq!(cart.get_chr_rom_size(), 0xA54);
//         assert_eq!(cart.has_alt_nametables(), true);
//         assert_eq!(cart.has_trainer_area(), true);
//         assert_eq!(cart.has_battery(), true);
//         assert_eq!(cart.has_hardwired_nametable(), false);
//         assert_eq!(cart.get_mapper_num(), 0x174);
//         assert_eq!(cart.get_console_type(), 0x3);
//         assert_eq!(cart.get_submapper_num(), 0xB);
//         assert_eq!(cart.get_prg_nv_ram_size(), 0x0);
//         assert_eq!(cart.get_prg_ram_size(), 64<<0x4);
//         assert_eq!(cart.get_chr_nv_ram_size(), 64<<0xF);
//         assert_eq!(cart.get_chr_ram_size(), 0x0);
//         assert_eq!(cart.get_timing_mode(), 0x2);
//         assert_eq!(cart.get_vs_hardware_type(), 0x0);
//         assert_eq!(cart.get_vs_ppu_type(), 0x0);
//         assert_eq!(cart.get_extended_console_type(), 0x6);
//         assert_eq!(cart.get_misc_rom_count(), 0x2);
//         assert_eq!(cart.get_default_expansion_device(), 0x39);

//         let mut cart = Cartridge::new();
//         let result = cart.parse_header(&header_data2);

//         assert!(result.is_ok());
//         assert_eq!(*cart.get_format(), CartFormat::V2NES); // Now flags in byte 7 match, so using NES 2.0
//         assert_eq!(cart.get_prg_rom_size(), 0x60); // Exp & Mult mode bc MSB of PRG ROM == 0xF
//         assert_eq!(cart.get_chr_rom_size(), 0x200000); // Also exp & mult mode bc MSB of CHR ROM == 0xF
//         assert_eq!(cart.get_vs_hardware_type(), 0x7);
//         assert_eq!(cart.get_vs_ppu_type(), 0x6);
//         assert_eq!(cart.get_extended_console_type(), 0x0);
//     }

//     #[test]
//     fn test_cart_read() {
//         let reader = Reader::new();

//         let cart_result = reader.read_cart("File/that/does/not/exist.nes");

//         assert!(cart_result.is_err());

//         if let Err(message) = cart_result {
//             assert_eq!(message, String::from("Could not find file 'File/that/does/not/exist.nes'"));
//         }

//         let cart_result = reader.read_cart("branch_timing_tests/1.Branch_Basics.nes");

//         assert!(cart_result.is_ok());

//         let cart = cart_result.unwrap();

//         let expected_ident = std::str::from_utf8(&NES_2V0_IDENT).unwrap();

//         assert_eq!(cart.get_ident(), expected_ident);
//         assert_eq!(*cart.get_format(), CartFormat::INES);
//         assert_eq!(cart.get_trainer_area(), None);

//         println!("Program ROM Size: {}", cart.get_prg_rom_size());
//     }
// }
