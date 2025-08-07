mod emulator;

use emulator::Emulator;
use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut emu = Emulator::new("roms/03-op sp,hl.gb")?;
    emu.run()
}
