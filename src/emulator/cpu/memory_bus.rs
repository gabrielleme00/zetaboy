pub mod io_registers;

use crate::emulator::cart::Cart;
use crate::emulator::ppu::*;
use crate::emulator::timer::Timer;
use io_registers::*;

const HRAM_SIZE: usize = 0x7F;
const WRAM_SIZE: usize = 0x8000;

pub struct MemoryBus {
    cart: Cart,
    pub ppu: PPU,
    pub timer: Timer,
    hram: [u8; HRAM_SIZE],
    wram: [u8; WRAM_SIZE],
    wram_bank: usize,
    pub io: IORegisters,
}

impl MemoryBus {
    pub fn new(cart: Cart) -> Self {
        Self {
            cart,
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
        let address_usize = address as usize;
        match address {
            0x0000..=0x7FFF => self.cart.read_rom(address),
            0x8000..=0x9FFF => self.ppu.read_vram(address),
            0xA000..=0xBFFF => self.cart.read_ram(address),
            0xC000..=0xCFFF => self.wram[address_usize - 0xC000],
            0xD000..=0xDFFF => self.wram[address_usize - 0xD000 + 0x1000 * self.wram_bank],
            0xE000..=0xEFFF => self.wram[address_usize - 0xE000], // WRAM mirror
            0xF000..=0xFDFF => self.wram[address_usize - 0xF000 + 0x1000 * self.wram_bank],
            0xFE00..=0xFE9F => self.ppu.read_oam((address_usize - 0xFE00) as u16),
            0xFF68..=0xFF69 => self.ppu.read_bg_palette_ram(address),
            0xFF6A..=0xFF6B => self.ppu.read_obj_palette_ram(address),
            0xFF00..=0xFF7F => match address {
                0xFF04 => self.timer.div,
                0xFF05 => self.timer.tima,
                0xFF06 => self.timer.tma,
                0xFF07 => self.timer.tac,
                0xFF44 => 0x90, // TEMP: for logs only
                0xFF68..=0xFF69 => self.ppu.read_bg_palette_ram(address),
                0xFF6A..=0xFF6B => self.ppu.read_obj_palette_ram(address),
                _ => self.io.read(address),
            },
            0xFF80..=0xFFFE => self.hram[address_usize - 0xFF80],
            0xFFFF => self.io.read(0x0FFFF),
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
        let address_usize = address as usize;
        match address {
            0x0000..=0x7FFF => self.cart.write_rom(address, value),
            0x8000..=0x9FFF => self.ppu.write_vram(address, value),
            0xA000..=0xBFFF => self.cart.write_ram(address, value),
            0xC000..=0xCFFF => self.wram[address_usize - 0xC000] = value,
            0xD000..=0xDFFF => self.wram[address_usize - 0xD000 + 0x1000 * self.wram_bank] = value,
            0xE000..=0xEFFF => self.wram[address_usize - 0xE000] = value, // WRAM mirror
            0xF000..=0xFDFF => self.wram[address_usize - 0xF000 + 0x1000 * self.wram_bank] = value,
            0xFE00..=0xFE9F => self.ppu.write_oam((address_usize - 0xFE00) as u16, value),
            0xFEA0..=0xFEFF => {} // Unused OAM area
            0xFF00..=0xFF7F => match address {
                0xFF04 => {
                    if self.timer.reset_div() {
                        // Timer interrupt triggered by DIV reset
                        let current_if = self.io.read(0xFF0F);
                        self.io.write(0xFF0F, current_if | 0x04);
                    }
                },
                0xFF05 => self.timer.tima = value,
                0xFF06 => self.timer.tma = value,
                0xFF07 => {
                    let value = value & 0x07; // Only lower 3 bits are writable
                    let current_tac = self.timer.tac;
                    if (current_tac & 0x03) != (value & 0x03) {
                        // If frequency changed, reset the timer
                        self.timer.tima = self.timer.tma;
                    }
                    self.timer.tac = value;
                },
                0xFF46 => {
                    // DMA register - perform OAM DMA transfer
                    self.io.write(address, value);
                    self.perform_dma_transfer(value);
                }
                0xFF68..=0xFF69 => self.ppu.write_bg_palette_ram(address, value),
                0xFF6A..=0xFF6B => self.ppu.write_obj_palette_ram(address, value),
                _ => self.io.write(address, value),
            },
            0xFF80..=0xFFFE => self.hram[address_usize - 0xFF80] = value,
            0xFFFF => self.io.write(address, value),
        };
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        self.write_byte(address, (value & 0xFF) as u8);
        self.write_byte(address + 1, (value >> 8) as u8);
    }

    /// Performs DMA transfer to OAM
    /// Copies 160 bytes from source address (value * 0x100) to OAM (0xFE00-0xFE9F)
    fn perform_dma_transfer(&mut self, source_page: u8) {
        let source_address = (source_page as u16) << 8; // source_page * 0x100

        // Copy 160 bytes (OAM size) from source to OAM area
        for i in 0..160 {
            let source_addr = source_address + i;

            // Read from source address
            let byte = self.read_byte(source_addr);

            // Write to OAM through PPU
            self.ppu.write_oam(i, byte);
        }
    }
}
