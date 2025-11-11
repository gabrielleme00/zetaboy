use super::Mbc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const EEPROM_SIZE: usize = 128; // 256 bytes / 2 = 128 16-bit words

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
enum EepromCommand {
    None,
    Read,
    Write,
    Erase,
    Ewen,  // Erase/Write Enable
    Ewds,  // Erase/Write Disable
    Eral,  // Erase All
    Wral,  // Write All
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Eeprom {
    data: Vec<u16>,
    cs: bool,      // Chip Select
    clk: bool,     // Clock
    di: bool,      // Data In
    do_: bool,     // Data Out
    write_enabled: bool,
    
    // State machine
    command: EepromCommand,
    address: u8,
    bit_count: u8,
    shift_register: u16,
    processing: bool,
}

impl PartialEq for Eeprom {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for Eeprom {}

impl Eeprom {
    fn new() -> Self {
        Self {
            data: vec![0xFFFF; EEPROM_SIZE],
            cs: false,
            clk: false,
            di: false,
            do_: true,
            write_enabled: false,
            command: EepromCommand::None,
            address: 0,
            bit_count: 0,
            shift_register: 0,
            processing: false,
        }
    }

    fn write(&mut self, value: u8) {
        let new_cs = (value & 0x80) != 0;
        let new_clk = (value & 0x40) != 0;
        let new_di = (value & 0x02) != 0;

        // CS rising edge - start of command
        if new_cs && !self.cs {
            self.bit_count = 0;
            self.shift_register = 0;
            self.command = EepromCommand::None;
            self.processing = false;
            self.do_ = true;
        }

        // CS falling edge - end of command
        if !new_cs && self.cs {
            self.bit_count = 0;
            self.shift_register = 0;
        }

        self.cs = new_cs;
        self.di = new_di;

        // Clock rising edge - shift in bit
        if new_clk && !self.clk && self.cs {
            self.shift_bit_in();
        }

        self.clk = new_clk;
    }

    fn shift_bit_in(&mut self) {
        if self.processing {
            // Don't accept new bits while processing a command
            return;
        }

        self.shift_register = (self.shift_register << 1) | (self.di as u16);
        self.bit_count += 1;

        match self.command {
            EepromCommand::None => {
                // Waiting for command (10 bits total)
                if self.bit_count >= 10 {
                    self.decode_command();
                }
            }
            EepromCommand::Read => {
                // Already decoded, should be shifting out
            }
            EepromCommand::Write | EepromCommand::Wral => {
                // Need 16 more bits of data after command
                if self.bit_count >= 26 {
                    self.execute_write();
                }
            }
            _ => {}
        }
    }

    fn decode_command(&mut self) {
        let opcode = (self.shift_register >> 6) & 0xF;
        let addr = (self.shift_register & 0x7F) as u8;

        match opcode {
            0b0000 => {
                // EWDS - Erase/Write Disable
                self.command = EepromCommand::Ewds;
                self.write_enabled = false;
            }
            0b0001 => {
                // WRAL - Write All
                self.command = EepromCommand::Wral;
                self.bit_count = 10; // Ready to receive 16 data bits
            }
            0b0010 => {
                // ERAL - Erase All
                self.command = EepromCommand::Eral;
                if self.write_enabled {
                    self.data.fill(0xFFFF);
                    self.processing = true;
                }
            }
            0b0011 => {
                // EWEN - Erase/Write Enable
                self.command = EepromCommand::Ewen;
                self.write_enabled = true;
            }
            0b0100..=0b0111 => {
                // WRITE
                self.command = EepromCommand::Write;
                self.address = addr;
                self.bit_count = 10; // Ready to receive 16 data bits
            }
            0b1000..=0b1011 => {
                // READ
                self.command = EepromCommand::Read;
                self.address = addr;
                self.shift_register = self.read_word(addr);
                self.bit_count = 0;
                self.start_read();
            }
            0b1100..=0b1111 => {
                // ERASE
                self.command = EepromCommand::Erase;
                self.address = addr;
                if self.write_enabled && (addr as usize) < EEPROM_SIZE {
                    self.data[addr as usize] = 0xFFFF;
                    self.processing = true;
                }
            }
            _ => {}
        }
    }

    fn execute_write(&mut self) {
        let data = self.shift_register & 0xFFFF;
        
        match self.command {
            EepromCommand::Write => {
                if self.write_enabled && (self.address as usize) < EEPROM_SIZE {
                    self.data[self.address as usize] = data;
                    self.processing = true;
                }
            }
            EepromCommand::Wral => {
                if self.write_enabled {
                    self.data.fill(data);
                    self.processing = true;
                }
            }
            _ => {}
        }
    }

    fn start_read(&mut self) {
        // Prepare to shift out 16 bits
        self.do_ = (self.shift_register & 0x8000) != 0;
    }

    fn read_word(&self, addr: u8) -> u16 {
        if (addr as usize) < EEPROM_SIZE {
            self.data[addr as usize]
        } else {
            0xFFFF
        }
    }

    fn read(&mut self) -> u8 {
        let result = if self.processing {
            // RDY signal - returns 1 when done (we simulate instant completion)
            self.processing = false;
            0x01
        } else if self.command == EepromCommand::Read && self.cs {
            // Shift out data during read
            let bit = if self.do_ { 0x01 } else { 0x00 };
            
            // Advance to next bit on clock edge (simplified)
            if self.clk && self.bit_count < 16 {
                self.bit_count += 1;
                self.shift_register <<= 1;
                self.do_ = (self.shift_register & 0x8000) != 0;
            }
            
            bit
        } else {
            if self.do_ { 0x01 } else { 0x00 }
        };

        result
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Mbc7 {
    rom_bank: u8,
    ram_enabled_1: bool,  // 0x0000-0x1FFF enable
    ram_enabled_2: bool,  // 0x4000-0x5FFF enable (must be 0x40)
    rom_banks: usize,
    
    // Accelerometer
    accel_x: u16,
    accel_y: u16,
    accel_latched: bool,
    
    // EEPROM
    eeprom: Eeprom,
    
    // Save path for EEPROM
    #[serde(skip)]
    eeprom_path: PathBuf,
}

impl PartialEq for Mbc7 {
    fn eq(&self, other: &Self) -> bool {
        self.rom_bank == other.rom_bank
            && self.ram_enabled_1 == other.ram_enabled_1
            && self.ram_enabled_2 == other.ram_enabled_2
            && self.rom_banks == other.rom_banks
            && self.accel_x == other.accel_x
            && self.accel_y == other.accel_y
            && self.accel_latched == other.accel_latched
            && self.eeprom == other.eeprom
    }
}

impl Eq for Mbc7 {}

impl Mbc7 {
    pub fn new(rom_banks: usize, rom_path: PathBuf) -> Self {
        let mut eeprom_path = rom_path;
        eeprom_path.set_extension("eeprom");
        
        let eeprom = Self::load_eeprom(&eeprom_path).unwrap_or_else(Eeprom::new);
        
        Self {
            rom_bank: 1,
            ram_enabled_1: false,
            ram_enabled_2: false,
            rom_banks,
            accel_x: 0x8000,
            accel_y: 0x8000,
            accel_latched: false,
            eeprom,
            eeprom_path,
        }
    }

    fn load_eeprom(path: &PathBuf) -> Option<Eeprom> {
        if let Ok(data) = std::fs::read(path) {
            if let Ok(eeprom) = bincode::deserialize::<Eeprom>(&data) {
                return Some(eeprom);
            }
        }
        None
    }

    fn save_eeprom(&self) {
        if let Ok(serialized) = bincode::serialize(&self.eeprom) {
            let _ = std::fs::write(&self.eeprom_path, serialized);
        }
    }

    fn ram_enabled(&self) -> bool {
        self.ram_enabled_1 && self.ram_enabled_2
    }
}

impl Mbc for Mbc7 {
    fn read_rom(&self, rom_data: &[u8], address: u16) -> u8 {
        match address {
            // ROM Bank 0 (fixed)
            0x0000..=0x3FFF => {
                if (address as usize) < rom_data.len() {
                    rom_data[address as usize]
                } else {
                    0xFF
                }
            }
            // ROM Bank 1-127 (switchable)
            0x4000..=0x7FFF => {
                let bank = (self.rom_bank as usize) % self.rom_banks.max(1);
                let offset = (bank * 0x4000) + (address as usize - 0x4000);
                if offset < rom_data.len() {
                    rom_data[offset]
                } else {
                    0xFF
                }
            }
            _ => 0xFF,
        }
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            // 0x0000-0x1FFF: RAM Enable 1
            0x0000..=0x1FFF => {
                self.ram_enabled_1 = (value & 0x0F) == 0x0A;
            }
            // 0x2000-0x3FFF: ROM Bank Number
            0x2000..=0x3FFF => {
                self.rom_bank = if value == 0 { 1 } else { value & 0x7F };
            }
            // 0x4000-0x5FFF: RAM Enable 2
            0x4000..=0x5FFF => {
                self.ram_enabled_2 = value == 0x40;
            }
            _ => {}
        }
    }

    fn read_ram(&self, _ram_data: &[u8], address: u16) -> u8 {
        if !self.ram_enabled() {
            return 0xFF;
        }

        // Extract register index from bits 4-7
        let reg = ((address >> 4) & 0x0F) as u8;

        match reg {
            0x0 | 0x1 => {
                // Latch registers - write only
                0xFF
            }
            0x2 => {
                // Accelerometer X low byte
                (self.accel_x & 0xFF) as u8
            }
            0x3 => {
                // Accelerometer X high byte
                (self.accel_x >> 8) as u8
            }
            0x4 => {
                // Accelerometer Y low byte
                (self.accel_y & 0xFF) as u8
            }
            0x5 => {
                // Accelerometer Y high byte
                (self.accel_y >> 8) as u8
            }
            0x6 => {
                // Unknown - always 0x00
                0x00
            }
            0x7 => {
                // Unknown - always 0xFF
                0xFF
            }
            0x8 => {
                // EEPROM register
                let mut eeprom = self.eeprom.clone();
                let result = eeprom.read();
                result
            }
            _ => {
                // 0x9-0xF: Unused
                0xFF
            }
        }
    }

    fn write_ram(&mut self, _ram_data: &mut [u8], address: u16, value: u8) {
        if !self.ram_enabled() {
            return;
        }

        // Extract register index from bits 4-7
        let reg = ((address >> 4) & 0x0F) as u8;

        match reg {
            0x0 => {
                // Erase latched data - write 0x55
                if value == 0x55 {
                    self.accel_x = 0x8000;
                    self.accel_y = 0x8000;
                    self.accel_latched = false;
                }
            }
            0x1 => {
                // Latch accelerometer - write 0xAA
                if value == 0xAA && !self.accel_latched {
                    // Simulate accelerometer reading
                    // Centered at 0x81D0, Earth's gravity affects by ~0x70
                    // For now, return centered values (no acceleration)
                    self.accel_x = 0x81D0;
                    self.accel_y = 0x81D0;
                    self.accel_latched = true;
                }
            }
            0x8 => {
                // EEPROM register
                self.eeprom.write(value);
                // Save EEPROM on writes to persist data
                self.save_eeprom();
            }
            _ => {
                // Other registers are read-only or unused
            }
        }
    }
}
