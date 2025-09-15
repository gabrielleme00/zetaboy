use super::Mbc;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
pub struct Mbc2 {
    rom_bank_select: u8,
    rom_bank_count: usize,
    ram_enable: bool,
}

impl Mbc2 {
    pub fn new(rom_bank_count: usize) -> Self {
        Mbc2 {
            rom_bank_select: 1,
            rom_bank_count,
            ram_enable: false,
        }
    }
}

impl Mbc for Mbc2 {
    fn read_rom(&self, rom_data: &[u8], address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => rom_data[address as usize],
            0x4000..=0x7FFF => {
                let bank = (self.rom_bank_select as usize) % self.rom_bank_count;
                let rom_address = bank * 0x4000 + (address as usize - 0x4000);
                rom_data[rom_address]
            }
            _ => 0xFF,
        }
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x3FFF => {
                if address & 0x0100 == 0 {
                    self.ram_enable = value & 0xF == 0xA;
                } else {
                    self.rom_bank_select = (value & 0x0F).max(1);
                }
            }
            _ => {}
        }
    }

    fn read_ram(&self, ram_data: &[u8], address: u16) -> u8 {
        if self.ram_enable {
            let ram_address = (address - 0xA000) as usize & 0x1FF;
            ram_data[ram_address]
        } else {
            0xFF
        }
    }

    fn write_ram(&mut self, ram_data: &mut [u8], address: u16, value: u8) {
        if self.ram_enable {
            let ram_address = (address - 0xA000) as usize & 0x1FF;
            ram_data[ram_address] = value & 0x0F;
        }
    }
}
