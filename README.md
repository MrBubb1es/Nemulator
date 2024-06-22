# Nemulator
## An open-source NES emulator written in Rust by two people who don't know Rust
Nemulator is an emulator for the Nintendo Entertainment System written by MrBubb1es
and logocrazymon (who happen to be brothers). This was largely a project done for
our own learning and benefit, and as such we make no guarantees of code quality
or correctness. We did our best to test our code with a variety of games and test
ROMs, but there are probably a few good bugs left in there. We learned a lot and
had fun making it.

Huge credit is due to OneLoneCoder (javidx9) for his [youtube series](https://www.youtube.com/playlist?list=PLrOv9FMX8xJHqMvSGB_9G9nZZ_4IgteYf) documenting his own implementation
of a working NES emulator, as well as the [NesDev Wiki](https://www.nesdev.org/wiki/Nesdev_Wiki), without which this code would have been almost impossible to write correctly.

## Usage
To run the emulator, simply invoke it with a path to the .nes file you'd like to run.
Nemulator supports the NES 2.0 header format, which is backwards-compatible with the
ubiquitous iNES header format.

Emulation can be paused by hitting the `ESC` key, which brings up a menu that allows for volume control and controller re-mapping. Emulation can also be restarted by holding the `r` key.

Player one controls are always accessible through the keyboard (Arrow keys, `z` and `x` for A and B, `RETURN` and `RSHIFT` for Start and Select, respectively). The pause menu is only navigable via the keyboard.

If run with the `--debug` flag, additional controls are enabled. Pressing `v` brings up a debugger view which shows the state of the CPU, pagetables, and first 256 bytes of memory. If the emulation is paused, `c` single-steps the CPU and `f` steps frame-by-frame.

The emulator typically tries to recreate the timing of an NTSC NES. If invoked with the `--nolimit` flag, the emulator will run as fast as possible; this disable sound, but it's a fun challenge to try and beat Mario Bros. 1-1 this way. (The nolimit behavior can also be toggled from the pause menu.)

## Support
Nemulator only supports a few of the most common mappers (iNES numbers 0-4, plus 9). We felt those gave us good enough coverage of popular games for our liking, but it means certain games may not work correctly (or at all) with this emulator.