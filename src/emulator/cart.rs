mod cart_type;
mod licensee;

use std::{error::Error, fs, path::Path};

pub struct Cart {
    pub rom_data: Vec<u8>,
    header: Header,
    // MBC state
    rom_bank: usize,
    ram_bank: usize,
    ram_enabled: bool,
    ram_data: Vec<u8>,
}

struct Header {
    _entry: [u8; 4],
    _logo: [u8; 0x30],
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
}

impl Cart {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let rom_data = fs::read(path)?;
        let header = Header::new(&rom_data);
        
        // Initialize RAM based on header info
        let ram_size = match header.ram_size {
            0 => 0,
            1 => 0, // Unused
            2 => 8 * 1024,      // 8 kB
            3 => 32 * 1024,     // 32 kB
            4 => 128 * 1024,    // 128 kB
            5 => 64 * 1024,     // 64 kB
            _ => 0,
        };
        
        Ok(Self { 
            rom_data, 
            header,
            rom_bank: 1, // Start at bank 1 (bank 0 is fixed)
            ram_bank: 0,
            ram_enabled: false,
            ram_data: vec![0; ram_size],
        })
    }

    pub fn print_info(&self) {
        let checksum = match self.is_checksum_valid() {
            true => format!("{:#04X} (PASSED)", self.header.checksum),
            false => format!("{:#04X} (FAILED)", self.header.checksum),
        };

        println!("#------ ROM INFO ------#");
        println!("| Title    : {}", self.header.title_to_string());
        println!("| Type     : {}", self.header.cart_type_to_string());
        println!("| ROM Size : {}", self.header.rom_size_to_string());
        println!("| RAM Size : {}", self.header.ram_size_to_string());
        println!("| Licensee : {}", self.header.lic_to_string());
        println!("| Version  : {}", self.header.version);
        println!("| Checksum : {}", checksum);
        println!("#----------------------#");
    }

    pub fn get_title(&self) -> String {
        self.header.title_to_string()
    }

    /// Read from ROM area (0x0000-0x7FFF)
    pub fn read_rom(&self, address: u16) -> u8 {
        let address = address as usize;
        match address {
            0x0000..=0x3FFF => {
                // Bank 0 is always fixed
                if address < self.rom_data.len() {
                    self.rom_data[address]
                } else {
                    0xFF
                }
            }
            0x4000..=0x7FFF => {
                // Switchable ROM bank
                let bank_offset = 0x4000 * self.rom_bank;
                let real_address = bank_offset + (address - 0x4000);
                if real_address < self.rom_data.len() {
                    self.rom_data[real_address]
                } else {
                    0xFF
                }
            }
            _ => 0xFF,
        }
    }

    /// Write to ROM area (triggers MBC operations)
    pub fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                // RAM Enable
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                // ROM Bank Number (lower 5 bits)
                let bank = (value & 0x1F) as usize;
                // Bank 0 is treated as bank 1
                self.rom_bank = if bank == 0 { 1 } else { bank };
            }
            0x4000..=0x5FFF => {
                // RAM Bank Number or upper ROM bank bits
                self.ram_bank = (value & 0x03) as usize;
            }
            0x6000..=0x7FFF => {
                // Banking Mode Select (for advanced MBCs)
                // For now, we'll ignore this
            }
            _ => {}
        }
    }

    /// Read from RAM area (0xA000-0xBFFF)
    pub fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled || self.ram_data.is_empty() {
            return 0xFF;
        }

        let address = (address - 0xA000) as usize;
        let bank_offset = 0x2000 * self.ram_bank;
        let real_address = bank_offset + address;

        if real_address < self.ram_data.len() {
            self.ram_data[real_address]
        } else {
            0xFF
        }
    }

    /// Write to RAM area (0xA000-0xBFFF)
    pub fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled || self.ram_data.is_empty() {
            return;
        }

        let address = (address - 0xA000) as usize;
        let bank_offset = 0x2000 * self.ram_bank;
        let real_address = bank_offset + address;

        if real_address < self.ram_data.len() {
            self.ram_data[real_address] = value;
        }
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
            _entry: rom[0x100..=0x103].try_into().unwrap(),
            _logo: rom[0x104..=0x133].try_into().unwrap(),
            title: rom[0x134..=0x143].try_into().unwrap(),
            new_lic_code: ((rom[0x144] as u16) << 8) | rom[0x145] as u16,
            _sgb_flag: rom[0x146],
            cart_type: rom[0x147],
            rom_size: rom[0x148],
            ram_size: rom[0x149],
            _dest_code: rom[0x1A],
            lic_code: rom[0x14B],
            version: rom[0x14C],
            checksum: rom[0x14D],
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
        cart_type::name(self.cart_type)
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
