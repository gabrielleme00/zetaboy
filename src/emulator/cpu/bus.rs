mod gpu;

use core::panic;
use gpu::*;

const MEMORY_SIZE: usize = 0x10000;
const ROM_BANK_0_BEGIN: usize = 0x0000;
const ROM_BANK_0_END: usize = 0x7FFF;
const WRAM_BEGIN: usize = 0xC000;
const WRAM_END: usize = 0xDFFF;

pub struct MemoryBus {
    memory: [u8; MEMORY_SIZE],
    gpu: GPU,
}

impl MemoryBus {
    pub fn new(cart_data: &Vec<u8>) -> Self {
        let mut memory = [0; MEMORY_SIZE];
        for i in 0..cart_data.len() {
            memory[i] = cart_data[i];
        }
        Self {
            memory,
            gpu: GPU::new(),
        }
    }

    /// Returns a byte from the `address`.
    pub fn read_byte(&self, address: u16) -> u8 {
        let address = address as usize;
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_0_END => self.memory[address],
            VRAM_BEGIN..=VRAM_END => self.gpu.read_vram(address - VRAM_BEGIN),
            WRAM_BEGIN..=WRAM_END => self.memory[address],
            0xFF00..=0xFFFF => self.memory[address],
            _ => panic!("TODO: support other areas of memory"),
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
            VRAM_BEGIN..=VRAM_END => self.gpu.write_vram(address - VRAM_BEGIN, value),
            WRAM_BEGIN..=WRAM_END => self.memory[address] = value,
            0xFF00..=0xFFFF => self.memory[address] = value,
            _ => {
                println!("Write: {:#04X} at {:#04X}", value, address);
                panic!("TODO: support other areas of memory")
            },
        };
    }
}
