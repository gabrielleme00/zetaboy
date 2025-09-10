mod dma;
pub mod io_registers;

use crate::emulator::cart::Cart;
use crate::emulator::ppu::*;
use crate::emulator::timer::Timer;
use crate::utils::bits::BIT_7;
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
    pub dma: Dma,
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
            0xFE00..=0xFE9F => {
                let lcdc = self.io.read(REG_LCDC);
                let lcd_enabled = (lcdc & BIT_7) != 0;

                if !lcd_enabled {
                    // When LCD is disabled, OAM is always accessible
                    self.ppu.read_oam(address)
                } else if self.dma.is_enabled() {
                    // During DMA with LCD on, cpu reads always return 0xFF
                    println!(
                        "Ignored read from OAM at {:#06X} during DMA",
                        address
                    );
                    0xFF
                } else {
                    // Normal OAM access
                    let value = self.ppu.read_oam(address);
                    println!("Read {:#04X} from OAM at {:#06X}", value, address);
                    value
                }
            }
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
            0xFE00..=0xFE9F => {
                let lcdc = self.io.read(REG_LCDC);
                let lcd_enabled = (lcdc & BIT_7) != 0;

                if !lcd_enabled {
                    // When LCD is disabled, OAM is always accessible
                    self.ppu.write_oam(address, value);
                } else if self.ppu.can_use_oam() && !self.dma.is_enabled() {
                    // LCD is on, normal OAM access (with restrictions)
                    self.ppu.write_oam(address, value);
                    println!("Wrote {:#04X} to OAM at {:#06X}", value, address);
                } else {
                    println!(
                        "Ignored write to OAM at {:#06X} while in mode {:?} or during DMA",
                        address, self.ppu.mode
                    );
                }
            }
            0xFEA0..=0xFEFF => {} // Unused OAM area
            0xFF00..=0xFF7F => match address {
                0xFF04..=0xFF07 => self.timer.write(address, value, &mut self.io),
                0xFF40 => {
                    let lcdc = self.io.read(address);
                    let lcd_was_on = lcdc & BIT_7 != 0;

                    self.io.write(address, value);
                    let lcd_is_on = value & BIT_7 != 0;

                    if lcd_was_on && !lcd_is_on {
                        self.ppu.mode = PPUMode::HBlank;
                        self.ppu.dot_counter = 0;
                        self.io.write(REG_LY, 0);
                    } else if !lcd_was_on && lcd_is_on {
                        self.ppu.mode = PPUMode::OAMSearch;
                        self.ppu.dot_counter = 0;
                        self.io.write(REG_LY, 0);
                        self.ppu.check_lyc(&mut self.io);
                        self.ppu.check_stat_interrupts(&mut self.io);
                    }
                }
                0xFF41 => {
                    // Undocumented GameBoy bug, needed by Road Rash
                    // http://www.devrs.com/gb/files/faqs.html#GBBugs
                    if self.ppu.mode == PPUMode::VBlank || self.ppu.mode == PPUMode::HBlank {
                        let lcd_enabled = self.io.read(REG_LCDC) & BIT_7 != 0;
                        if lcd_enabled {
                            self.io.request_interrupt(InterruptBit::LCDStat);
                        }
                    }
                    self.io.write(address, value);
                }
                0xFF45 => {
                    self.io.write(address, value);
                    let lcd_is_on = value & BIT_7 != 0;
                    if lcd_is_on {
                        self.ppu.check_lyc(&mut self.io);
                        self.ppu.check_stat_interrupts(&mut self.io);
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
            let value = self.read_byte(source);
            self.ppu.write_oam(destination, value);
        }
    }
}
