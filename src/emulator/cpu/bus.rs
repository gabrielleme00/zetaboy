<<<<<<< HEAD
mod regions;
mod ram;

use crate::emulator::ppu::*;
use ram::*;
=======
use crate::emulator::ppu::*;

const HRAM_SIZE: usize = 0x7F;
const WRAM_SIZE: usize = 0x8000;
>>>>>>> c934821f511b7999703abb6d1ff8d829f2498e1b

pub struct MemoryBus {
    cart: Vec<u8>,
    ppu: PPU,
<<<<<<< HEAD
    ram: RAM,
    int_enable: u8,
    int_flag: u8,
=======
    hram: [u8; HRAM_SIZE],
    wram: [u8; WRAM_SIZE],
    wram_bank: usize,
    int_enable: u8,
>>>>>>> c934821f511b7999703abb6d1ff8d829f2498e1b
}

impl MemoryBus {
    pub fn new(cart_data: &Vec<u8>) -> Self {
        Self {
            cart: cart_data.to_vec(),
            ppu: PPU::new(),
            ram: RAM::new(),
            int_enable: 0,
            int_flag: 0,
            hram: [0; HRAM_SIZE],
            wram: [0; WRAM_SIZE],
            wram_bank: 1,
            int_enable: 0,
        }
    }

    /// Returns a byte from the `address`.
    pub fn read_byte(&self, address: u16) -> u8 {
        let address = address as usize;
        match address{
            0x0000 ..= 0x7FFF => self.cart[address],
            0x8000 ..= 0x9FFF => self.ppu.read_vram(address),
            0xA000 ..= 0xBFFF => self.cart[address],
            0xC000 ..= 0xDFFF => self.ram.read_wram(address),
            0xE000 ..= 0xFDFF => todo!("Read ECHO RAM"),
            0xFE00 ..= 0xFE9F => self.ppu.read_vram(address),
            0xFF00 => todo!("Read KEYPAD"),
            0xFF01 ..= 0xFF02 => todo!("Read SERIAL"),
            0xFF04 ..= 0xFF07 => todo!("Read TIMER"),
            0xFF0F => self.int_flag,
            0xFF10 ..= 0xFF3F => todo!("Read SOUND"),
            0xFF4D => todo!("Read GBSPEED"),
            0xFF40 ..= 0xFF4F => self.ppu.read_vram(address),
            0xFF51 ..= 0xFF55 => todo!("Read HDMA"),
            0xFF68 ..= 0xFF6B => self.ppu.read_vram(address),
            0xFF70 => todo!("Read current RAM bank"),
            0xFF80 ..= 0xFFFE => self.ram.read_hram(address),
            0xFFFF => self.int_enable,
            _ => 0,
        }
    }

    /// Returns 2 bytes from the `address` (little-endian).
    pub fn read_word(&self, address: u16) -> u16 {
        let a = self.read_byte(address) as u16;
        let b = self.read_byte(address + 1) as u16;
        (b << 8) | a
    }

    /// Writes a byte of `value` to the `address`.
    pub fn write_byte(&mut self, address: u16, value: u8) {
        let address = address as usize;
        match address {
<<<<<<< HEAD
            0x0000 ..= 0x7FFF => self.cart[address] = value,
            0x8000 ..= 0x9FFF => self.ppu.write_vram(address, value),
            0xA000 ..= 0xBFFF => self.cart[address] = value,
            0xC000 ..= 0xDFFF => self.ram.write_wram(address, value),
            0xE000 ..= 0xFDFF => (),
            0xFE00 ..= 0xFE9F => self.ppu.write_vram(address, value),
            0xFF00 => todo!("Write KEYPAD"),
            0xFF01 ..= 0xFF02 => todo!("Write SERIAL"),
            0xFF04 ..= 0xFF07 => todo!("Write TIMER"),
            0xFF0F => self.int_flag = value,
            0xFF10 ..= 0xFF3F => todo!("Write SOUND"),
            0xFF4D => todo!("Write GBSPEED"),
            0xFF40 ..= 0xFF4F => self.ppu.write_vram(address, value),
            0xFF51 ..= 0xFF55 => todo!("Read HDMA"),
            0xFF68 ..= 0xFF6B => self.ppu.write_vram(address, value),
            0xFF70 => todo!("Read current RAM bank"),
            0xFF80 ..= 0xFFFE => self.ram.write_hram(address, value),
            0xFFFF => self.int_enable = value,
            _ => (),
=======
            0x0000..=0x7FFF => self.cart[address] = value,
            0x8000..=0x9FFF => self.ppu.write_vram(address, value),
            0xA000..=0xBFFF => self.cart[address] = value,
            0xC000..=0xCFFF => self.wram[address - 0xC000] = value,
            0xD000..=0xDFFF => self.wram[address - 0xD000 + 0x1000 * self.wram_bank] = value,
            0xE000..=0xEFFF => self.wram[address - 0xE000] = value,
            0xF000..=0xFDFF => self.wram[address - 0xF000 + 0x1000 * self.wram_bank] = value,
            0xFE00..=0xFE9F => todo!("Unsupported bus read: {:#04X} ({})", address, "OAM"),
            0xFF00..=0xFF7F => todo!("Unsupported bus read: {:#04X} ({})", address, "IO Regs"),
            0xFF80..=0xFFFE => self.hram[address - 0xFF80] = value,
            0xFFFF => self.int_enable = value,
            _ => {}
>>>>>>> c934821f511b7999703abb6d1ff8d829f2498e1b
        };
    }
}
