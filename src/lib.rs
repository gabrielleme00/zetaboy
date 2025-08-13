mod emulator;

use emulator::Emulator;
use std::error::Error;

pub const PRINT_SERIAL: bool = false; // Print serial output
pub const PRINT_STATE: bool = false; // Print CPU state after each instruction

pub fn run(rom_path: &str) -> Result<(), Box<dyn Error>> {
    let mut emu = Emulator::new(rom_path)?;
    emu.run()
}
