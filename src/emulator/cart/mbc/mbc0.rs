use super::Mbc;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
pub struct Mbc0;

impl Mbc for Mbc0 {
    fn read_rom(&self, rom_data: &[u8], address: u16) -> u8 {
        let address = address as usize;
        if address < rom_data.len() {
            rom_data[address]
        } else {
            0xFF
        }
    }

    fn write_rom(&mut self, _address: u16, _value: u8) {
        // No-op for ROM-only cartridges
    }

    fn read_ram(&self, _ram_data: &[u8], _address: u16) -> u8 {
        // ROM-only cartridges do not have RAM, always return 0xFF
        0xFF
    }

    fn write_ram(&mut self, _ram_data: &mut [u8], _address: u16, _value: u8) {
        // ROM-only cartridges do not have RAM, do nothing
    }
}
