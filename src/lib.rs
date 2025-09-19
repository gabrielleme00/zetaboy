mod audio;
mod emulator;
mod utils;

use std::error::Error;

use audio::AudioManager;
use emulator::Emulator;

pub const PRINT_SERIAL: bool = false; // Print serial output
pub const PRINT_STATE: bool = false; // Print CPU state after each instruction
pub const PRINT_CART_INFO: bool = false; // Prints cartridge information

pub const CPU_FREQUENCY: u32 = 4194304;

pub fn run(rom_path: &str) -> Result<(), Box<dyn Error>> {
    let mut emu = Emulator::new(rom_path)?;
    let (_audio_manager, audio_sender) = AudioManager::new()?;

    emu.run_with_audio(audio_sender)
}
