mod emulator;

use emulator::Emulator;
use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut emu = Emulator::new("roms/tetris.gb")?;
    emu.run()
}
