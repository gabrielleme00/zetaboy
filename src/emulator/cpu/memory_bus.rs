mod dma;
pub mod io_registers;

use crate::emulator::cart::Cart;
use crate::emulator::ppu::*;
use crate::emulator::timer::Timer;
use dma::Dma;
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
    dma: Dma,
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
            dma: Dma::new(),
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
            0xFE00..=0xFE9F => self.ppu.read_oam(address),
            0xFEA0..=0xFEFF => 0x00,
            0xFF00..=0xFF7F => match address {
                0xFF04..=0xFF07 => self.timer.read(address),
                // 0xFF44 => 0x90, // Only return 0x90 for logging purposes
                // 0xFF68..=0xFF69 => self.ppu.read_bg_palette_ram(address), // CGB
                // 0xFF6A..=0xFF6B => self.ppu.read_obj_palette_ram(address), // CGB
                _ => self.io.read(address),
            },
            0xFF80..=0xFFFE => self.hram[address_usize - 0xFF80],
            0xFFFF => self.io.read(address),
        }
    }

    pub fn get_interrupt_flags(&self) -> u8 {
        self.io.read(REG_IF)
    }

    pub fn get_interrupt_enable(&self) -> u8 {
        self.io.read(REG_IE)
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        if self.dma.is_enabled() {
            // During DMA, only OAM area is blocked for CPU access
            // MBC registers and other areas should still be accessible
            if (0xFE00..=0xFE9F).contains(&address) {
                return;
            }
            // Also block access to most memory areas except HRAM and MBC registers
            match address {
                0x0000..=0x7FFF => {} // Allow MBC register writes
                0xFF80..=0xFFFF => {} // Allow HRAM and IO
                _ => return,          // Block everything else
            }
        }

        // Perform the write operation
        self.perform_write_byte(address, value);
    }

    /// Writes a byte of `value` to the `address`.
    fn perform_write_byte(&mut self, address: u16, value: u8) {
        let address_usize = address as usize;
        match address {
            0x0000..=0x7FFF => self.cart.write_rom(address, value),
            0x8000..=0x9FFF => self.ppu.write_vram(address, value),
            0xA000..=0xBFFF => self.cart.write_ram(address, value),
            0xC000..=0xCFFF => self.wram[address_usize - 0xC000] = value,
            0xD000..=0xDFFF => self.wram[address_usize - 0xD000 + 0x1000 * self.wram_bank] = value,
            0xE000..=0xEFFF => self.wram[address_usize - 0xE000] = value, // WRAM mirror
            0xF000..=0xFDFF => self.wram[address_usize - 0xF000 + 0x1000 * self.wram_bank] = value,
            0xFE00..=0xFE9F => self.ppu.write_oam(address, value),
            0xFEA0..=0xFEFF => {} // Unused OAM area
            0xFF00..=0xFF7F => match address {
                0xFF04..=0xFF07 => self.timer.write(address, value, &mut self.io),
                0xFF40 => {
                    let lcdc = self.io.read(0xFF40);
                    if lcdc & 0x80 != 0 && value & 0x80 == 0 {
                        // Disabling bit 7 (LCD & PPY enable) can only happen
                        // during V-Blank
                        if self.ppu.is_vblank() {
                            self.ppu.mode = PPUMode::HBlank;
                        } else {
                            // Write to the LCDC register without changing the mode
                            self.io.write(0xFF40, value & 0x7F);
                        }
                    } else {
                        self.io.write(0xFF40, value);
                    }
                }
                0xFF46 => {
                    // DMA register - perform OAM DMA transfer
                    self.io.write(address, value);
                    self.dma.start(value);
                }
                // 0xFF68..=0xFF69 => self.ppu.write_bg_palette_ram(address, value),
                // 0xFF6A..=0xFF6B => self.ppu.write_obj_palette_ram(address, value),
                _ => self.io.write(address, value),
            },
            0xFF80..=0xFFFE => self.hram[address_usize - 0xFF80] = value,
            0xFFFF => self.io.write(address, value),
        };
    }

    pub fn tick(&mut self) {
        self.dma_tick();
        self.timer.tick(&mut self.io);
        // TODO: Serial tick
        self.ppu.tick(&mut self.io);
        // TODO: APU tick
    }

    pub fn dma_tick(&mut self) {
        if let Some((source, destination)) = self.dma.tick() {
            // Perform the DMA transfer
            let byte = self.read_byte(source);
            self.perform_write_byte(destination, byte);
        }
    }
}
