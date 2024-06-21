use chars::{CHAR_HEIGHT, CHAR_WIDTH};

use crate::{cartridge::mapper::NametableMirror, system::{controller::ControllerButton, nes::Nes}};

use super::app::{PauseMenu, PauseMenuItem};

pub const DEBUG_FRAME_WIDTH: usize = 960;
pub const DEBUG_FRAME_HEIGHT: usize = 540;

pub const GAME_FRAME_WIDTH: usize = 256;
pub const GAME_FRAME_HEIGHT: usize = 240;

pub mod chars {
    pub const CHAR_WIDTH: usize = 7;
    pub const CHAR_HEIGHT: usize = 8;
    /// Number of pixels left between newlines
    pub const NEWLINE_PADDING: usize = 3;

    pub const A: u64 = 
        0b0111_0000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b1000_1000 << 5*8 |
        0b1111_1000 << 4*8 |
        0b1000_1000 << 3*8 |
        0b1000_1000 << 2*8 |
        0b1000_1000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const B: u64 = 
        0b1111_0000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b1000_1000 << 5*8 |
        0b1111_0000 << 4*8 |
        0b1000_1000 << 3*8 |
        0b1000_1000 << 2*8 |
        0b1111_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const C: u64 = 
        0b0111_0000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b1000_0000 << 5*8 |
        0b1000_0000 << 4*8 |
        0b1000_0000 << 3*8 |
        0b1000_1000 << 2*8 |
        0b0111_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const D: u64 = 
        0b1111_0000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b1000_1000 << 5*8 |
        0b1000_1000 << 4*8 |
        0b1000_1000 << 3*8 |
        0b1000_1000 << 2*8 |
        0b1111_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const E: u64 = 
        0b1111_1000 << 7*8 |
        0b1000_0000 << 6*8 |
        0b1000_0000 << 5*8 |
        0b1111_0000 << 4*8 |
        0b1000_0000 << 3*8 |
        0b1000_0000 << 2*8 |
        0b1111_1000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const F: u64 = 
        0b1111_1000 << 7*8 |
        0b1000_0000 << 6*8 |
        0b1000_0000 << 5*8 |
        0b1111_0000 << 4*8 |
        0b1000_0000 << 3*8 |
        0b1000_0000 << 2*8 |
        0b1000_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const G: u64 = 
        0b0111_0000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b1000_0000 << 5*8 |
        0b1000_0000 << 4*8 |
        0b1001_1000 << 3*8 |
        0b1000_1000 << 2*8 |
        0b0111_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const H: u64 = 
        0b1000_1000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b1000_1000 << 5*8 |
        0b1111_1000 << 4*8 |
        0b1000_1000 << 3*8 |
        0b1000_1000 << 2*8 |
        0b1000_1000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const I: u64 = 
        0b1111_1000 << 7*8 |
        0b0010_0000 << 6*8 |
        0b0010_0000 << 5*8 |
        0b0010_0000 << 4*8 |
        0b0010_0000 << 3*8 |
        0b0010_0000 << 2*8 |
        0b1111_1000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const J: u64 = 
        0b0011_1000 << 7*8 |
        0b0001_0000 << 6*8 |
        0b0001_0000 << 5*8 |
        0b0001_0000 << 4*8 |
        0b0001_0000 << 3*8 |
        0b1001_0000 << 2*8 |
        0b0110_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const K: u64 = 
        0b1000_1000 << 7*8 |
        0b1001_0000 << 6*8 |
        0b1010_0000 << 5*8 |
        0b1100_0000 << 4*8 |
        0b1010_0000 << 3*8 |
        0b1001_0000 << 2*8 |
        0b1000_1000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const L: u64 = 
        0b1000_0000 << 7*8 |
        0b1000_0000 << 6*8 |
        0b1000_0000 << 5*8 |
        0b1000_0000 << 4*8 |
        0b1000_0000 << 3*8 |
        0b1000_0000 << 2*8 |
        0b1111_1000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const M: u64 = 
        0b1000_1000 << 7*8 |
        0b1101_1000 << 6*8 |
        0b1010_1000 << 5*8 |
        0b1010_1000 << 4*8 |
        0b1000_1000 << 3*8 |
        0b1000_1000 << 2*8 |
        0b1000_1000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const N: u64 = 
        0b1000_1000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b1100_1000 << 5*8 |
        0b1010_1000 << 4*8 |
        0b1001_1000 << 3*8 |
        0b1000_1000 << 2*8 |
        0b1000_1000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const O: u64 = 
        0b0111_0000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b1000_1000 << 5*8 |
        0b1000_1000 << 4*8 |
        0b1000_1000 << 3*8 |
        0b1000_1000 << 2*8 |
        0b0111_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const P: u64 = 
        0b1111_0000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b1000_1000 << 5*8 |
        0b1111_0000 << 4*8 |
        0b1000_0000 << 3*8 |
        0b1000_0000 << 2*8 |
        0b1000_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const Q: u64 = 
        0b0111_0000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b1000_1000 << 5*8 |
        0b1000_1000 << 4*8 |
        0b1010_1000 << 3*8 |
        0b1001_0000 << 2*8 |
        0b0110_1000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const R: u64 = 
        0b1111_0000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b1000_1000 << 5*8 |
        0b1111_0000 << 4*8 |
        0b1010_0000 << 3*8 |
        0b1001_0000 << 2*8 |
        0b1000_1000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const S: u64 = 
        0b0111_1000 << 7*8 |
        0b1000_0000 << 6*8 |
        0b1000_0000 << 5*8 |
        0b0111_0000 << 4*8 |
        0b0000_1000 << 3*8 |
        0b0000_1000 << 2*8 |
        0b1111_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const T: u64 = 
        0b1111_1000 << 7*8 |
        0b0010_0000 << 6*8 |
        0b0010_0000 << 5*8 |
        0b0010_0000 << 4*8 |
        0b0010_0000 << 3*8 |
        0b0010_0000 << 2*8 |
        0b0010_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const U: u64 = 
        0b1000_1000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b1000_1000 << 5*8 |
        0b1000_1000 << 4*8 |
        0b1000_1000 << 3*8 |
        0b1000_1000 << 2*8 |
        0b0111_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const V: u64 = 
        0b1000_1000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b1000_1000 << 5*8 |
        0b1000_1000 << 4*8 |
        0b0101_0000 << 3*8 |
        0b0101_0000 << 2*8 |
        0b0010_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const W: u64 = 
        0b1000_1000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b1000_1000 << 5*8 |
        0b1010_1000 << 4*8 |
        0b1010_1000 << 3*8 |
        0b1010_1000 << 2*8 |
        0b0101_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const X: u64 = 
        0b1000_1000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b0101_0000 << 5*8 |
        0b0010_0000 << 4*8 |
        0b0101_0000 << 3*8 |
        0b1000_1000 << 2*8 |
        0b1000_1000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const Y: u64 = 
        0b1000_1000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b0101_0000 << 5*8 |
        0b0010_0000 << 4*8 |
        0b0010_0000 << 3*8 |
        0b0010_0000 << 2*8 |
        0b0010_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const Z: u64 = 
        0b1111_1000 << 7*8 |
        0b0000_1000 << 6*8 |
        0b0001_0000 << 5*8 |
        0b0010_0000 << 4*8 |
        0b0100_0000 << 3*8 |
        0b1000_0000 << 2*8 |
        0b1111_1000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const SPACE: u64 = 
        0b0000_0000 << 7*8 |
        0b0000_0000 << 6*8 |
        0b0000_0000 << 5*8 |
        0b0000_0000 << 4*8 |
        0b0000_0000 << 3*8 |
        0b0000_0000 << 2*8 |
        0b0000_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const ZERO: u64 = 
        0b0111_0000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b1001_1000 << 5*8 |
        0b1010_1000 << 4*8 |
        0b1100_1000 << 3*8 |
        0b1000_1000 << 2*8 |
        0b0111_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const ONE: u64 = 
        0b0010_0000 << 7*8 |
        0b1110_0000 << 6*8 |
        0b0010_0000 << 5*8 |
        0b0010_0000 << 4*8 |
        0b0010_0000 << 3*8 |
        0b0010_0000 << 2*8 |
        0b1111_1000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const TWO: u64 = 
        0b0111_0000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b0000_1000 << 5*8 |
        0b0001_0000 << 4*8 |
        0b0010_0000 << 3*8 |
        0b0100_0000 << 2*8 |
        0b1111_1000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const THREE: u64 = 
        0b1111_1000 << 7*8 |
        0b0001_0000 << 6*8 |
        0b0010_0000 << 5*8 |
        0b0001_0000 << 4*8 |
        0b0000_1000 << 3*8 |
        0b1000_1000 << 2*8 |
        0b0111_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const FOUR: u64 = 
        0b0001_0000 << 7*8 |
        0b0011_0000 << 6*8 |
        0b0101_0000 << 5*8 |
        0b1001_0000 << 4*8 |
        0b1111_1000 << 3*8 |
        0b0001_0000 << 2*8 |
        0b0001_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const FIVE: u64 = 
        0b1111_1000 << 7*8 |
        0b1000_0000 << 6*8 |
        0b1111_0000 << 5*8 |
        0b0000_1000 << 4*8 |
        0b0000_1000 << 3*8 |
        0b1000_1000 << 2*8 |
        0b0111_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const SIX: u64 = 
        0b0011_0000 << 7*8 |
        0b0100_0000 << 6*8 |
        0b1000_0000 << 5*8 |
        0b1111_0000 << 4*8 |
        0b1000_1000 << 3*8 |
        0b1000_1000 << 2*8 |
        0b0111_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const SEVEN: u64 = 
        0b1111_1000 << 7*8 |
        0b0000_1000 << 6*8 |
        0b0001_0000 << 5*8 |
        0b0010_0000 << 4*8 |
        0b0100_0000 << 3*8 |
        0b0100_0000 << 2*8 |
        0b0100_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const EIGHT: u64 = 
        0b0111_0000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b1000_1000 << 5*8 |
        0b0111_0000 << 4*8 |
        0b1000_1000 << 3*8 |
        0b1000_1000 << 2*8 |
        0b0111_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const NINE: u64 = 
        0b0111_0000 << 7*8 |
        0b1000_1000 << 6*8 |
        0b1000_1000 << 5*8 |
        0b0111_1000 << 4*8 |
        0b0000_1000 << 3*8 |
        0b0001_0000 << 2*8 |
        0b0110_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const DOLLAR: u64 = 
        0b0010_0000 << 7*8 |
        0b0111_1000 << 6*8 |
        0b1010_0000 << 5*8 |
        0b0111_0000 << 4*8 |
        0b0010_1000 << 3*8 |
        0b1111_0000 << 2*8 |
        0b0010_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const COLON: u64 = 
        0b0000_0000 << 7*8 |
        0b0100_0000 << 6*8 |
        0b0000_0000 << 5*8 |
        0b0000_0000 << 4*8 |
        0b0000_0000 << 3*8 |
        0b0100_0000 << 2*8 |
        0b0000_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const LBRACE: u64 = 
        0b0001_1000 << 7*8 |
        0b0010_0000 << 6*8 |
        0b0010_0000 << 5*8 |
        0b0100_0000 << 4*8 |
        0b0010_0000 << 3*8 |
        0b0010_0000 << 2*8 |
        0b0001_1000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const RBRACE: u64 = 
        0b1100_0000 << 7*8 |
        0b0010_0000 << 6*8 |
        0b0010_0000 << 5*8 |
        0b0001_0000 << 4*8 |
        0b0010_0000 << 3*8 |
        0b0010_0000 << 2*8 |
        0b1100_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const LANGLE: u64 = 
        0b0000_0100 << 7*8 |
        0b0000_1000 << 6*8 |
        0b0001_0000 << 5*8 |
        0b0010_0000 << 4*8 |
        0b0001_0000 << 3*8 |
        0b0000_1000 << 2*8 |
        0b0000_0100 << 1*8 |
        0b0000_0000 << 0*8;
    pub const RANGLE: u64 = 
        0b0100_0000 << 7*8 |
        0b0010_0000 << 6*8 |
        0b0001_0000 << 5*8 |
        0b0010_1000 << 4*8 |
        0b0001_0000 << 3*8 |
        0b0010_0000 << 2*8 |
        0b0100_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const LSQR: u64 = 
        0b0011_1000 << 7*8 |
        0b0010_0000 << 6*8 |
        0b0010_0000 << 5*8 |
        0b0010_0000 << 4*8 |
        0b0010_0000 << 3*8 |
        0b0010_0000 << 2*8 |
        0b0011_1000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const RSQR: u64 = 
        0b1110_0000 << 7*8 |
        0b0010_0000 << 6*8 |
        0b0010_0000 << 5*8 |
        0b0010_0000 << 4*8 |
        0b0010_0000 << 3*8 |
        0b0010_0000 << 2*8 |
        0b1110_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const HASH: u64 = 
        0b0101_0000 << 7*8 |
        0b0101_0000 << 6*8 |
        0b1111_1000 << 5*8 |
        0b0101_0000 << 4*8 |
        0b1111_1000 << 3*8 |
        0b0101_0000 << 2*8 |
        0b0101_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const DASH: u64 = 
        0b0000_0000 << 7*8 |
        0b0000_0000 << 6*8 |
        0b0000_0000 << 5*8 |
        0b1111_1000 << 4*8 |
        0b0000_0000 << 3*8 |
        0b0000_0000 << 2*8 |
        0b0000_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const EQUAL: u64 = 
        0b0000_0000 << 7*8 |
        0b0000_0000 << 6*8 |
        0b1111_1000 << 5*8 |
        0b0000_0000 << 4*8 |
        0b1111_1000 << 3*8 |
        0b0000_0000 << 2*8 |
        0b0000_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const COMMA: u64 = 
        0b0000_0000 << 7*8 |
        0b0000_0000 << 6*8 |
        0b0000_0000 << 5*8 |
        0b0000_0000 << 4*8 |
        0b0110_0000 << 3*8 |
        0b0110_0000 << 2*8 |
        0b0010_0000 << 1*8 |
        0b0100_0000 << 0*8;
    pub const FORWARD_SLASH: u64 = 
        0b0000_0100 << 7*8 |
        0b0000_1100 << 6*8 |
        0b0001_1000 << 5*8 |
        0b0011_0000 << 4*8 |
        0b0110_0000 << 3*8 |
        0b1100_0000 << 2*8 |
        0b1000_0000 << 1*8 |
        0b0000_0000 << 0*8;
    pub const PERCENT: u64 = 
        0b1110_0100 << 7*8 |
        0b1010_1100 << 6*8 |
        0b1101_1000 << 5*8 |
        0b0011_0000 << 4*8 |
        0b0110_1100 << 3*8 |
        0b1101_0100 << 2*8 |
        0b1001_1100 << 1*8 |
        0b0000_0000 << 0*8;

    // Characters wrapped in `backticks` in a string will be considered special.
    // These characters map to their special counterpart if one exists, else
    // it is treated like normal.
    pub mod special {
        pub const ARROW_SELECT: u64 = 
            0b1110_0000 << 7*8 |
            0b1111_0000 << 6*8 |
            0b1111_1000 << 5*8 |
            0b1111_1100 << 4*8 |
            0b1111_1000 << 3*8 |
            0b1111_0000 << 2*8 |
            0b1110_0000 << 1*8 |
            0b0000_0000 << 0*8;
    }

    // # # # # # # # #
    // # . . # # . . #
    // # . # . . # . #
    // # . . . . # . #
    // # . . # # . . #
    // # . . . . . . #
    // # . . # . . . #
    // # # # # # # # #
    pub const UNKNOWN: u64 = 
        0b1111_1111 << 7*8 |
        0b1001_1001 << 6*8 |
        0b1010_0101 << 5*8 |
        0b1000_0101 << 4*8 |
        0b1001_1001 << 3*8 |
        0b1000_0001 << 2*8 |
        0b1001_0001 << 1*8 |
        0b1111_1111 << 0*8;

    pub fn get_letter_from_char(letter_char: char, special_char: bool) -> u64 {
        match letter_char.to_ascii_uppercase() {
            'A' => A,
            'B' => B,
            'C' => C,
            'D' => D,
            'E' => E,
            'F' => F,
            'G' => G,
            'H' => H,
            'I' => I,
            'J' => J,
            'K' => K,
            'L' => L,
            'M' => M,
            'N' => N,
            'O' => O,
            'P' => P,
            'Q' => Q,
            'R' => R,
            'S' => S,
            'T' => T,
            'U' => U,
            'V' => V,
            'W' => W,
            'X' => X,
            'Y' => Y,
            'Z' => Z,
            '0' => ZERO,
            '1' => ONE,
            '2' => TWO,
            '3' => THREE,
            '4' => FOUR,
            '5' => FIVE,
            '6' => SIX,
            '7' => SEVEN,
            '8' => EIGHT,
            '9' => NINE,
            '$' => DOLLAR,
            ':' => COLON,
            '{' => LBRACE,
            '}' => RBRACE,
            '<' => LANGLE,
            '>' => if special_char { special::ARROW_SELECT } else { RANGLE },
            '[' => LSQR,
            ']' => RSQR,
            '#' => HASH,
            '-' => DASH,
            '=' => EQUAL,
            ',' => COMMA,
            '/' => FORWARD_SLASH,
            '%' => PERCENT,
            ' ' => SPACE,

            _ => UNKNOWN,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<&[u8]> for Color {
    fn from(value: &[u8]) -> Self {
        match value.len() {
            0 => Color::default(),
            1 => Color{
                r: value[0], 
                ..Default::default()
            },
            2 => Color{
                r: value[0], 
                g: value[1],
                ..Default::default()
            },
            _ => Color{
                r: value[0],
                g: value[1],
                b: value[2],
            }
        }
    }
}

pub const RED: Color = Color{r: 255, g: 0, b: 0};
pub const GREEN: Color = Color{r: 0, g: 255, b: 0};
pub const BLUE: Color = Color{r: 0, g: 0, b: 255};
pub const BLACK: Color = Color{r: 0, g: 0, b: 0};
pub const WHITE: Color = Color{r: 255, g: 255, b: 255};
pub const GREY: Color = Color{r: 128, g: 128, b: 128};

#[derive(Clone, Copy)]
pub struct DebugPalette {
    pub bg_col: Color,
    pub txt_col: Color,
    pub border_col: Color,
    pub ok_col: Color,
    pub err_col: Color,
}

pub const DEFAULT_DEBUG_PAL: DebugPalette = DebugPalette {
    // bg_col: Color{r: 50, g: 69, b: 62},
    bg_col: Color{r: 0, g: 0, b: 0},
    txt_col: Color{r: 0xF4, g: 0xE2, b: 0x85},
    border_col: Color{r: 0xF4, g: 0xA2, b: 0x59},
    ok_col: Color{r: 0x8C, g: 0xB3, b: 0x69},
    err_col: Color{r: 0xBC, g: 0x4B, b: 0x51},
};


const DEBUG_NES_SCREEN_X: usize = 9;
const DEBUG_NES_SCREEN_Y: usize = 38;
const DEBUG_CPU_STATE_X: usize = 543;
const DEBUG_CPU_STATE_Y: usize = 45;
const DEBUG_ZPAGE_STATE_X: usize = 543;
const DEBUG_ZPAGE_STATE_Y: usize = 161;
const DEBUG_PGTBL1_VIEW_X: usize = 546;
const DEBUG_PGTBL1_VIEW_Y: usize = 368;
const DEBUG_PGTBL2_VIEW_X: usize = 690;
const DEBUG_PGTBL2_VIEW_Y: usize = 368;
const DEBUG_FPS_COUNTER_X: usize = 835;
const DEBUG_FPS_COUNTER_Y: usize = 520;

const MENU_CONTROLLER_X: usize = 27;
const MENU_CONTROLLER_Y: usize = 145;
const MENU_VOLUME_SLIDER_X: usize = 9;
const MENU_VOLUME_SLIDER_Y: usize = 154;

/// Function to draw a dot to the frame at some (x,y) pair. Doesn't check for
/// out of bounds. Draws the dot with the top-left pixel at (x,y), not centered.
pub fn dot(frame: &mut [u8], frame_width: usize, frame_height: usize, x: usize, 
        y: usize, size: usize, color: Color) {
    
    // Handle sizes < 3 manually for speed
    if size == 0 {
        return;
    } else if size == 1 {
        let pix_idx = (y * frame_width + x)*4;
        frame[pix_idx + 0] = color.r;
        frame[pix_idx + 1] = color.g;
        frame[pix_idx + 2] = color.b;
        frame[pix_idx + 3] = 0xFF;
    } else if size == 2 {
        let top_left_idx = (y * frame_width + x)*4;
        frame[top_left_idx + 0] = color.r;
        frame[top_left_idx + 1] = color.g;
        frame[top_left_idx + 2] = color.b;
        frame[top_left_idx + 3] = 0xFF;

        let top_right_idx = (y * frame_width + x + 1)*4;
        frame[top_right_idx + 0] = color.r;
        frame[top_right_idx + 1] = color.g;
        frame[top_right_idx + 2] = color.b;
        frame[top_right_idx + 3] = 0xFF;

        let bottom_left_idx = ((y+1) * frame_width + x)*4;
        frame[bottom_left_idx + 0] = color.r;
        frame[bottom_left_idx + 1] = color.g;
        frame[bottom_left_idx + 2] = color.b;
        frame[bottom_left_idx + 3] = 0xFF;

        let bottom_right_idx = ((y+1) * frame_width + x + 1)*4;
        frame[bottom_right_idx + 0] = color.r;
        frame[bottom_right_idx + 1] = color.g;
        frame[bottom_right_idx + 2] = color.b;
        frame[bottom_right_idx + 3] = 0xFF;
    } else {
        for i in 0..size {
            for j in 0..size {
                let pix_idx = ((y+i) * frame_width + x + j)*4;
                frame[pix_idx + 0] = color.r;
                frame[pix_idx + 1] = color.g;
                frame[pix_idx + 2] = color.b;
                frame[pix_idx + 3] = 0xFF;
            }
        }
    }
}

/// Draw a single character to the frame buffer with the top left most pixels at
/// (x_pos, y_pos). Also takes the frame width & height in pixels to ensure no
/// pixels are drawn outside of the frame.
pub fn draw_char(frame: &mut [u8], frame_width: usize, frame_height: usize, 
                character: char, x_pos: usize, y_pos: usize, chr_col: Color, 
                bg_col: Color, text_size: usize, special_char: bool) {
    
    const ROW0_MASK: u64 = 0x80_00_00_00_00_00_00_00;
    const ROW1_MASK: u64 = 0x00_80_00_00_00_00_00_00;
    const ROW2_MASK: u64 = 0x00_00_80_00_00_00_00_00;
    const ROW3_MASK: u64 = 0x00_00_00_80_00_00_00_00;
    const ROW4_MASK: u64 = 0x00_00_00_00_80_00_00_00;
    const ROW5_MASK: u64 = 0x00_00_00_00_00_80_00_00;
    const ROW6_MASK: u64 = 0x00_00_00_00_00_00_80_00;
    const ROW7_MASK: u64 = 0x00_00_00_00_00_00_00_80;

    let mut bitmap = chars::get_letter_from_char(character, special_char);
    let s = text_size;

    for i in 0..8 {
        dot(frame, frame_width, frame_height, x_pos + i*s, y_pos + 0*s, s, if bitmap & ROW0_MASK != 0 { chr_col } else { bg_col });
        dot(frame, frame_width, frame_height, x_pos + i*s, y_pos + 1*s, s, if bitmap & ROW1_MASK != 0 { chr_col } else { bg_col });
        dot(frame, frame_width, frame_height, x_pos + i*s, y_pos + 2*s, s, if bitmap & ROW2_MASK != 0 { chr_col } else { bg_col });
        dot(frame, frame_width, frame_height, x_pos + i*s, y_pos + 3*s, s, if bitmap & ROW3_MASK != 0 { chr_col } else { bg_col });
        dot(frame, frame_width, frame_height, x_pos + i*s, y_pos + 4*s, s, if bitmap & ROW4_MASK != 0 { chr_col } else { bg_col });
        dot(frame, frame_width, frame_height, x_pos + i*s, y_pos + 5*s, s, if bitmap & ROW5_MASK != 0 { chr_col } else { bg_col });
        dot(frame, frame_width, frame_height, x_pos + i*s, y_pos + 6*s, s, if bitmap & ROW6_MASK != 0 { chr_col } else { bg_col });
        dot(frame, frame_width, frame_height, x_pos + i*s, y_pos + 7*s, s, if bitmap & ROW7_MASK != 0 { chr_col } else { bg_col });
        bitmap <<= 1;
    }
}

/// Draw a string of characters to the frame buffer with the top left most 
/// pixels at (x_pos, y_pos). Also takes the frame width & height in pixels to 
/// ensure no pixels are drawn outside of the frame. Returns an (x, y)
/// coordinate for where the nect character would go.
pub fn draw_string(frame: &mut [u8], frame_width: usize, frame_height: usize, 
                text: &str, x_pos: usize, y_pos: usize, chr_col: Color, 
                bg_col: Color, text_size: usize) -> (usize, usize) {

    let mut curr_x = x_pos;
    let mut curr_y = y_pos;

    let mut horizontal_step = text_size * (chars::CHAR_WIDTH - 1) + 1;;
    let mut vertical_step = text_size * chars::CHAR_HEIGHT + chars::NEWLINE_PADDING;

    let mut special = false;
    for character in text.chars().into_iter() {
        if character == '\n' {
            curr_x = x_pos;
            curr_y += vertical_step;
            continue;
        }

        // backtick toggles the "special" flag
        if character == '`' {
            special = !special;
            continue;
        }

        if !special || character != ' ' {
            draw_char(frame, frame_width, frame_height, character, curr_x, curr_y, chr_col, bg_col, text_size, special); 
        }

        curr_x += horizontal_step;       
    }

    (curr_x, curr_y)
}

pub fn horizontal_line(frame: &mut [u8], frame_width: usize, frame_height: usize, 
                    x_start: usize, x_end: usize, y: usize, thickness: usize, 
                    color: Color) {

    let mut x_start = x_start;
    let mut x_end = x_end;

    // there is no line bruh
    if thickness == 0 {
        return;
    }
    // line is off the screen
    if y >= frame_height {
        return;
    }
    // Ensure x_start is smaller than x_end
    if x_start > x_end {
        std::mem::swap(&mut x_start, &mut x_end);
    }
    // line is off the screen
    if x_start >= frame_width {
        return;
    }
    // clamp the line to on the screen
    if x_end + thickness > frame_width {
        x_end = frame_width - thickness;
        x_start = std::cmp::min(x_start, x_end);
    }

    for x in (x_start..x_end).step_by(thickness) {
        dot(frame, frame_width, frame_height, x, y, thickness, color);
    }
}

pub fn vertical_line(frame: &mut [u8], frame_width: usize, frame_height: usize, 
    y_start: usize, y_end: usize, x: usize, thickness: usize, 
    color: Color) {

    let mut y_start = y_start;
    let mut y_end = y_end;

    // there is no line bruh
    if thickness == 0 {
        return;
    }
    // line is off the screen
    if x >= frame_width {
        return;
    }
    // Ensure x_start is smaller than x_end
    if y_start > y_end {
        std::mem::swap(&mut y_start, &mut y_end);
    }
    // line is off the screen
    if y_start >= frame_height {
        return;
    }
    // clamp the line to on the screen
    if y_end + thickness > frame_height {
        y_end = frame_height - thickness;
        y_start = std::cmp::min(y_start, y_end);
    }

    for y in (y_start..y_end).step_by(thickness) {
        dot(frame, frame_width, frame_height, x, y, thickness, color);
    }
}

pub fn draw_box(frame: &mut [u8], frame_width: usize, frame_height: usize,
                x: usize, y: usize, width: usize, height: usize, 
                thickness: usize, palette: DebugPalette, title: Option<&str>) {
    
    // Top line (with optional title)
    if let Some(text) = title {
        const TITLE_OFFSET: f64 = 0.2;
        const TITLE_PADDING: usize = 5;

        let title_x = x + (width as f64 * TITLE_OFFSET) as usize;
        let title_y = y - chars::CHAR_HEIGHT; // Titles use large chars, so 1/2 large char height is just char height

        let title_width = text.len() * (chars::CHAR_WIDTH * 2 - 1) - 5;

        horizontal_line(frame, frame_width, frame_height, x, title_x - TITLE_PADDING, y, thickness, palette.border_col);
        horizontal_line(frame, frame_width, frame_height, title_x + title_width + TITLE_PADDING, x + width, y, thickness, palette.border_col);

        draw_string(frame, frame_width, frame_height, text, title_x, title_y, palette.border_col, palette.bg_col, 2);
    } else {
        horizontal_line(frame, frame_width, frame_height, x, x + width, y, thickness, palette.border_col);
    }

    // Bottom line (+ thickness for x_end to fill little gap artifact)
    horizontal_line(frame, frame_width, frame_height, x, x + width + 1, y + height, thickness, palette.border_col);
    // Left line
    vertical_line(frame, frame_width, frame_height, y, y + height, x, thickness, palette.border_col);
    // Right line
    vertical_line(frame, frame_width, frame_height, y, y + height, x + width, thickness, palette.border_col);

}


pub fn draw_nes_screen(frame: &mut [u8], frame_width: usize, frame_height: usize, 
                    screen_buf: &[u8], x: usize, y: usize, double_size: bool) {
    
    let s = if double_size { 2 } else { 1 };

    for (py, row) in screen_buf.chunks(256*4).enumerate() {
        for (px, pix) in row.chunks(4).enumerate() {
            dot(frame, frame_width, frame_height, x+s*px, y+s*py, s, Color::from(pix));
        }
    }
}

pub fn draw_nes_pagetable_8x8(frame: &mut [u8], frame_width: usize, frame_height: usize, 
                        pagetable: Box<[u8; 0x1000]>, x: usize, y: usize) {
    
    const GREYSCALE: [Color; 4] = [
        Color{r: 0x40, g: 0x40, b: 0x40},
        Color{r: 0x80, g: 0x80, b: 0x80},
        Color{r: 0xC0, g: 0xC0, b: 0xC0},
        Color{r: 0xF0, g: 0xF0, b: 0xF0},
    ];

    const SPRITE_WIDTH: usize = 8; // Num pixels per side of sprite
    const PGTBL_WIDTH: usize = 16; // Num sprites per side of pagetable
    
    for spr_y in 0..PGTBL_WIDTH {
        for spr_x in 0..PGTBL_WIDTH {
            let spr_idx = spr_y * PGTBL_WIDTH + spr_x;
            let sprite_bytes = &pagetable[spr_idx*PGTBL_WIDTH..(spr_idx+1)*PGTBL_WIDTH];

            for r in 0..8 {
                let lsb_byte = sprite_bytes[r];
                let msb_byte = sprite_bytes[r + 8];

                for c in 0..8 {
                    let col_idx = (((msb_byte >> (7 - c)) & 1) << 1) | ((lsb_byte >> (7 - c)) & 1);
                    let col = GREYSCALE[col_idx as usize];

                    let pixel_x = x + spr_x*SPRITE_WIDTH + c;
                    let pixel_y = y + spr_y*SPRITE_WIDTH + r;

                    dot(frame, frame_width, frame_height, pixel_x, pixel_y, 1, col);
                }
            }
        }
    }
}

pub fn draw_nes_pagetable_8x16(frame: &mut [u8], frame_width: usize, frame_height: usize, 
                        pagetable: Box<[u8; 0x1000]>, x: usize, y: usize) {

    const GREYSCALE: [Color; 4] = [
        Color{r: 0x40, g: 0x40, b: 0x40},
        Color{r: 0x80, g: 0x80, b: 0x80},
        Color{r: 0xC0, g: 0xC0, b: 0xC0},
        Color{r: 0xF0, g: 0xF0, b: 0xF0},
    ];

    const SPRITE_WIDTH: usize = 8; // Num pixels per side of sprite
    const PGTBL_WIDTH: usize = 16; // Num sprites per side of pagetable

    for spr_y in 0..PGTBL_WIDTH {
        for spr_x in 0..PGTBL_WIDTH {
            let spr_idx = (spr_x * 2 + spr_y % 2) + (spr_y / 2) * PGTBL_WIDTH;

            // let spr_idx = spr_y * PGTBL_WIDTH + spr_x;
            let sprite_bytes = &pagetable[spr_idx*PGTBL_WIDTH..(spr_idx+1)*PGTBL_WIDTH];

            for r in 0..8 {
                let lsb_byte = sprite_bytes[r];
                let msb_byte = sprite_bytes[r + 8];

                for c in 0..8 {
                    let col_idx = (((msb_byte >> (7 - c)) & 1) << 1) | ((lsb_byte >> (7 - c)) & 1);
                    let col = GREYSCALE[col_idx as usize];

                    let pixel_x = x + spr_x*SPRITE_WIDTH + c;
                    let pixel_y = y + spr_y*SPRITE_WIDTH + r;

                    dot(frame, frame_width, frame_height, pixel_x, pixel_y, 1, col);
                }
            }
        }
    }
}

pub fn draw_cpu_state(frame: &mut [u8], frame_width: usize, frame_height: usize,
                    nes: &Nes, x: usize, y: usize, palette: DebugPalette) {
    
    let cpu_state = nes.get_cpu_state();

    let mut text = String::with_capacity(200);
    text.push_str(&format!("A:{:02X}  X:{:02X}  Y:{:02X}\n", cpu_state.acc, cpu_state.x, cpu_state.y));
    text.push_str(&format!("SP:{:02X}  PC:{:04X}\n", cpu_state.sp, cpu_state.pc));
    text.push_str(&format!("Total Clks:{}\nStatus:", cpu_state.total_clocks));

    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, &text, x, y, palette.txt_col, palette.bg_col, 2);
    // Flags
    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, 
        "N", next_x, next_y, 
        if cpu_state.status.negative() { palette.ok_col } else { palette.err_col }, 
        palette.bg_col, 2);
    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, 
        "V", next_x, next_y, 
        if cpu_state.status.overflow() { palette.ok_col } else { palette.err_col }, 
        palette.bg_col, 2);
    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, 
        "U", next_x, next_y, 
        if cpu_state.status.unused() { palette.ok_col } else { palette.err_col }, 
        palette.bg_col, 2);
    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, 
        "B", next_x, next_y, 
        if cpu_state.status.b() { palette.ok_col } else { palette.err_col }, 
        palette.bg_col, 2);
    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, 
        "D", next_x, next_y, 
        if cpu_state.status.decimal() { palette.ok_col } else { palette.err_col }, 
        palette.bg_col, 2);
    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, 
        "I", next_x, next_y, 
        if cpu_state.status.interrupt() { palette.ok_col } else { palette.err_col }, 
        palette.bg_col, 2);
    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, 
        "Z", next_x, next_y, 
        if cpu_state.status.zero() { palette.ok_col } else { palette.err_col }, 
        palette.bg_col, 2);
    let (_, next_y) = draw_string(frame, frame_width, frame_height, 
        "C\n", next_x, next_y, 
        if cpu_state.status.carry() { palette.ok_col } else { palette.err_col }, 
        palette.bg_col, 2);

    let instr_str = format!("{: <34}", nes.get_cpu().current_instr_str());

    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, 
        "Last Instr:", x, next_y, 
        palette.txt_col, palette.bg_col, 1);
    draw_string(frame, frame_width, frame_height, 
        &instr_str, next_x, next_y, 
        palette.txt_col, palette.bg_col, 1);
}

fn draw_zpage(frame: &mut [u8], frame_width: usize, frame_height: usize,
            nes: &mut Nes, x: usize, y: usize, palette: DebugPalette) {
    
    let zpage_str = nes.zpage_str();

    draw_string(frame, frame_width, frame_height, &zpage_str, x, y, palette.txt_col, palette.bg_col, 1);
}

/// Draw the background of the debug view to the frame buffer. This renders the
/// title, outlines, and pagetables (i.e. everything that doesn't change)
/// This function should only be called once.
pub fn draw_debug_bg(frame: &mut [u8], palette: DebugPalette, nes: &Nes) {
    // TITLE DECOR
    horizontal_line(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 5, 255, 4, 2, palette.border_col);
    horizontal_line(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 10, 250, 10, 2, palette.border_col);
    
    draw_string(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, "NEmulator", 262, 2, palette.txt_col, palette.bg_col, 2);

    horizontal_line(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 382, 632, 4, 2, palette.border_col);
    horizontal_line(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 387, 627, 10, 2, palette.border_col);

    // NES SCREEN DECOR
    draw_box(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 
        DEBUG_NES_SCREEN_X - 4, DEBUG_NES_SCREEN_Y - 4, 
        518, 486, 2, palette, None);

    // PAGETABLE VIEWS
    draw_box(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 
        DEBUG_PGTBL1_VIEW_X - 10, DEBUG_PGTBL1_VIEW_Y - 14,
         290, 175, 2, palette, Some("Pagetables"));
    draw_box(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 
        DEBUG_PGTBL1_VIEW_X - 4, DEBUG_PGTBL1_VIEW_Y - 4,
        134, 134, 2, palette, None);
    draw_box(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 
        DEBUG_PGTBL2_VIEW_X - 4, DEBUG_PGTBL2_VIEW_Y - 4,
        134, 134, 2, palette, None);

    let pgtbl1 = nes.get_pgtbl1();
    let pgtbl2 = nes.get_pgtbl2();

    if nes.large_sprites() {
        draw_nes_pagetable_8x16(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, pgtbl1, DEBUG_PGTBL1_VIEW_X, DEBUG_PGTBL1_VIEW_Y);
        draw_nes_pagetable_8x16(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, pgtbl2, DEBUG_PGTBL2_VIEW_X, DEBUG_PGTBL2_VIEW_Y);
    } else {
        draw_nes_pagetable_8x8(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, pgtbl1, DEBUG_PGTBL1_VIEW_X, DEBUG_PGTBL1_VIEW_Y);
        draw_nes_pagetable_8x8(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, pgtbl2, DEBUG_PGTBL2_VIEW_X, DEBUG_PGTBL2_VIEW_Y);
    }

    // CPU INFO DECOR
    draw_box(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 
        DEBUG_CPU_STATE_X - 7, DEBUG_CPU_STATE_Y - 11, 
        331, 100, 2, palette, Some("CPU Info"));

    // ZPAGE DECOR
    draw_box(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 
        DEBUG_ZPAGE_STATE_X - 7, DEBUG_ZPAGE_STATE_Y - 11, 
        390, 188, 2, palette, Some("Zero-Page"))
}

pub fn draw_debug(frame: &mut [u8], palette: DebugPalette, nes: &mut Nes, fps: usize) {
    draw_nes_screen(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, nes.screen_buf_slice(), 
                        DEBUG_NES_SCREEN_X, DEBUG_NES_SCREEN_Y, true);

    draw_cpu_state(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, nes, 
                    DEBUG_CPU_STATE_X, DEBUG_CPU_STATE_Y, palette);

    draw_zpage(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, nes, 
                DEBUG_ZPAGE_STATE_X, DEBUG_ZPAGE_STATE_Y, palette);

    let mirror_text = match nes.current_mirror_type() {
        NametableMirror::Horizontal =>        "Horizontal    ",
        NametableMirror::Vertical =>          "Vertical      ",
        NametableMirror::SingleScreenLower => "1-Screen Lower",
        NametableMirror::SingleScreenUpper => "1-Screen Upper",
        NametableMirror::FourScreen =>        "4-Screen      ",
    };
    let mirror_text = format!("Mirror: {mirror_text}\n");

    let (new_x, new_y) = draw_string(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 
        &mirror_text, DEBUG_PGTBL1_VIEW_X - 4, DEBUG_PGTBL1_VIEW_Y + 128 + 9, 
        palette.txt_col, palette.bg_col, 1);
    
    let spr_size_text = if nes.large_sprites() {
        "8x16 - Large"
    } else {
        "8x8 - Small "
    };
    let spr_size_text = format!("Spr Size: {spr_size_text}");

    draw_string(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 
        &spr_size_text, new_x, new_y, 
        palette.txt_col, palette.bg_col, 1);

    let fps_str = format!("FPS: {fps} ");

    draw_string(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, &fps_str, 
        DEBUG_FPS_COUNTER_X, DEBUG_FPS_COUNTER_Y, palette.txt_col, palette.bg_col, 2);
}



/// Draw the background for the regular game view (i.e. everything in the game view)
/// that won't change. This should only be called once when the frame is created.
pub fn draw_game_view_bg(frame: &mut [u8], palette: DebugPalette) {
    // // TITLE DECOR
    // horizontal_line(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, 4, 144, 4, 2, palette.border_col);
    // horizontal_line(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, 14, 134, 10, 2, palette.border_col);
    
    // draw_string(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, "NEmulator", 150, 2, palette.txt_col, palette.bg_col, true);

    // horizontal_line(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, 270, 316, 4, 2, palette.border_col);
    // horizontal_line(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, 280, 306, 10, 2, palette.border_col);

    // // NES SCREEN DECOR
    // draw_box(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, 30, 30, 259, 243, 2, palette, None);
}

pub fn draw_game_view(frame: &mut [u8], nes: &mut Nes) {
    draw_nes_screen(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, nes.screen_buf_slice(), 0, 0, false);
}


pub fn draw_paused_menu_bg(frame: &mut [u8]) {
    // No background. Will leave this here in case I want to add something later
}

fn draw_menu_options(frame: &mut [u8], menu: &PauseMenu, limit_fps: bool) {
    let w = GAME_FRAME_WIDTH;
    let h = GAME_FRAME_HEIGHT;
    let x = 10;
    let y = 10;
    let chr_col = DEFAULT_DEBUG_PAL.txt_col;
    let bg_col = DEFAULT_DEBUG_PAL.bg_col;
    let s = 1;

    let (next_x, next_y) = draw_string(frame, w, h, "Emulator Paused\n\n ", 
        x, y, chr_col, bg_col, s+1);

    let (next_x, next_y) = draw_string(frame, w, h, "Continue [ESC]\n", 
        next_x, next_y, chr_col, bg_col, s);

    let (next_x, next_y) = draw_string(frame, w, h, "Map Controller Inputs\n", 
        next_x, next_y, chr_col, bg_col, s);
    
    let (next_x, next_y) = draw_string(frame, w, h, "Volume\n", 
        next_x, next_y, chr_col, bg_col, s);

    let (next_x, next_y) = draw_string(frame, w, h, "NoLimit\n", 
        next_x, next_y, chr_col, bg_col, s);

    draw_string(frame, w, h, "Quit\n", 
        next_x, next_y, chr_col, bg_col, s);

    let select_str = match menu.selected {
        PauseMenuItem::Continue =>      "`>`\n \n \n \n ",
        PauseMenuItem::ControllerMap => " \n`>`\n \n \n ",
        PauseMenuItem::Volume =>        " \n \n`>`\n \n ",
        PauseMenuItem::NoLimit =>       " \n \n \n`>`\n ",
        PauseMenuItem::Quit =>          " \n \n \n \n`>`",
    };
    let (next_x, next_y) = draw_string(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, "\n\n", 10, 10, WHITE, DEFAULT_DEBUG_PAL.bg_col, 2);
    draw_string(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, select_str, next_x, next_y, WHITE, DEFAULT_DEBUG_PAL.bg_col, 1);
    
    let (nolimit_str_x, nolimit_str_y) = draw_string(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, "` `", next_x, next_y, DEFAULT_DEBUG_PAL.txt_col, DEFAULT_DEBUG_PAL.bg_col, 2);
    let (nolimit_str_x, nolimit_str_y) = draw_string(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, "\n\n\n`        `", nolimit_str_x, nolimit_str_y, DEFAULT_DEBUG_PAL.txt_col, DEFAULT_DEBUG_PAL.bg_col, 1);
    let (nolimit_str_x, nolimit_str_y) = draw_string(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, "[", nolimit_str_x, nolimit_str_y, WHITE, DEFAULT_DEBUG_PAL.bg_col, 1);
    let (nolimit_str_x, nolimit_str_y) = draw_string(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, "ON", nolimit_str_x, nolimit_str_y, if limit_fps { GREY } else { GREEN }, DEFAULT_DEBUG_PAL.bg_col, 1);
    let (nolimit_str_x, nolimit_str_y) = draw_string(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, "/", nolimit_str_x, nolimit_str_y, WHITE, DEFAULT_DEBUG_PAL.bg_col, 1);
    let (nolimit_str_x, nolimit_str_y) = draw_string(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, "OFF", nolimit_str_x, nolimit_str_y, if limit_fps { RED } else { GREY }, DEFAULT_DEBUG_PAL.bg_col, 1);
    draw_string(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, "]", nolimit_str_x, nolimit_str_y, WHITE, DEFAULT_DEBUG_PAL.bg_col, 1);
}

fn draw_volume_menu(frame: &mut [u8], menu: &PauseMenu) {
    let w = GAME_FRAME_WIDTH;
    let h = GAME_FRAME_HEIGHT;
    let x = 10;
    let y = 10;
    let chr_col = DEFAULT_DEBUG_PAL.txt_col;
    let bg_col = DEFAULT_DEBUG_PAL.bg_col;
    let s = 1;

    let (next_x, next_y) = draw_string(frame, w, h, "Setting Volume\n\n ", 
        x, y, chr_col, bg_col, s+1);

    let volume_text = format!("Volume: {}%", (menu.volume_percent * 100.0) as usize);

    draw_string(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, &volume_text, 
        next_x, next_y, chr_col, bg_col, s);
    
        menu.slider_sprite.draw(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, MENU_VOLUME_SLIDER_X, MENU_VOLUME_SLIDER_Y, menu.volume_percent);
}

fn draw_controller_mapping_menu(frame: &mut [u8], menu: &PauseMenu) {
    let w = GAME_FRAME_WIDTH;
    let h = GAME_FRAME_HEIGHT;
    let x = 10;
    let y = 10;
    let chr_col = DEFAULT_DEBUG_PAL.txt_col;
    let bg_col = DEFAULT_DEBUG_PAL.bg_col;
    let s = 1;

    menu.controller_sprite.draw(frame, w, h, MENU_CONTROLLER_X, MENU_CONTROLLER_Y, menu.controller_state);
    
    if menu.map_controller1 || menu.map_controller2 {
        if menu.map_controller1 {
            draw_string(frame, w, h, "Player 1",
                x, y, chr_col, bg_col, 2);
        } else {
            draw_string(frame, w, h, "Player 2",
                x, y, chr_col, bg_col, 2);
        }

        let button_str = match menu.controller_read.button() {
            ControllerButton::A =>      "A Button     ",
            ControllerButton::B =>      "B Button     ",
            ControllerButton::Up =>     "Up Arrow     ",
            ControllerButton::Down =>   "Down Arrow   ",
            ControllerButton::Left =>   "Left Arrow   ",
            ControllerButton::Right =>  "Right Arrow  ",
            ControllerButton::Select => "Select Button",
            ControllerButton::Start =>  "Start Button ",
        };
        let button_str = format!("Press the {}", button_str);

        draw_string(frame, w, h, &button_str, 
            MENU_CONTROLLER_X + 20, MENU_CONTROLLER_Y - 16, chr_col, bg_col, 1);

    } else {
        let(next_x, next_y) = draw_string(frame, w, h, "Controller Mappings\n\n ",
            x, y, chr_col, bg_col, 2);
        
        if menu.player1_map_selected {
            draw_string(frame, w, h,
                "`> `", next_x, next_y, WHITE, bg_col, 1);
        } else {
            draw_string(frame, w, h,
                "\n`> `", next_x, next_y, WHITE, bg_col, 1);
        }

        draw_string(frame, w, h, "`  `Player 1\n`  `Player 2", 
            next_x, next_y, chr_col, bg_col, 1);
        
    }
}

pub fn draw_menu(frame: &mut [u8], menu: &PauseMenu, limit_fps: bool) {
    frame.fill(0);

    if menu.mapping_controller {
        draw_controller_mapping_menu(frame, menu);
    } else if menu.setting_volume {
        draw_volume_menu(frame, menu);
    } else {
        draw_menu_options(frame, menu, limit_fps);
    }
}