mod emulator;

use emulator::Emulator;
use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut emu = Emulator::new("roms/11-op a,(hl).gb")?;
    emu.run()
}
