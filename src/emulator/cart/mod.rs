mod licensee;
mod cart_type;

const HEADER_SIZE: usize = 0x50;
const HEADER_OFFSET:usize = 0x100;

use std::error::Error;
use std::fs;
use std::path::Path;

struct Header {
    entry: [u8; 4],
    logo: [u8; 0x30],
    title: [u8; 16],
    new_lic_code: u16,
    sgb_flag: u8,
    cart_type: u8,
    rom_size: u8,
    ram_size: u8,
    dest_code: u8,
    lic_code: u8,
    version: u8,
    checksum: u8,
    global_checksum: u16,
}

impl Header {
    fn new(rom: &[u8]) -> Self {
        Self {
            entry: rom[0x100..=0x103].try_into().unwrap(),
            logo: rom[0x104..=0x133].try_into().unwrap(),
            title: rom[0x134..=0x143].try_into().unwrap(),
            new_lic_code: ((rom[0x144] as u16) << 8) | rom[0x145] as u16,
            sgb_flag: rom[0x146],
            cart_type: rom[0x147],
            rom_size: rom[0x148],
            ram_size: rom[0x149],
            dest_code: rom[0x1A],
            lic_code: rom[0x14B],
            version: rom[0x14C],
            checksum: rom[0x14D],
            global_checksum: ((rom[0x14E] as u16) << 8) | rom[0x14F] as u16,
        }
    }

    fn new_empty() -> Self {
        Self {
            entry: [0; 4],
            logo: [0; 0x30],
            title: [0; 16],
            new_lic_code: 0,
            sgb_flag: 0,
            cart_type: 0,
            rom_size: 0,
            ram_size: 0,
            dest_code: 0,
            lic_code: 0,
            version: 0,
            checksum: 0,
            global_checksum: 0,
        }
    }

    fn title_to_string(&self) -> String {
        let title_from_utf8 = std::str::from_utf8(&self.title).unwrap();
        String::from(title_from_utf8).trim_matches('\0').to_string()
    }

    fn lic_to_string(&self) -> &'static str {
        let code = self.new_lic_code;
        if code <= 0xA4 {
            return licensee::name(code);
        }
        "UNKKOWN"
    }

    fn cart_type_to_string(&self) -> &'static str {
        let code = self.cart_type;
        if code <= 0x22 {
            return cart_type::name(code);
        }
        "UNKKOWN"
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
            _ => format!("UNKNOWN ({})", self.ram_size)
        }
    }

    pub fn print(&self) {
        println!("#------ ROM INFO ------#");
        println!("| Title    : {}", self.title_to_string());
        println!("| Type     : {}", self.cart_type_to_string());
        println!("| ROM Size : {}", self.rom_size_to_string());
        println!("| RAM Size : {}", self.ram_size_to_string());
        println!("| Licensee : {}", self.lic_to_string());
        println!("| Version  : {:?}", self.version);
        println!("#----------------------#");
    }
}

pub struct Cart {
    rom_data: Vec<u8>,
    rom_size: u32,
    header: Header,
}

impl Cart {
    pub fn new() -> Self {
        Self {
            rom_data: vec![],
            rom_size: 0,
            header: Header::new_empty(),
        }
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        self.rom_data = fs::read(path)?;
        self.rom_size = self.rom_data.len() as u32;
        Ok(())
    }

    pub fn read_header(&mut self) {
        self.header = Header::new(&self.rom_data);
        self.header.print();
    }
}
