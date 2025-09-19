mod audio;
mod emulator;
mod utils;
mod app;

use winit::event_loop::EventLoop;

use std::error::Error;

use audio::AudioManager;
use emulator::Emulator;
use app::App;

pub const PRINT_SERIAL: bool = false; // Print serial output
pub const PRINT_STATE: bool = false; // Print CPU state after each instruction
pub const PRINT_CART_INFO: bool = false; // Prints cartridge information

pub fn run(rom_path: &str) -> Result<(), Box<dyn Error>> {
    let (_audio_manager, audio_sender) = AudioManager::new()?;

    let mut app = App::default();
    app.emulator = Some(Emulator::new(rom_path)?);
    app.audio_sender = Some(audio_sender);

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut app)?;

    Ok(())
}
