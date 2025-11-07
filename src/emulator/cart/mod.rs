mod mbc;
mod licensee;

use std::{error::Error, fs, path::Path};
use mbc::MbcType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Cart {
    pub rom_data: Vec<u8>,
    header: Header,
    pub ram_data: Vec<u8>,
    mbc_type: MbcType,
}

#[derive(Clone, Deserialize, Serialize)]
struct Header {
    // _entry: [u8; 4],
    // _logo: [u8; 0x30],
    title: [u8; 16],
    new_lic_code: u16,
    _sgb_flag: u8,
    cart_type: u8,
    rom_size: u8,
    ram_size: u8,
    _dest_code: u8,
    lic_code: u8,
    version: u8,
    checksum: u8,
    cgb_flag: u8,
}

impl Header {
    fn get_rom_banks_number(&self) -> usize {
        match self.rom_size {
            0x00 => 2,
            0x01 => 4,
            0x02 => 8,
            0x03 => 16,
            0x04 => 32,
            0x05 => 64,
            0x06 => 128,
            0x07 => 256,
            0x08 => 512,
            0x52 => 72,
            0x53 => 80,
            0x54 => 96,
            _ => {
                println!("Unknown ROM size code: {:#04X}", self.rom_size);
                0
            }
        }
    }

    fn get_ram_size_in_bytes(&self) -> usize {
        match self.ram_size {
            0 => 0,
            1 => 0,          // 2 kB (unused)
            2 => 8 * 1024,   // 8 kB
            3 => 32 * 1024,  // 32 kB
            4 => 128 * 1024, // 128 kB
            5 => 64 * 1024,  // 64 kB
            _ => {
                println!("Unknown RAM size code: {:#04X}", self.ram_size);
                0
            }
        }
    }

    fn get_ram_banks_number(&self) -> usize {
        match self.ram_size {
            0 => 0,
            1 => 0, // 2 kB (unused)
            2 => 1, // 8 kB
            3 => 4, // 32 kB
            4 => 16, // 128 kB
            5 => 8, // 64 kB
            _ => {
                println!("Unknown RAM size code: {:#04X}", self.ram_size);
                0
            }
        }
    }
}

impl Cart {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let rom_data = fs::read(path)?;
        let header = Header::new(&rom_data);

        let ram_size = header.get_ram_size_in_bytes();
        let rom_banks = header.get_rom_banks_number();
        let ram_banks = header.get_ram_banks_number();
        let mbc_type = MbcType::from_byte(header.cart_type, rom_banks, ram_banks);

         // Load RTC data if applicable

        Ok(Self {
            rom_data,
            header,
            ram_data: vec![0; ram_size],
            mbc_type,
        })
    }

    pub fn has_battery(&self) -> bool {
        MbcType::has_battery(self.header.cart_type)
    }

    pub fn is_cgb(&self) -> bool {
        self.header.cgb_flag == 0x80 || self.header.cgb_flag == 0xC0
    }

    pub fn is_cgb_only(&self) -> bool {
        self.header.cgb_flag == 0xC0
    }

    pub fn print_info(&self) {
        let checksum = match self.is_checksum_valid() {
            true => format!("{:#04X} (PASSED)", self.header.checksum),
            false => format!("{:#04X} (FAILED)", self.header.checksum),
        };

        let mode = if self.is_cgb_only() {
            "CGB Only"
        } else if self.is_cgb() {
            "CGB Enhanced"
        } else {
            "DMG"
        };

        println!("#------ ROM INFO ------#");
        println!("| Title    : {}", self.header.title_to_string());
        println!("| Mode     : {}", mode);
        println!("| Type     : {}", self.header.cart_type_to_string());
        println!("| ROM Size : {}", self.header.rom_size_to_string());
        println!("| RAM Size : {}", self.header.ram_size_to_string());
        println!("| Licensee : {}", self.header.lic_to_string());
        println!("| Version  : {}", self.header.version);
        println!("| Checksum : {}", checksum);
        println!("#----------------------#");
    }

    /// Read from ROM area (0x0000-0x7FFF)
    pub fn read_rom(&self, address: u16) -> u8 {
        self.mbc_type.read_rom(&self.rom_data, address)
    }

    /// Write to ROM area (triggers MBC operations)
    pub fn write_rom(&mut self, address: u16, value: u8) {
        self.mbc_type.write_rom(address, value);
    }

    /// Read from RAM area (0xA000-0xBFFF)
    pub fn read_ram(&self, address: u16) -> u8 {
        self.mbc_type.read_ram(&self.ram_data, address)
    }

    /// Write to RAM area (0xA000-0xBFFF)
    pub fn write_ram(&mut self, address: u16, value: u8) {
        self.mbc_type.write_ram(&mut self.ram_data, address, value);
    }

    fn is_checksum_valid(&self) -> bool {
        let mut x: u8 = 0;
        for i in 0x134..=0x14C {
            x = x.wrapping_sub(self.rom_data[i]).wrapping_sub(1);
        }
        x == self.header.checksum
    }
}

impl Header {
    fn new(rom: &[u8]) -> Self {
        Self {
            // _entry: rom[0x100..=0x103].try_into().unwrap(),
            // _logo: rom[0x104..=0x133].try_into().unwrap(),
            title: rom[0x134..=0x143].try_into().unwrap(),
            new_lic_code: ((rom[0x144] as u16) << 8) | rom[0x145] as u16,
            _sgb_flag: rom[0x146],
            cart_type: rom[0x147],
            rom_size: rom[0x148],
            ram_size: rom[0x149],
            _dest_code: rom[0x14A],
            lic_code: rom[0x14B],
            version: rom[0x14C],
            checksum: rom[0x14D],
            cgb_flag: rom[0x143],
        }
    }

    fn title_to_string(&self) -> String {
        let title_from_utf8 = match std::str::from_utf8(&self.title) {
            Ok(v) => v,
            Err(_) => return String::from("Unknown Title"),
        };
        String::from(title_from_utf8).trim_matches('\0').to_string()
    }

    fn lic_to_string(&self) -> &'static str {
        match self.lic_code {
            0x33 => licensee::new_name(self.new_lic_code),
            _ => licensee::old_name(self.lic_code),
        }
    }

    fn cart_type_to_string(&self) -> &'static str {
        MbcType::name(self.cart_type)
    }

    fn rom_size_to_string(&self) -> String {
        format!("{} kB", 32 << self.rom_size)
    }

    fn ram_size_to_string(&self) -> String {
        match self.ram_size {
            0 => format!("No RAM"),
            1 => format!("Unused"),
            2 => format!("8 kB (1 bank)"),
            3 => format!("32 kB (4 banks of 8 kB each)"),
            4 => format!("128 kB (16 banks of 8 kB each)"),
            5 => format!("64 kB (8 banks of 8 kB each)"),
            _ => format!("UNKNOWN ({})", self.ram_size),
        }
    }
}
