use super::Mbc;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const INVALID_MAX_SECONDS: u8 = 63;
const INVALID_MAX_MINUTES: u8 = 63;
const INVALID_MAX_HOURS: u8 = 31;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
struct Rtc {
    millis: u16,
    seconds: u8,
    minutes: u8,
    hours: u8,
    days: u16,
    halted: bool,
    base_timestamp: u64,
    day_carry: bool,
}

impl Rtc {
    fn new() -> Self {
        Rtc {
            millis: 0,
            seconds: 0,
            minutes: 0,
            hours: 0,
            days: 0,
            halted: false,
            base_timestamp: 0,
            day_carry: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Mbc3 {
    rom_bank: u8,
    ram_rtc_selection: u8,
    ram_rtc_enabled: bool,
    banking_mode: u8,
    rom_banks_number: usize,
    ram_banks_number: usize,
    rtc_enabled: bool,
    rtc: Rtc,
    rtc_latch: u8,
    rtc_path: PathBuf,
}

impl Mbc3 {
    pub fn new(
        rom_banks_number: usize,
        ram_banks_number: usize,
        rtc_enabled: bool,
        rom_path: PathBuf,
    ) -> Self {
        let mut rtc_path = rom_path;
        rtc_path.set_extension("rtc");
        let loaded_rtc = load_rtc_state(rtc_path.as_path());
        Mbc3 {
            rom_bank: 1,
            ram_rtc_selection: 0,
            ram_rtc_enabled: false,
            banking_mode: 0,
            rom_banks_number,
            ram_banks_number,
            rtc_enabled,
            rtc: loaded_rtc.unwrap_or_else(Rtc::new),
            rtc_latch: 0xFF,
            rtc_path,
        }
    }

    fn update_rtc_registers(&mut self, elapsed_ms: u64) {
        let mut new_millis = self.rtc.millis as u64 + elapsed_ms;
        let mut new_seconds = self.rtc.seconds as u64;
        let mut new_minutes = self.rtc.minutes as u64;
        let mut new_hours = self.rtc.hours as u64;
        let mut new_days = self.rtc.days as u64;

        let mut invalid_rollover = false;

        new_seconds += new_millis / 1000;
        new_millis %= 1000;

        if self.rtc.seconds < 60 {
            new_minutes += new_seconds / 60;
            new_seconds %= 60;
        } else if new_seconds > INVALID_MAX_SECONDS as u64 {
            invalid_rollover = true;
            new_seconds = 0;
        }

        if self.rtc.minutes < 60 && !invalid_rollover {
            new_hours += new_minutes / 60;
            new_minutes %= 60;
        } else if new_minutes > INVALID_MAX_MINUTES as u64 {
            invalid_rollover = true;
            new_minutes = 0;
        }

        if self.rtc.hours < 24 && !invalid_rollover {
            new_days += new_hours / 24;
            new_hours %= 24;
        } else if new_hours > INVALID_MAX_HOURS as u64 {
            invalid_rollover = true;
            new_hours = 0;
        }

        if new_days >= 512 && !invalid_rollover {
            new_days %= 512;
            self.rtc.day_carry = true;
        }

        self.rtc.millis = new_millis as u16;
        self.rtc.seconds = new_seconds as u8;
        self.rtc.minutes = new_minutes as u8;
        self.rtc.hours = new_hours as u8;
        self.rtc.days = new_days as u16;
    }

    fn save_rtc_state(&self) {
        save_rtc_state(&self.rtc_path, &self.rtc);
    }
}

impl Mbc for Mbc3 {
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
            0x0000..=0x1FFF => self.ram_rtc_enabled = (value & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                self.rom_bank = if value == 0 { 1 } else { value & 0x7F };
            }
            0x4000..=0x5FFF => {
                if value <= 0x03 || (value >= 0x08 && value <= 0x0C) {
                    self.ram_rtc_selection = value;
                }
            }
            0x6000..=0x7FFF => {
                if self.rtc_enabled {
                    if self.rtc_latch == 0 && value == 1 {
                        let current_time_ms = system_time_to_u64_millis(SystemTime::now());

                        if !self.rtc.halted {
                            let elapsed_ms = current_time_ms - self.rtc.base_timestamp;

                            if elapsed_ms > 0 {
                                self.update_rtc_registers(elapsed_ms);
                                self.rtc.base_timestamp = current_time_ms;
                                self.save_rtc_state();
                            }
                        }
                    }
                    self.rtc_latch = value;
                }
            }
            _ => {}
        }
    }

    fn read_ram(&self, ram_data: &[u8], address: u16) -> u8 {
        if self.ram_rtc_enabled {
            match self.ram_rtc_selection {
                0x00..=0x03 => {
                    let address = (address - 0xA000) as usize;
                    let bank = if self.banking_mode == 1 {
                        self.ram_rtc_selection as usize
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
                0x08 => self.rtc.seconds,
                0x09 => self.rtc.minutes,
                0x0A => self.rtc.hours,
                0x0B => (self.rtc.days & 0xFF) as u8,
                0x0C => {
                    let mut value = ((self.rtc.days >> 8) & 0x01) as u8;
                    if self.rtc.halted {
                        value |= 0x40;
                    }
                    if self.rtc.day_carry {
                        value |= 0x80;
                    }
                    value
                }
                _ => 0xFF,
            }
        } else {
            0xFF
        }
    }

    fn write_ram(&mut self, ram_data: &mut [u8], address: u16, value: u8) {
        if self.ram_rtc_enabled {
            match self.ram_rtc_selection {
                0x00..=0x03 => {
                    let address = (address - 0xA000) as usize;
                    let bank = if self.banking_mode == 1 {
                        self.ram_rtc_selection as usize
                    } else {
                        0
                    };

                    let bank_offset = 0x2000 * bank;
                    let real_address = bank_offset + address;

                    if real_address < ram_data.len() {
                        ram_data[real_address] = value;
                    }
                }
                0x08 => self.rtc.seconds = value,
                0x09 => self.rtc.minutes = value,
                0x0A => self.rtc.hours = value,
                0x0B => self.rtc.days = (self.rtc.days & 0x100) | value as u16,
                0x0C => {
                    let current_time_ms = system_time_to_u64_millis(SystemTime::now());

                    if !self.rtc.halted {
                        let elapsed_ms = current_time_ms - self.rtc.base_timestamp;
                        if elapsed_ms > 0 {
                            self.update_rtc_registers(elapsed_ms);
                        }
                    }

                    self.rtc.base_timestamp = current_time_ms;
                    self.save_rtc_state();
                }
                _ => {}
            }
        }
    }
}

fn system_time_to_u64_millis(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn load_rtc_state(path: &Path) -> Option<Rtc> {
    if let Ok(data) = std::fs::read(path) {
        if let Ok(loaded_rtc) = bincode::deserialize::<Rtc>(&data) {
            return Some(loaded_rtc);
        }
    }
    None
}

fn save_rtc_state(path: &Path, rtc: &Rtc) {
    if let Ok(serialized) = bincode::serialize(rtc) {
        let _ = std::fs::write(path, serialized);
    }
}
