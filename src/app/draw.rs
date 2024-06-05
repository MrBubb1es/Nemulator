use crate::system::nes::NES;

pub const DEBUG_FRAME_WIDTH: usize = 960;
pub const DEBUG_FRAME_HEIGHT: usize = 540;

// 256 pixels for NES + 32 pixels padding on either size
pub const GAME_FRAME_WIDTH: usize = 256;
// 240 pixels for NES + 24 pixels padding on either size
pub const GAME_FRAME_HEIGHT: usize = 240;

// Array that the NES is given to render to
pub static mut RENDER_ARR: [u8; 256*240*4] = [0; 256*240*4];

// pub struct DrawState {
//     zpage_str: String,
//     updated_nes_pixels:
// } 

pub mod chars {
    pub const CHAR_WIDTH: usize = 7;
    pub const CHAR_HEIGHT: usize = 8;
    /// Number of pixels left between newlines
    pub const NEWLINE_PADDING: usize = 3;

    pub const A: [u8; 8] = [
        0b0111_0000,
        0b1000_1000,
        0b1000_1000,
        0b1111_1000,
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b0000_0000,
    ];
    pub const B: [u8; 8] = [
        0b1111_0000,
        0b1000_1000,
        0b1000_1000,
        0b1111_0000,
        0b1000_1000,
        0b1000_1000,
        0b1111_0000,
        0b0000_0000,
    ];
    pub const C: [u8; 8] = [
        0b0111_0000,
        0b1000_1000,
        0b1000_0000,
        0b1000_0000,
        0b1000_0000,
        0b1000_1000,
        0b0111_0000,
        0b0000_0000,
    ];
    pub const D: [u8; 8] = [
        0b1111_0000,
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b1111_0000,
        0b0000_0000,
    ];
    pub const E: [u8; 8] = [
        0b1111_1000,
        0b1000_0000,
        0b1000_0000,
        0b1111_0000,
        0b1000_0000,
        0b1000_0000,
        0b1111_1000,
        0b0000_0000,
    ];
    pub const F: [u8; 8] = [
        0b1111_1000,
        0b1000_0000,
        0b1000_0000,
        0b1111_0000,
        0b1000_0000,
        0b1000_0000,
        0b1000_0000,
        0b0000_0000,
    ];
    pub const G: [u8; 8] = [
        0b0111_0000,
        0b1000_1000,
        0b1000_0000,
        0b1000_0000,
        0b1001_1000,
        0b1000_1000,
        0b0111_0000,
        0b0000_0000,
    ];
    pub const H: [u8; 8] = [
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b1111_1000,
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b0000_0000,
    ];
    pub const I: [u8; 8] = [
        0b1111_1000,
        0b0010_0000,
        0b0010_0000,
        0b0010_0000,
        0b0010_0000,
        0b0010_0000,
        0b1111_1000,
        0b0000_0000,
    ];
    pub const J: [u8; 8] = [
        0b0011_1000,
        0b0001_0000,
        0b0001_0000,
        0b0001_0000,
        0b0001_0000,
        0b1001_0000,
        0b0110_0000,
        0b0000_0000,
    ];
    pub const K: [u8; 8] = [
        0b1000_1000,
        0b1001_0000,
        0b1010_0000,
        0b1100_0000,
        0b1010_0000,
        0b1001_0000,
        0b1000_1000,
        0b0000_0000,
    ];
    pub const L: [u8; 8] = [
        0b1000_0000,
        0b1000_0000,
        0b1000_0000,
        0b1000_0000,
        0b1000_0000,
        0b1000_0000,
        0b1111_1000,
        0b0000_0000,
    ];
    pub const M: [u8; 8] = [
        0b1000_1000,
        0b1101_1000,
        0b1010_1000,
        0b1010_1000,
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b0000_0000,
    ];
    pub const N: [u8; 8] = [
        0b1000_1000,
        0b1000_1000,
        0b1100_1000,
        0b1010_1000,
        0b1001_1000,
        0b1000_1000,
        0b1000_1000,
        0b0000_0000,
    ];
    pub const O: [u8; 8] = [
        0b0111_0000,
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b0111_0000,
        0b0000_0000,
    ];
    pub const P: [u8; 8] = [
        0b1111_0000,
        0b1000_1000,
        0b1000_1000,
        0b1111_0000,
        0b1000_0000,
        0b1000_0000,
        0b1000_0000,
        0b0000_0000,
    ];
    pub const Q: [u8; 8] = [
        0b0111_0000,
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b1010_1000,
        0b1001_0000,
        0b0110_1000,
        0b0000_0000,
    ];
    pub const R: [u8; 8] = [
        0b1111_0000,
        0b1000_1000,
        0b1000_1000,
        0b1111_0000,
        0b1010_0000,
        0b1001_0000,
        0b1000_1000,
        0b0000_0000,
    ];
    pub const S: [u8; 8] = [
        0b0111_1000,
        0b1000_0000,
        0b1000_0000,
        0b0111_0000,
        0b0000_1000,
        0b0000_1000,
        0b1111_0000,
        0b0000_0000,
    ];
    pub const T: [u8; 8] = [
        0b1111_1000,
        0b0010_0000,
        0b0010_0000,
        0b0010_0000,
        0b0010_0000,
        0b0010_0000,
        0b0010_0000,
        0b0000_0000,
    ];
    pub const U: [u8; 8] = [
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b0111_0000,
        0b0000_0000,
    ];
    pub const V: [u8; 8] = [
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b0101_0000,
        0b0101_0000,
        0b0010_0000,
        0b0000_0000,
    ];
    pub const W: [u8; 8] = [
        0b1000_1000,
        0b1000_1000,
        0b1000_1000,
        0b1010_1000,
        0b1010_1000,
        0b1010_1000,
        0b0101_0000,
        0b0000_0000,
    ];
    pub const X: [u8; 8] = [
        0b1000_1000,
        0b1000_1000,
        0b0101_0000,
        0b0010_0000,
        0b0101_0000,
        0b1000_1000,
        0b1000_1000,
        0b0000_0000,
    ];
    pub const Y: [u8; 8] = [
        0b1000_1000,
        0b1000_1000,
        0b0101_0000,
        0b0010_0000,
        0b0010_0000,
        0b0010_0000,
        0b0010_0000,
        0b0000_0000,
    ];
    pub const Z: [u8; 8] = [
        0b1111_1000,
        0b0000_1000,
        0b0001_0000,
        0b0010_0000,
        0b0100_0000,
        0b1000_0000,
        0b1111_1000,
        0b0000_0000,
    ];
    pub const SPACE: [u8; 8] = [
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
    ];
    pub const ZERO: [u8; 8] = [
        0b0111_0000,
        0b1000_1000,
        0b1001_1000,
        0b1010_1000,
        0b1100_1000,
        0b1000_1000,
        0b0111_0000,
        0b0000_0000,
    ];
    pub const ONE: [u8; 8] = [
        0b0010_0000,
        0b1110_0000,
        0b0010_0000,
        0b0010_0000,
        0b0010_0000,
        0b0010_0000,
        0b1111_1000,
        0b0000_0000,
    ];
    pub const TWO: [u8; 8] = [
        0b0111_0000,
        0b1000_1000,
        0b0000_1000,
        0b0001_0000,
        0b0010_0000,
        0b0100_0000,
        0b1111_1000,
        0b0000_0000,
    ];
    pub const THREE: [u8; 8] = [
        0b1111_1000,
        0b0001_0000,
        0b0010_0000,
        0b0001_0000,
        0b0000_1000,
        0b1000_1000,
        0b0111_0000,
        0b0000_0000,
    ];
    pub const FOUR: [u8; 8] = [
        0b0001_0000,
        0b0011_0000,
        0b0101_0000,
        0b1001_0000,
        0b1111_1000,
        0b0001_0000,
        0b0001_0000,
        0b0000_0000,
    ];
    pub const FIVE: [u8; 8] = [
        0b1111_1000,
        0b1000_0000,
        0b1111_0000,
        0b0000_1000,
        0b0000_1000,
        0b1000_1000,
        0b0111_0000,
        0b0000_0000,
    ];
    pub const SIX: [u8; 8] = [
        0b0011_0000,
        0b0100_0000,
        0b1000_0000,
        0b1111_0000,
        0b1000_1000,
        0b1000_1000,
        0b0111_0000,
        0b0000_0000,
    ];
    pub const SEVEN: [u8; 8] = [
        0b1111_1000,
        0b0000_1000,
        0b0001_0000,
        0b0010_0000,
        0b0100_0000,
        0b0100_0000,
        0b0100_0000,
        0b0000_0000,
    ];
    pub const EIGHT: [u8; 8] = [
        0b0111_0000,
        0b1000_1000,
        0b1000_1000,
        0b0111_0000,
        0b1000_1000,
        0b1000_1000,
        0b0111_0000,
        0b0000_0000,
    ];
    pub const NINE: [u8; 8] = [
        0b0111_0000,
        0b1000_1000,
        0b1000_1000,
        0b0111_1000,
        0b0000_1000,
        0b0001_0000,
        0b0110_0000,
        0b0000_0000,
    ];
    pub const DOLLAR: [u8; 8] = [
        0b0010_0000,
        0b0111_1000,
        0b1010_0000,
        0b0111_0000,
        0b0010_1000,
        0b1111_0000,
        0b0010_0000,
        0b0000_0000,
    ];
    pub const COLON: [u8; 8] = [
        0b0000_0000,
        0b0100_0000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0100_0000,
        0b0000_0000,
        0b0000_0000,
    ];
    pub const LBRACE: [u8; 8] = [
        0b0001_1000,
        0b0010_0000,
        0b0010_0000,
        0b0100_0000,
        0b0010_0000,
        0b0010_0000,
        0b0001_1000,
        0b0000_0000,
    ];
    pub const RBRACE: [u8; 8] = [
        0b1100_0000,
        0b0010_0000,
        0b0010_0000,
        0b0001_0000,
        0b0010_0000,
        0b0010_0000,
        0b1100_0000,
        0b0000_0000,
    ];
    pub const LANGLE: [u8; 8] = [
        0b0000_0100,
        0b0000_1000,
        0b0001_0000,
        0b0010_0000,
        0b0001_0000,
        0b0000_1000,
        0b0000_0100,
        0b0000_0000,
    ];
    pub const RANGLE: [u8; 8] = [
        0b0100_0000,
        0b0010_0000,
        0b0001_0000,
        0b0010_1000,
        0b0001_0000,
        0b0010_0000,
        0b0100_0000,
        0b0000_0000,
    ];
    pub const LSQR: [u8; 8] = [
        0b0011_1000,
        0b0010_0000,
        0b0010_0000,
        0b0010_0000,
        0b0010_0000,
        0b0010_0000,
        0b0011_1000,
        0b0000_0000,
    ];
    pub const RSQR: [u8; 8] = [
        0b1110_0000,
        0b0010_0000,
        0b0010_0000,
        0b0010_0000,
        0b0010_0000,
        0b0010_0000,
        0b1110_0000,
        0b0000_0000,
    ];
    pub const HASH: [u8; 8] = [
        0b0101_0000,
        0b0101_0000,
        0b1111_1000,
        0b0101_0000,
        0b1111_1000,
        0b0101_0000,
        0b0101_0000,
        0b0000_0000,
    ];
    pub const DASH: [u8; 8] = [
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b1111_1000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
    ];
    pub const EQUAL: [u8; 8] = [
        0b0000_0000,
        0b0000_0000,
        0b1111_1000,
        0b0000_0000,
        0b1111_1000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
    ];
    pub const COMMA: [u8; 8] = [
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0110_0000,
        0b0110_0000,
        0b0010_0000,
        0b0100_0000,
    ];

    // # # # # # # # #
    // # . . # # . . #
    // # . # . . # . #
    // # . . . . # . #
    // # . . # # . . #
    // # . . . . . . #
    // # . . # . . . #
    // # # # # # # # #
    pub const UNKNOWN: [u8; 8] = [
        0b1111_1111,
        0b1001_1001,
        0b1010_0101,
        0b1000_0101,
        0b1001_1001,
        0b1000_0001,
        0b1001_0001,
        0b1111_1111,
    ];

    pub fn get_letter_from_char(letter_char: char) -> [u8; 8] {
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
            '>' => RANGLE,
            '[' => LSQR,
            ']' => RSQR,
            '#' => HASH,
            '-' => DASH,
            '=' => EQUAL,
            ',' => COMMA,
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

// pub const DEFAULT_DEBUG_PAL: DebugPalette = DebugPalette {
//     // bg_col: Color{r: 50, g: 69, b: 62},
//     bg_col: Color{r: 0, g: 0, b: 0},
//     txt_col: Color{r: 0xF0, g: 0xF0, b: 0xF0},
//     border_col: Color{r: 0xA0, g: 0xA0, b: 0xA0},
//     ok_col: Color{r: 0x8C, g: 0xB3, b: 0x69},
//     err_col: Color{r: 0xBC, g: 0x4B, b: 0x51},
// };

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
                bg_col: Color, large_text: bool) {
    let bitmap = chars::get_letter_from_char(character);
    let scale_factor = if large_text { 2 } else { 1 };
    
    for (y, row) in bitmap.iter().enumerate() {
        let global_y = y_pos + y*scale_factor;

        if global_y >= frame_height {
            break;
        }

        for x in 0..8 {
            let global_x = x_pos + x*scale_factor;

            if global_x >= frame_width {
                break;
            }

            // x4 because the frame buffer stores 4 vals per pixel (R,G,B,A)
            let pixel_pos = (global_y * frame_width + global_x) * 4;
            let col = if (row >> (7 - x)) & 1 == 1 { chr_col } else { bg_col };

            dot(frame, frame_width, frame_height, global_x, global_y, scale_factor, col);
        }
    }
}

/// Draw a string of characters to the frame buffer with the top left most 
/// pixels at (x_pos, y_pos). Also takes the frame width & height in pixels to 
/// ensure no pixels are drawn outside of the frame. Returns an (x, y)
/// coordinate for where the nect character would go.
pub fn draw_string(frame: &mut [u8], frame_width: usize, frame_height: usize, 
                text: &str, x_pos: usize, y_pos: usize, chr_col: Color, 
                bg_col: Color, large_text: bool) -> (usize, usize) {

    let mut curr_x = x_pos;
    let mut curr_y = y_pos;

    let mut horizontal_step = chars::CHAR_WIDTH;
    let mut vertical_step = chars::CHAR_HEIGHT + chars::NEWLINE_PADDING;
    
    if large_text {
        horizontal_step = 2 * horizontal_step - 1;
        vertical_step += chars::CHAR_HEIGHT;
    }

    for character in text.chars().into_iter() {
        if character == '\n' {
            curr_x = x_pos;
            curr_y += vertical_step;
            continue;
        }

        draw_char(frame, frame_width, frame_height, character, curr_x, curr_y, chr_col, bg_col, large_text); 
        
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

        draw_string(frame, frame_width, frame_height, text, title_x, title_y, palette.border_col, palette.bg_col, true);
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

/// Draw the background of the debug view to the frame buffer. This renders the
/// title, outlines, and pagetables (i.e. everything that doesn't change)
/// This function should only be called once.
pub fn draw_debug_bg(frame: &mut [u8], palette: DebugPalette, nes: &NES) {
    // TITLE DECOR
    horizontal_line(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 5, 255, 4, 2, palette.border_col);
    horizontal_line(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 10, 250, 10, 2, palette.border_col);
    
    draw_string(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, "NEmulator", 262, 2, palette.txt_col, palette.bg_col, true);

    horizontal_line(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 382, 632, 4, 2, palette.border_col);
    horizontal_line(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 387, 627, 10, 2, palette.border_col);

    // NES SCREEN DECOR
    draw_box(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 5, 34, 518, 486, 2, palette, None);

    // PAGETABLE VIEWS
    draw_box(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 536, 378, 290, 150, 2, palette, Some("Pagetables"));
    draw_box(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 542, 388, 134, 134, 2, palette, None);
    draw_box(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 686, 388, 134, 134, 2, palette, None);

    let pgtbl1 = nes.get_pgtbl1();
    let pgtbl2 = nes.get_pgtbl2();

    draw_nes_pagetable(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, pgtbl1, 546, 392);
    draw_nes_pagetable(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, pgtbl2, 690, 392);
    
    // CPU INFO DECOR
    draw_box(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 536, 34, 331, 110, 2, palette, Some("CPU Info"));

    // ZPAGE DECOR
    draw_box(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, 536, 160, 390, 188, 2, palette, Some("Zero-Page"))
}

pub fn draw_debug(frame: &mut [u8], palette: DebugPalette, nes: &mut NES) {
    draw_nes_screen(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, nes.get_ppu().frame_buf_slice(), 9, 38, true);
    draw_cpu_state(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, nes, 543, 45, palette);
    draw_zpage(frame, DEBUG_FRAME_WIDTH, DEBUG_FRAME_HEIGHT, nes, 543, 171, palette);
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

pub fn draw_nes_pagetable(frame: &mut [u8], frame_width: usize, frame_height: usize, 
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

pub fn draw_cpu_state(frame: &mut [u8], frame_width: usize, frame_height: usize,
                    nes: &NES, x: usize, y: usize, palette: DebugPalette) {
    
    let cpu_state = nes.get_cpu_state();

    let mut text = String::with_capacity(200);
    text.push_str(&format!("A:{:02X}  X:{:02X}  Y:{:02X}\n", cpu_state.acc, cpu_state.x, cpu_state.y));
    text.push_str(&format!("SP:{:02X}  PC:{:04X}\n", cpu_state.sp, cpu_state.pc));
    text.push_str(&format!("Total Clks:{}\nStatus:", cpu_state.total_clocks));

    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, &text, x, y, palette.txt_col, palette.bg_col, true);
    // Flags
    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, 
        "N", next_x, next_y, 
        if cpu_state.status.negative() { palette.ok_col } else { palette.err_col }, 
        palette.bg_col, true);
    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, 
        "V", next_x, next_y, 
        if cpu_state.status.overflow() { palette.ok_col } else { palette.err_col }, 
        palette.bg_col, true);
    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, 
        "U", next_x, next_y, 
        if cpu_state.status.unused() { palette.ok_col } else { palette.err_col }, 
        palette.bg_col, true);
    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, 
        "B", next_x, next_y, 
        if cpu_state.status.b() { palette.ok_col } else { palette.err_col }, 
        palette.bg_col, true);
    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, 
        "D", next_x, next_y, 
        if cpu_state.status.decimal() { palette.ok_col } else { palette.err_col }, 
        palette.bg_col, true);
    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, 
        "I", next_x, next_y, 
        if cpu_state.status.interrupt() { palette.ok_col } else { palette.err_col }, 
        palette.bg_col, true);
    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, 
        "Z", next_x, next_y, 
        if cpu_state.status.zero() { palette.ok_col } else { palette.err_col }, 
        palette.bg_col, true);
    let (_, next_y) = draw_string(frame, frame_width, frame_height, 
        "C\n", next_x, next_y, 
        if cpu_state.status.carry() { palette.ok_col } else { palette.err_col }, 
        palette.bg_col, true);

    let instr_str = format!("{: <34}", nes.get_cpu().current_instr_str());

    let (next_x, next_y) = draw_string(frame, frame_width, frame_height, 
        "Last Instr:", x, next_y, 
        palette.txt_col, palette.bg_col, false);
    draw_string(frame, frame_width, frame_height, 
        &instr_str, next_x, next_y, 
        palette.txt_col, palette.bg_col, false);
}

fn draw_zpage(frame: &mut [u8], frame_width: usize, frame_height: usize,
            nes: &mut NES, x: usize, y: usize, palette: DebugPalette) {
    
    let zpage_str = nes.zpage_str();

    draw_string(frame, frame_width, frame_height, &zpage_str, x, y, palette.txt_col, palette.bg_col, false);
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

pub fn draw_game_view(frame: &mut [u8], nes: &mut NES) {
    draw_nes_screen(frame, GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT, nes.get_ppu().frame_buf_slice(), 0, 0, false);
}

struct DrawDiff {
    x: usize,
    y: usize,
    col: Color,
    from_scroll_x: bool,
    from_scroll_y: bool,
}

pub struct NesRenderer {
    nes_screen: [u8; 256*240*4],
    scroll_x: isize,
    scroll_y: isize,

    // Vector to keep track of the differences since last frame
    diffs: Vec<DrawDiff>,
}