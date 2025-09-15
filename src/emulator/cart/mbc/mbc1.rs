use super::Mbc;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
pub struct Mbc1 {
    rom_bank: u8,
    ram_bank: u8,
    ram_enabled: bool,
    banking_mode: u8,
    rom_banks_number: usize,
}

impl Mbc1 {
    pub fn new(rom_banks_number: usize) -> Self {
        Mbc1 {
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            banking_mode: 0,
            rom_banks_number,
        }
    }
}

impl Mbc for Mbc1 {
    fn read_rom(&self, rom_data: &[u8], address: u16) -> u8 {
        let address = address as usize;
        match address {
            0x0000..=0x3FFF => {
                let bank = if self.banking_mode == 0 {
                    0
                } else {
                    self.rom_bank as usize & 0x60
                };
                let real_address = bank * 0x4000 + address;
                if real_address < rom_data.len() {
                    rom_data[real_address]
                } else {
                    0xFF
                }
            }
            0x4000..=0x7FFF => {
                let max_banks = rom_data.len() / 0x4000;
                let bank = self.rom_bank as usize % max_banks.max(1);
                let real_address = bank * 0x4000 + (address - 0x4000);
                if real_address < rom_data.len() {
                    rom_data[real_address]
                } else {
                    0xFF
                }
            }
            _ => 0xFF,
        }
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_enabled = (value & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                let bitmask = match self.rom_banks_number {
                    128 => 0b00011111,
                    064 => 0b00011111,
                    032 => 0b00011111,
                    016 => 0b00001111,
                    008 => 0b00000111,
                    004 => 0b00000011,
                    002 => 0b00000001,
                    _ => panic!("Unsupported number of ROM banks: {}", self.rom_banks_number),
                };
                let value = value & 0x1F;
                self.rom_bank =
                    (self.rom_bank & 0x60) | if value == 0 { 1 } else { value & bitmask };
            }
            0x4000..=0x5FFF => {
                let value = value & 0x03;
                if self.banking_mode == 0 {
                    // ROM banking mode
                    self.rom_bank = (self.rom_bank & 0x1F) | (value << 5);
                } else {
                    // RAM banking mode
                    self.ram_bank = value;
                }
            }
            0x6000..=0x7FFF => {
                // Banking mode select
                self.banking_mode = value & 0x01;
                if self.banking_mode == 0 {
                    self.ram_bank = 0; // In ROM mode, RAM bank is always 0
                }
            }
            _ => {}
        }
    }

    fn read_ram(&self, ram_data: &[u8], address: u16) -> u8 {
        if !self.ram_enabled || ram_data.is_empty() {
            return 0xFF;
        }

        let address = (address - 0xA000) as usize;
        let bank = if self.banking_mode == 1 {
            self.ram_bank as usize
        } else {
            0
        };

        let bank_offset = 0x2000 * bank;
        let real_address = bank_offset + address;

        if real_address < ram_data.len() {
            ram_data[real_address]
        } else {
            0xFF
        }
    }

    fn write_ram(&mut self, ram_data: &mut [u8], address: u16, value: u8) {
        if !self.ram_enabled || ram_data.is_empty() {
            return;
        }

        let address = (address - 0xA000) as usize;
        let bank = if self.banking_mode == 1 {
            self.ram_bank as usize
        } else {
            0
        };

        let bank_offset = 0x2000 * bank;
        let real_address = bank_offset + address;

        if real_address < ram_data.len() {
            ram_data[real_address] = value;
        }
    }
}
