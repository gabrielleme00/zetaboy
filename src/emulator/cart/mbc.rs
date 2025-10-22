mod mbc0;
mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;

use mbc0::Mbc0;
use mbc1::Mbc1;
use mbc2::Mbc2;
use mbc3::Mbc3;
use mbc5::Mbc5;
use serde::{Deserialize, Serialize};

trait Mbc {
    fn read_rom(&self, rom_data: &[u8], address: u16) -> u8;
    fn write_rom(&mut self, address: u16, value: u8);
    fn read_ram(&self, ram_data: &[u8], address: u16) -> u8;
    fn write_ram(&mut self, ram_data: &mut [u8], address: u16, value: u8);
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub enum MbcType {
    Mbc0(Mbc0),
    Mbc1(Mbc1),
    Mbc2(Mbc2),
    Mbc3(Mbc3),
    Mbc5(Mbc5),
}

impl MbcType {
    pub fn from_byte(byte: u8, rom_banks: usize, ram_banks: usize) -> Self {
        match byte {
            0x00 => MbcType::Mbc0(Mbc0 {}),
            0x01 | 0x02 | 0x03 => MbcType::Mbc1(Mbc1::new(rom_banks)),
            0x05 | 0x06 => MbcType::Mbc2(Mbc2::new(rom_banks)),
            0x0F | 0x10 | 0x11 | 0x12 | 0x13 => {
                MbcType::Mbc3(Mbc3::new(rom_banks, ram_banks, Self::has_timer(byte)))
            }
            0x19 | 0x1A | 0x1B | 0x1C | 0x1D | 0x1E => MbcType::Mbc5(Mbc5::new(rom_banks, ram_banks)),
            _ => panic!("Unsupported MBC type: {:#X} - {}", byte, Self::name(byte)),
        }
    }

    pub fn read_rom(&self, rom_data: &[u8], address: u16) -> u8 {
        match self {
            MbcType::Mbc0(mbc) => mbc.read_rom(rom_data, address),
            MbcType::Mbc1(mbc) => mbc.read_rom(rom_data, address),
            MbcType::Mbc2(mbc) => mbc.read_rom(rom_data, address),
            MbcType::Mbc3(mbc) => mbc.read_rom(rom_data, address),
            MbcType::Mbc5(mbc) => mbc.read_rom(rom_data, address),
        }
    }

    pub fn write_rom(&mut self, address: u16, value: u8) {
        match self {
            MbcType::Mbc0(mbc) => mbc.write_rom(address, value),
            MbcType::Mbc1(mbc) => mbc.write_rom(address, value),
            MbcType::Mbc2(mbc) => mbc.write_rom(address, value),
            MbcType::Mbc3(mbc) => mbc.write_rom(address, value),
            MbcType::Mbc5(mbc) => mbc.write_rom(address, value),
        }
    }

    pub fn read_ram(&self, ram_data: &[u8], address: u16) -> u8 {
        match self {
            MbcType::Mbc0(mbc) => mbc.read_ram(ram_data, address),
            MbcType::Mbc1(mbc) => mbc.read_ram(ram_data, address),
            MbcType::Mbc2(mbc) => mbc.read_ram(ram_data, address),
            MbcType::Mbc3(mbc) => mbc.read_ram(ram_data, address),
            MbcType::Mbc5(mbc) => mbc.read_ram(ram_data, address),
        }
    }

    pub fn write_ram(&mut self, ram_data: &mut [u8], address: u16, value: u8) {
        match self {
            MbcType::Mbc0(mbc) => mbc.write_ram(ram_data, address, value),
            MbcType::Mbc1(mbc) => mbc.write_ram(ram_data, address, value),
            MbcType::Mbc2(mbc) => mbc.write_ram(ram_data, address, value),
            MbcType::Mbc3(mbc) => mbc.write_ram(ram_data, address, value),
            MbcType::Mbc5(mbc) => mbc.write_ram(ram_data, address, value),
        }
    }

    pub fn name(code: u8) -> &'static str {
        match code {
            0x00 => "ROM ONLY",
            0x01 => "MBC1",
            0x02 => "MBC1+RAM",
            0x03 => "MBC1+RAM+BATTERY",
            0x04 => "0x04 ???",
            0x05 => "MBC2",
            0x06 => "MBC2+BATTERY",
            0x07 => "0x07 ???",
            0x08 => "ROM+RAM 1",
            0x09 => "ROM+RAM+BATTERY 1",
            0x0A => "0x0A ???",
            0x0B => "MMM01",
            0x0C => "MMM01+RAM",
            0x0D => "MMM01+RAM+BATTERY",
            0x0E => "0x0E ???",
            0x0F => "MBC3+TIMER+BATTERY",
            0x10 => "MBC3+TIMER+RAM+BATTERY 2",
            0x11 => "MBC3",
            0x12 => "MBC3+RAM 2",
            0x13 => "MBC3+RAM+BATTERY 2",
            0x14 => "0x14 ???",
            0x15 => "0x15 ???",
            0x16 => "0x16 ???",
            0x17 => "0x17 ???",
            0x18 => "0x18 ???",
            0x19 => "MBC5",
            0x1A => "MBC5+RAM",
            0x1B => "MBC5+RAM+BATTERY",
            0x1C => "MBC5+RUMBLE",
            0x1D => "MBC5+RUMBLE+RAM",
            0x1E => "MBC5+RUMBLE+RAM+BATTERY",
            0x1F => "0x1F ???",
            0x20 => "MBC6",
            0x21 => "0x21 ???",
            0x22 => "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
            _ => "UNKNOWN",
        }
    }

    pub fn has_battery(code: u8) -> bool {
        matches!(
            code,
            0x03 | 0x06 | 0x09 | 0x0D | 0x0F | 0x10 | 0x13 | 0x1B | 0x1E | 0x22
        )
    }

    pub fn has_timer(code: u8) -> bool {
        matches!(code, 0x0F | 0x10)
    }
}
