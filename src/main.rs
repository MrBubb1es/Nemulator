/* NES EMULATOR
*
* Created by MrBubbles and Logocrazymon
*
* This application is designed to be a basic implementation of a NES emulator.
* 
* Planned Features:
*   - Full ability to play (legally obtained!!!) iNES and NES 2.0 roms
*   - Nerd view (mostly for debugging but it could just be neat to see what the CPU is up to)
*   - Controller support
*   - Local (Couch) Co-op
*
* Possible Future Features:
*   - Local online Co-op
*   - Online multiplayer (with servers and junk)
*/

pub mod cartridge;

fn main() {
    const TEST_ARR: [u8; 4] = [0, 1, 2, 3];

    let other_arr: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

    let eq = TEST_ARR == other_arr[0..4];

    println!("{eq}");

    println!("Hello, world!");
}
