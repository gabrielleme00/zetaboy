mod cart_type;
mod licensee;

use std::{error::Error, fs, path::Path};

pub struct Cart {
    pub rom_data: Vec<u8>,
    header: Header,
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
        Ok(Self { rom_data, header })
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
            Err(_) => return String::from("Invalid UTF-8"),
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
