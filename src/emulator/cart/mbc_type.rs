#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MBCType {
    None,
    MBC1,
    MBC2,
    MBC3,
    MBC5,
    MBC6,
    MBC7,
}

impl MBCType {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => MBCType::None,
            0x01 | 0x02 | 0x03 => MBCType::MBC1,
            0x05 | 0x06 => MBCType::MBC2,
            0x11 | 0x12 | 0x13 => MBCType::MBC3,
            0x19 | 0x1A | 0x1B => MBCType::MBC5,
            0x20 => MBCType::MBC6,
            0x22 => MBCType::MBC7,
            _ => MBCType::None,
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
}
