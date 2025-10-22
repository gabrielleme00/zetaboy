use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Mbc5 {
    rom_bank: u16,      // ROM bank number (0-511, needs 9 bits)
    ram_bank: u8,       // RAM bank number (0-15)
    ram_enabled: bool,
    rom_banks: usize,
    ram_banks: usize,
}

impl Mbc5 {
    pub fn new(rom_banks: usize, ram_banks: usize) -> Self {
        Self {
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            rom_banks,
            ram_banks,
        }
    }
}

impl super::Mbc for Mbc5 {
    fn read_rom(&self, rom_data: &[u8], address: u16) -> u8 {
        match address {
            // ROM Bank 0 (fixed)
            0x0000..=0x3FFF => rom_data[address as usize],
            // ROM Bank 1-511 (switchable)
            0x4000..=0x7FFF => {
                let bank = self.rom_bank as usize % self.rom_banks;
                let offset = (bank * 0x4000) + (address as usize - 0x4000);
                rom_data[offset]
            }
            _ => 0xFF,
        }
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            // 0x0000-0x1FFF: RAM Enable
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            // 0x2000-0x2FFF: ROM Bank Number (lower 8 bits)
            0x2000..=0x2FFF => {
                self.rom_bank = (self.rom_bank & 0x0100) | (value as u16);
            }
            // 0x3000-0x3FFF: ROM Bank Number (upper bit, bit 8)
            0x3000..=0x3FFF => {
                self.rom_bank = (self.rom_bank & 0x00FF) | (((value & 0x01) as u16) << 8);
            }
            // 0x4000-0x5FFF: RAM Bank Number (0-15)
            0x4000..=0x5FFF => {
                self.ram_bank = value & 0x0F;
            }
            _ => {}
        }
    }

    fn read_ram(&self, ram_data: &[u8], address: u16) -> u8 {
        if !self.ram_enabled || ram_data.is_empty() {
            return 0xFF;
        }

        let bank = self.ram_bank as usize % self.ram_banks.max(1);
        let offset = (bank * 0x2000) + (address as usize - 0xA000);
        
        if offset < ram_data.len() {
            ram_data[offset]
        } else {
            0xFF
        }
    }

    fn write_ram(&mut self, ram_data: &mut [u8], address: u16, value: u8) {
        if !self.ram_enabled || ram_data.is_empty() {
            return;
        }

        let bank = self.ram_bank as usize % self.ram_banks.max(1);
        let offset = (bank * 0x2000) + (address as usize - 0xA000);
        
        if offset < ram_data.len() {
            ram_data[offset] = value;
        }
    }
}
