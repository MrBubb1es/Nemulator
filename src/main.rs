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
use std::env;

use nes_emulator;

#[tokio::main]
pub async fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("usage: {} <filename>", args[0]);
        return Ok(());
    }

    nes_emulator::run(&args[1]).await;

    Ok(())
}

