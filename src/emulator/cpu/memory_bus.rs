mod dma;
mod hdma;

use crate::emulator::apu::Apu;
use crate::emulator::cart::Cart;
use crate::emulator::joypad::Joypad;
use crate::emulator::ppu::*;
use crate::emulator::serial::Serial;
use crate::emulator::timer::Timer;
use crate::utils::bits::*;
use dma::Dma;
use hdma::Hdma;
use serde::{Deserialize, Serialize};

const HRAM_SIZE: usize = 0x7F;
const WRAM_SIZE: usize = 0x8000;

pub enum InterruptBit {
    VBlank = 0x01,
    LCDStat = 0x02,
    Timer = 0x04,
    Serial = 0x08,
    Joypad = 0x10,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct MemoryBus {
    pub cart: Cart,
    pub ppu: PPU,
    pub apu: Apu,
    pub timer: Timer,
    pub joypad: Joypad,
    pub serial: Serial,
    #[serde(with = "serde_arrays")]
    hram: [u8; HRAM_SIZE],
    #[serde(with = "serde_arrays")]
    wram: [u8; WRAM_SIZE],
    wram_bank: usize,
    pub dma: Dma,
    hdma: Hdma,
    key1: u8,             // CGB speed switch register
    interrupt_flag: u8,   // IF register (0xFF0F)
    interrupt_enable: u8, // IE register (0xFFFF)
}

impl MemoryBus {
    pub fn new(cart: Cart, force_dmg: bool) -> Self {
        let cgb_mode = cart.is_cgb() && !force_dmg;
        let mut ppu = PPU::new();
        ppu.set_cgb_mode(cgb_mode);

        Self {
            cart,
            ppu,
            apu: Apu::new(),
            timer: Timer::new(),
            joypad: Joypad::new(),
            serial: Serial::new(true),
            hram: [0; HRAM_SIZE],
            wram: [0; WRAM_SIZE],
            wram_bank: 1,
            dma: Dma::new(),
            hdma: Hdma::new(),
            key1: 0, // Start in normal speed mode
            interrupt_flag: 0,
            interrupt_enable: 0,
        }
    }

    /// Returns a byte from the `address`.
    pub fn read_byte(&self, address: u16) -> u8 {
        let address_usize = address as usize;
        match address {
            0x0000..=0x7FFF => self.cart.read_rom(address),
            0x8000..=0x9FFF => self.ppu.read_vram(address),
            0xA000..=0xBFFF => self.cart.read_ram(address),
            0xC000..=0xCFFF => self.wram[address_usize - 0xC000], // WRAM bank 0
            0xD000..=0xDFFF => self.wram[address_usize - 0xD000 + 0x1000 * self.wram_bank], // WRAM bank n
            0xE000..=0xEFFF => self.wram[address_usize - 0xE000], // WRAM bank 0 mirror
            0xF000..=0xFDFF => self.wram[address_usize - 0xF000 + 0x1000 * self.wram_bank], // WRAM bank n mirror
            0xFE00..=0xFE9F => {
                if !self.ppu.is_lcd_enabled() {
                    // When LCD is disabled, OAM is always accessible
                    self.ppu.read_oam(address)
                } else if self.dma.is_oam_blocked() {
                    // During DMA with LCD on, cpu reads always return 0xFF
                    0xFF
                } else {
                    // Normal OAM access
                    let value = self.ppu.read_oam(address);
                    value
                }
            }
            0xFEA0..=0xFEFF => 0x00, // Unused OAM area
            0xFF00..=0xFF7F => match address {
                0xFF00 => self.joypad.read_p1(),
                0xFF01 => self.serial.read_sb(),
                0xFF02 => self.serial.read_sc(),
                0xFF04..=0xFF07 => self.timer.read(address),
                0xFF0F => self.interrupt_flag | 0xE0,
                0xFF10..=0xFF3F => self.apu.read(address),
                0xFF40..=0xFF45 => self.ppu.read_register(address),
                0xFF46 => self.dma.read(),
                0xFF47..=0xFF4B => self.ppu.read_register(address),
                0xFF4F => self.ppu.read_vram_bank(),
                0xFF51..=0xFF55 => self.hdma.read(address),
                0xFF4D => {
                    if self.ppu.cgb_mode {
                        self.key1 | 0x7E // Bits 1-6 are always 1, bit 7 is current speed
                    } else {
                        0xFF
                    }
                }
                0xFF68 => self.ppu.read_bg_palette_index(),
                0xFF69 => self.ppu.read_bg_palette_data(),
                0xFF6A => self.ppu.read_obj_palette_index(),
                0xFF6B => self.ppu.read_obj_palette_data(),
                0xFF70 => {
                    if self.ppu.cgb_mode {
                        self.wram_bank as u8
                    } else {
                        0xFF
                    }
                }
                _ => 0xFF,
            },
            0xFF80..=0xFFFE => self.hram[address_usize - 0xFF80],
            0xFFFF => self.interrupt_enable,
        }
    }

    pub fn request_interrupt(&mut self, interrupt: InterruptBit) {
        self.interrupt_flag |= interrupt as u8;
    }

    /// Writes a byte of `value` to the `address`.
    pub fn write_byte(&mut self, address: u16, value: u8) {
        let address_usize = address as usize;
        match address {
            0x0000..=0x7FFF => self.cart.write_rom(address, value),
            0x8000..=0x9FFF => self.ppu.write_vram(address, value),
            0xA000..=0xBFFF => self.cart.write_ram(address, value),
            0xC000..=0xCFFF => self.wram[address_usize - 0xC000] = value, // WRAM bank 0
            0xD000..=0xDFFF => self.wram[address_usize - 0xD000 + 0x1000 * self.wram_bank] = value, // WRAM bank n
            0xE000..=0xEFFF => self.wram[address_usize - 0xE000] = value, // WRAM bank 0 mirror
            0xF000..=0xFDFF => self.wram[address_usize - 0xF000 + 0x1000 * self.wram_bank] = value, // WRAM bank n mirror
            0xFE00..=0xFE9F => {
                if !self.ppu.is_lcd_enabled() {
                    // When LCD is disabled, OAM is always accessible
                    self.ppu.write_oam(address, value);
                } else if self.ppu.can_use_oam() && !self.dma.is_oam_blocked() {
                    // LCD is on, normal OAM access (with restrictions)
                    self.ppu.write_oam(address, value);
                }
            }
            0xFEA0..=0xFEFF => {} // Unused OAM area
            0xFF00..=0xFF7F => match address {
                0xFF00 => self.joypad.write_p1(value),
                0xFF01 => self.serial.write_sb(value),
                0xFF02 => self.serial.write_sc(value),
                0xFF04..=0xFF07 => self.timer.write(address, value, &mut self.interrupt_flag),
                0xFF0F => self.interrupt_flag = value & 0x1F,
                0xFF10..=0xFF3F => self.apu.write(address, value),
                0xFF40 => {
                    let lcdc = self.ppu.read_register(address);
                    let lcd_was_on = lcdc & BIT_7 != 0;

                    self.ppu.write_register(address, value);
                    let lcd_is_on = value & BIT_7 != 0;

                    if lcd_was_on && !lcd_is_on {
                        self.ppu.mode = PPUMode::HBlank;
                        self.ppu.dot_counter = 0;
                        self.ppu.force_write_register(0xFF44, 0); // LY = 0
                    } else if !lcd_was_on && lcd_is_on {
                        self.ppu.mode = PPUMode::OAMSearch;
                        self.ppu.dot_counter = 0;
                        self.ppu.force_write_register(0xFF44, 0); // LY = 0
                        self.ppu.check_lyc();
                        self.ppu.check_stat_interrupts(&mut self.interrupt_flag);
                    }
                }
                0xFF41 => {
                    // Undocumented GameBoy bug, needed by Road Rash
                    // http://www.devrs.com/gb/files/faqs.html#GBBugs
                    if self.ppu.is_lcd_enabled()
                        && (self.ppu.mode == PPUMode::VBlank || self.ppu.mode == PPUMode::HBlank)
                    {
                        self.request_interrupt(InterruptBit::LCDStat);
                    }
                    self.ppu.write_register(address, value);
                }
                0xFF42..=0xFF44 => self.ppu.write_register(address, value),
                0xFF45 => {
                    self.ppu.write_register(address, value);
                    let lcdc = self.ppu.read_register(0xFF40);
                    let lcd_is_on = lcdc & BIT_7 != 0;
                    if lcd_is_on {
                        self.ppu.check_lyc();
                        self.ppu.check_stat_interrupts(&mut self.interrupt_flag);
                    }
                }
                0xFF46 => self.dma.start(value),
                0xFF47..=0xFF4B => self.ppu.write_register(address, value),
                0xFF4D => {
                    if self.ppu.cgb_mode {
                        // Only bit 0 is writable (speed switch prepare)
                        self.key1 = (self.key1 & 0x80) | (value & 0x01);
                    }
                }
                0xFF4F => self.ppu.write_vram_bank(value),
                0xFF51..=0xFF55 => {
                    self.hdma.write(address, value);
                    // If General Purpose DMA (bit 7 = 0), transfer immediately
                    if !self.hdma.is_h_blank_mode() {
                        self.perform_gdma();
                    }
                }
                0xFF68 => self.ppu.write_bg_palette_index(value),
                0xFF69 => self.ppu.write_bg_palette_data(value),
                0xFF6A => self.ppu.write_obj_palette_index(value),
                0xFF6B => self.ppu.write_obj_palette_data(value),
                0xFF70 => {
                    if self.ppu.cgb_mode {
                        self.wram_bank = match value & 0x07 {
                            0 => 1, // Bank 0 is not selectable, defaults to 1
                            n => n as usize,
                        };
                    }
                }
                _ => {}
            },
            0xFF80..=0xFFFE => self.hram[address_usize - 0xFF80] = value,
            0xFFFF => self.interrupt_enable = value,
        };
    }

    pub fn tick(&mut self) {
        let previous_mode = self.ppu.mode;

        self.dma_tick();

        if self.timer.tick() {
            self.request_interrupt(InterruptBit::Timer);
        }

        self.ppu.tick(&mut self.interrupt_flag);
        self.apu.tick();

        if self.serial.tick() {
            self.request_interrupt(InterruptBit::Serial);
        }

        // Trigger HBlank DMA if we just entered HBlank
        if previous_mode != PPUMode::HBlank && self.ppu.mode == PPUMode::HBlank {
            self.hdma_hblank_transfer();
        }
    }

    /// Advances the OAM DMA transfer if active
    pub fn dma_tick(&mut self) {
        if let Some((source, destination)) = self.dma.tick() {
            // Perform the DMA transfer
            let value = self.read_byte(source);
            self.ppu.write_oam(destination, value);
        }
    }

    /// Read a byte for HDMA transfer (avoids full read_byte overhead)
    fn read_for_hdma(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.cart.read_rom(address),
            0xA000..=0xBFFF => self.cart.read_ram(address),
            0xC000..=0xCFFF => self.wram[(address - 0xC000) as usize],
            0xD000..=0xDFFF => {
                let addr = (address - 0xD000) as usize;
                self.wram[addr + 0x1000 * self.wram_bank]
            }
            _ => 0xFF,
        }
    }

    /// Perform General Purpose DMA (immediate transfer)
    fn perform_gdma(&mut self) {
        while self.hdma.is_active() {
            if let Some((source, dest)) = self.hdma.transfer_block() {
                // Transfer 16 bytes
                for i in 0..16 {
                    let value = self.read_for_hdma(source.wrapping_add(i));
                    self.ppu.write_vram(dest.wrapping_add(i), value);
                }
            }
        }
    }

    /// Perform one block of HBlank DMA (called during HBlank)
    pub fn hdma_hblank_transfer(&mut self) {
        if !self.hdma.is_active() || !self.hdma.is_h_blank_mode() {
            return;
        }

        if let Some((source, dest)) = self.hdma.transfer_block() {
            // Transfer 16 bytes
            for i in 0..16 {
                let value = self.read_for_hdma(source.wrapping_add(i));
                self.ppu.write_vram(dest.wrapping_add(i), value);
            }
        }
    }

    /// Check if speed switch is prepared (KEY1 bit 0 is set)
    pub fn is_speed_switch_prepared(&self) -> bool {
        self.ppu.cgb_mode && (self.key1 & 0x01) != 0
    }

    /// Perform the speed switch (called after STOP instruction)
    pub fn perform_speed_switch(&mut self) {
        if !self.ppu.cgb_mode || (self.key1 & 0x01) == 0 {
            return;
        }

        // Toggle speed (bit 7): 0 = normal, 1 = double
        self.key1 ^= 0x80;
        // Clear the prepare bit (bit 0)
        self.key1 &= !0x01;
    }

    /// Set the state of a joypad button
    pub fn set_button_state(
        &mut self,
        button: crate::emulator::joypad::JoypadButton,
        pressed: bool,
    ) {
        if self.joypad.set_button_state(button, pressed) {
            // Request joypad interrupt if button was pressed
            self.request_interrupt(InterruptBit::Joypad);
        }
    }
}
