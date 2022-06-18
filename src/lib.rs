mod emulator;

use emulator::Emulator;
use piston_window::*;
use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
    println!("Starting Game Boy emulator...");

    let mut emu = Emulator::new();
    emu.load_rom("roms/tetris.gb")?;

    while let Some(e) = emu.window.next() {
        if let Some(_) = e.render_args() {
            emu.render(&e);
        }
        if let Some(_) = e.update_args() {
            if !emu.running {
                return Ok(());
            }
            if emu.paused {
                continue;
            }
            emu.update()?;
        }
    }

    Ok(())
}
