mod audio;
mod emulator;
mod gui;
mod utils;

use std::error::Error;

use audio::AudioManager;
use emulator::Emulator;
use gui::EmulatorApp;

pub const PRINT_SERIAL: bool = false; // Print serial output
pub const PRINT_STATE: bool = false; // Print CPU state after each instruction
pub const PRINT_CART_INFO: bool = false; // Prints cartridge information

pub fn run(rom_path: &str) -> Result<(), Box<dyn Error>> {
    let (_audio_manager, audio_sender) = AudioManager::new()?;

    let emulator = Emulator::new(rom_path)?;
    let app = EmulatorApp::new(Some(emulator), Some(audio_sender));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0])
            .with_title("ZetaBoy - Game Boy Emulator")
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };

    eframe::run_native("ZetaBoy", options, Box::new(|_cc| Ok(Box::new(app))))
        .map_err(|e| format!("Failed to run eframe app: {}", e).into())
}
