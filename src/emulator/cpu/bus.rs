mod regions;

use core::panic;
use crate::emulator::ppu::*;
use regions::*;

const MEMORY_SIZE: usize = 0x10000;

pub struct MemoryBus {
    memory: [u8; MEMORY_SIZE],
    ppu: PPU,
}

impl MemoryBus {
    pub fn new(cart_data: &Vec<u8>) -> Self {
        let mut memory = [0; MEMORY_SIZE];
        for i in 0..cart_data.len() {
            memory[i] = cart_data[i];
        }
        Self {
            memory,
            ppu: PPU::new(),
        }
    }

    /// Returns a byte from the `address`.
    pub fn read_byte(&self, address: u16) -> u8 {
        use MemoryRegion::*;
        let address = address as usize;
        match MemoryRegion::from_address(address) {
            ROM => self.memory[address],
            VRAM => self.ppu.read_vram(address),
            CRAM => self.memory[address],
            WRAM => todo!("Unsupported bus read: {:#04X} ({})", address, "WRAM"),
            ECHO => 0,
            OAM => todo!("Unsupported bus read: {:#04X} ({})", address, "OAM"),
            RESERVED => 0,
            IO => todo!("Unsupported bus read: {:#04X} ({})", address, "IO Regs"),
            HRAM => todo!("Unsupported bus read: {:#04X} ({})", address, "HRAM"),
            IER => todo!("Unsupported bus read: {:#04X} ({})", address, "IE"),
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
        use MemoryRegion::*;
        let address = address as usize;
        match MemoryRegion::from_address(address) {
            ROM => self.memory[address] = value,
            VRAM => self.ppu.write_vram(address, value),
            CRAM => self.memory[address] = value,
            WRAM => todo!("Unsupported bus write: {:#04X} ({})", address, "WRAM"),
            ECHO => (),
            OAM => todo!("Unsupported bus write: {:#04X} ({})", address, "OAM"),
            RESERVED => (),
            IO => todo!("Unsupported bus write: {:#04X} ({})", address, "IO Regs"),
            HRAM => todo!("Unsupported bus write: {:#04X} ({})", address, "HRAM"),
            IER => todo!("Unsupported bus write: {:#04X} ({})", address, "IE"),
        };
    }
}
