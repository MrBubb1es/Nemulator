pub const DEFAULT_PALETTE: NESPalette = NESPalette{ 
    _colors: [
        NESColor{r: 124, g: 124, b: 124},
        NESColor{r: 0, g: 0, b: 252},
        NESColor{r: 0, g: 0, b: 188},
        NESColor{r: 68, g: 40, b: 188},
        NESColor{r: 148, g: 0, b: 132},
        NESColor{r: 168, g: 0, b: 32},
        NESColor{r: 168, g: 16, b: 0},
        NESColor{r: 136, g: 20, b: 0},
        NESColor{r: 80, g: 48, b: 0},
        NESColor{r: 0, g: 120, b: 0},
        NESColor{r: 0, g: 104, b: 0},
        NESColor{r: 0, g: 88, b: 0},
        NESColor{r: 0, g: 64, b: 88},
        NESColor{r: 0, g: 0, b: 0},
        NESColor{r: 0, g: 0, b: 0},
        NESColor{r: 0, g: 0, b: 0},
        NESColor{r: 188, g: 188, b: 188},
        NESColor{r: 0, g: 120, b: 248},
        NESColor{r: 0, g: 88, b: 248},
        NESColor{r: 104, g: 68, b: 252},
        NESColor{r: 216, g: 0, b: 204},
        NESColor{r: 228, g: 0, b: 88},
        NESColor{r: 248, g: 56, b: 0},
        NESColor{r: 228, g: 92, b: 16},
        NESColor{r: 172, g: 124, b: 0},
        NESColor{r: 0, g: 184, b: 0},
        NESColor{r: 0, g: 168, b: 0},
        NESColor{r: 0, g: 168, b: 68},
        NESColor{r: 0, g: 136, b: 136},
        NESColor{r: 0, g: 0, b: 0},
        NESColor{r: 0, g: 0, b: 0},
        NESColor{r: 0, g: 0, b: 0},
        NESColor{r: 248, g: 248, b: 248},
        NESColor{r: 60, g: 188, b: 252},
        NESColor{r: 104, g: 136, b: 252},
        NESColor{r: 152, g: 120, b: 248},
        NESColor{r: 248, g: 120, b: 248},
        NESColor{r: 248, g: 88, b: 152},
        NESColor{r: 248, g: 120, b: 88},
        NESColor{r: 252, g: 160, b: 68},
        NESColor{r: 248, g: 184, b: 0},
        NESColor{r: 184, g: 248, b: 24},
        NESColor{r: 88, g: 216, b: 84},
        NESColor{r: 88, g: 248, b: 152},
        NESColor{r: 0, g: 232, b: 216},
        NESColor{r: 120, g: 120, b: 120},
        NESColor{r: 0, g: 0, b: 0},
        NESColor{r: 0, g: 0, b: 0},
        NESColor{r: 252, g: 252, b: 252},
        NESColor{r: 164, g: 228, b: 252},
        NESColor{r: 184, g: 184, b: 248},
        NESColor{r: 216, g: 184, b: 248},
        NESColor{r: 248, g: 184, b: 248},
        NESColor{r: 248, g: 164, b: 192},
        NESColor{r: 240, g: 208, b: 176},
        NESColor{r: 252, g: 224, b: 168},
        NESColor{r: 248, g: 216, b: 120},
        NESColor{r: 216, g: 248, b: 120},
        NESColor{r: 184, g: 248, b: 184},
        NESColor{r: 184, g: 248, b: 216},
        NESColor{r: 0, g: 252, b: 252},
        NESColor{r: 248, g: 216, b: 248},
        NESColor{r: 0, g: 0, b: 0},
        NESColor{r: 0, g: 0, b: 0},
    ]
};

#[derive(Clone, Copy)]
pub struct NESColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct NESPalette {
    _colors: [NESColor; 64],
}

impl NESPalette {
    pub fn _read(&self, address: u16) -> NESColor {
        // $3F00 	Universal background color
        // $3F01-$3F03 	Background palette 0
        // $3F04 	Normally unused color 1
        // $3F05-$3F07 	Background palette 1
        // $3F08 	Normally unused color 2
        // $3F09-$3F0B 	Background palette 2
        // $3F0C 	Normally unused color 3
        // $3F0D-$3F0F 	Background palette 3
        // $3F10 	Mirror of universal background color
        // $3F11-$3F13 	Sprite palette 0
        // $3F14 	Mirror of unused color 1
        // $3F15-$3F17 	Sprite palette 1
        // $3F18 	Mirror of unused color 2
        // $3F19-$3F1B 	Sprite palette 2
        // $3F1C 	Mirror of unused color 3
        // $3F1D-$3F1F 	Sprite palette 3

        self._colors[address as usize & 0x1F]
    }
}