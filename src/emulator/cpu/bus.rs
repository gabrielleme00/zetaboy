use crate::emulator::ppu::*;

const HRAM_SIZE: usize = 0x7F;
const WRAM_SIZE: usize = 0x8000;

pub struct MemoryBus {
    cart: Vec<u8>,
    ppu: PPU,
    hram: [u8; HRAM_SIZE],
    wram: [u8; WRAM_SIZE],
    wram_bank: usize,
    int_enable: u8,
}

impl MemoryBus {
    pub fn new(cart_data: &Vec<u8>) -> Self {
        Self {
            cart: cart_data.to_vec(),
            ppu: PPU::new(),
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
            0x0000..=0x7FFF => self.cart[address],
            0x8000..=0x9FFF => self.ppu.read_vram(address),
            0xA000..=0xBFFF => self.cart[address],
            0xC000..=0xCFFF => self.wram[address - 0xC000],
            0xD000..=0xDFFF => self.wram[address - 0xD000 + 0x1000 * self.wram_bank],
            0xE000..=0xEFFF => self.wram[address - 0xE000],
            0xF000..=0xFDFF => self.wram[address - 0xF000 + 0x1000 * self.wram_bank],
            0xFE00..=0xFE9F => todo!("Unsupported bus read: {:#04X} ({})", address, "OAM"),
            0xFF00..=0xFF7F => todo!("Unsupported bus read: {:#04X} ({})", address, "IO Regs"),
            0xFF80..=0xFFFE => self.hram[address - 0xFF80],
            0xFFFF => self.int_enable,
            _ => 0
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
        };
    }
}
