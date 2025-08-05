pub mod io_registers;

use crate::emulator::ppu::*;
use crate::emulator::timer::Timer;
use io_registers::*;

const HRAM_SIZE: usize = 0x7F;
const WRAM_SIZE: usize = 0x8000;

pub struct MemoryBus {
    cart: Vec<u8>,
    pub ppu: PPU,
    pub timer: Timer,
    hram: [u8; HRAM_SIZE],
    wram: [u8; WRAM_SIZE],
    wram_bank: usize,
    pub io: IORegisters,
}

impl MemoryBus {
    pub fn new(cart_data: &Vec<u8>) -> Self {
        Self {
            cart: cart_data.to_vec(),
            ppu: PPU::new(),
            timer: Timer::new(),
            hram: [0; HRAM_SIZE],
            wram: [0; WRAM_SIZE],
            wram_bank: 1,
            io: IORegisters::new(),
        }
    }

    /// Returns a byte from the `address`.
    pub fn read_byte(&self, address: u16) -> u8 {
        let address = address as usize;
        match address {
            0x0000..=0x7FFF => self.cart[address],
            0x8000..=0x9FFF => self.ppu.read_vram(address as u16),
            0xA000..=0xBFFF => self.cart[address],
            0xC000..=0xCFFF => self.wram[address - 0xC000],
            0xD000..=0xDFFF => self.wram[address - 0xD000 + 0x1000 * self.wram_bank],
            0xE000..=0xEFFF => self.wram[address - 0xE000], // WRAM mirror
            0xF000..=0xFDFF => self.wram[address - 0xF000 + 0x1000 * self.wram_bank],
            0xFE00..=0xFE9F => self.ppu.read_oam((address - 0xFE00) as u16),
            0xFF68..=0xFF69 => self.ppu.read_bg_palette_ram(address as u16),
            0xFF6A..=0xFF6B => self.ppu.read_obj_palette_ram(address as u16),
            0xFF00..=0xFF7F => match address {
                0xFF68..=0xFF69 => self.ppu.read_bg_palette_ram(address as u16),
                0xFF6A..=0xFF6B => self.ppu.read_obj_palette_ram(address as u16),
                _ => self.io.read(address as u16),
            },
            0xFF80..=0xFFFE => self.hram[address - 0xFF80],
            0xFFFF => self.io.int_enable,
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
    pub fn write_byte(&mut self, address: u16, value: u8) -> Result<(), String> {
        let address = address as usize;
        match address {
            0x0000..=0x7FFF => Ok(self.cart[address] = value),
            0x8000..=0x9FFF => self.ppu.write_vram(address as u16, value),
            0xA000..=0xBFFF => Ok(self.cart[address] = value),
            0xC000..=0xCFFF => Ok(self.wram[address - 0xC000] = value),
            0xD000..=0xDFFF => Ok(self.wram[address - 0xD000 + 0x1000 * self.wram_bank] = value),
            0xE000..=0xEFFF => Ok(self.wram[address - 0xE000] = value), // WRAM mirror
            0xF000..=0xFDFF => Ok(self.wram[address - 0xF000 + 0x1000 * self.wram_bank] = value),
            0xFE00..=0xFE9F => self.ppu.write_oam((address - 0xFE00) as u16, value),
            0xFEA0..=0xFEFF => Ok(()), // Unused OAM area
            0xFF00..=0xFF7F => {
                match address {
                    0xFF04 => { // DIV register - special handling
                        self.timer.reset_div(&mut self.io.div);
                        Ok(())
                    },
                    0xFF68..=0xFF69 => self.ppu.write_bg_palette_ram(address as u16, value),
                    0xFF6A..=0xFF6B => self.ppu.write_obj_palette_ram(address as u16, value),
                    _ => Ok(self.io.write(address as u16, value)),
                }
            },
            0xFF80..=0xFFFE => {
                if address == 0xFF80 && value == 0xFF {
                    panic!();
                }
                Ok(self.hram[address - 0xFF80] = value)
            },
            0xFFFF => Ok(self.io.int_flag = value & 0x1F),
            _ => Err(format!("Invalid write address: {:#04X}", address)),
        }
    }

    pub fn write_word(&mut self, address: u16, value: u16) -> Result<(), String> {
        self.write_byte(address, (value & 0xFF) as u8)?;
        self.write_byte(address + 1, (value >> 8) as u8)
    }
}
