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

use nes_emulator::{self, RuntimeConfig};

pub fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("usage: {} <filename>", args[0]);
        return Ok(());
    }

    let mut config = RuntimeConfig::default();

    config.cart_path = args[1].clone();
    config.limit_fps = !args.contains(&String::from("--nolimit")) && !args.contains(&String::from("-nl"));
    config.can_debug = args.contains(&String::from("--debug")) || args.contains(&String::from("-d"));

    nes_emulator::run(config);

    Ok(())
}

