use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Hdma {
    source_high: u8,      // HDMA1 - Source High
    source_low: u8,       // HDMA2 - Source Low
    dest_high: u8,        // HDMA3 - Destination High
    dest_low: u8,         // HDMA4 - Destination Low
    mode_length: u8,      // HDMA5 - Mode/Length
    active: bool,         // Is HDMA transfer active
    remaining_blocks: u8, // Blocks remaining in HBlank DMA
}

impl Hdma {
    pub fn new() -> Self {
        Self {
            source_high: 0xFF,
            source_low: 0xFF,
            dest_high: 0xFF,
            dest_low: 0xFF,
            mode_length: 0xFF,
            active: false,
            remaining_blocks: 0,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn write_source_high(&mut self, value: u8) {
        self.source_high = value;
    }

    pub fn read_source_high(&self) -> u8 {
        self.source_high
    }

    pub fn write_source_low(&mut self, value: u8) {
        // Lower 4 bits are ignored (aligned to 16 bytes)
        self.source_low = value & 0xF0;
    }

    pub fn read_source_low(&self) -> u8 {
        self.source_low
    }

    pub fn write_dest_high(&mut self, value: u8) {
        // Only bits 0-4 are used (VRAM is 0x8000-0x9FFF)
        self.dest_high = value & 0x1F;
    }

    pub fn read_dest_high(&self) -> u8 {
        self.dest_high
    }

    pub fn write_dest_low(&mut self, value: u8) {
        // Lower 4 bits are ignored (aligned to 16 bytes)
        self.dest_low = value & 0xF0;
    }

    pub fn read_dest_low(&self) -> u8 {
        self.dest_low
    }

    pub fn write_mode_length(&mut self, value: u8) {
        // Check if we should terminate an active HBlank DMA
        if self.active && self.is_h_blank_mode() && (value & 0x80) == 0 {
            // Writing with bit 7 = 0 while HBlank DMA is active terminates it
            self.active = false;
            self.mode_length = self.remaining_blocks.saturating_sub(1) | 0x80;
            return;
        }

        let length = (value & 0x7F) + 1; // Bits 0-6 = length - 1
        self.mode_length = value;
        self.remaining_blocks = length;
        self.active = true;
    }

    pub fn read_mode_length(&self) -> u8 {
        if self.active {
            // Return remaining length - 1, with bit 7 clear
            self.remaining_blocks.saturating_sub(1) & 0x7F
        } else {
            // Return 0xFF (bit 7 set = inactive)
            0xFF
        }
    }

    pub fn get_source(&self) -> u16 {
        ((self.source_high as u16) << 8) | (self.source_low as u16)
    }

    pub fn get_dest(&self) -> u16 {
        // Destination is in VRAM (0x8000-0x9FFF)
        0x8000 | ((self.dest_high as u16) << 8) | (self.dest_low as u16)
    }

    pub fn is_h_blank_mode(&self) -> bool {
        (self.mode_length & 0x80) != 0
    }

    /// Performs one block (16 bytes) of HDMA transfer
    /// Returns (source_addr, dest_addr) for the transfer
    pub fn transfer_block(&mut self) -> Option<(u16, u16)> {
        if !self.active || self.remaining_blocks == 0 {
            return None;
        }

        let source = self.get_source();
        let dest = self.get_dest();

        // Advance pointers by 16 bytes for next block
        let new_source = source.wrapping_add(16);
        self.source_high = (new_source >> 8) as u8;
        self.source_low = (new_source & 0xFF) as u8;

        let new_dest = dest.wrapping_add(16);
        self.dest_high = ((new_dest >> 8) & 0x1F) as u8;
        self.dest_low = (new_dest & 0xFF) as u8;

        self.remaining_blocks -= 1;

        if self.remaining_blocks == 0 {
            self.active = false;
        }

        Some((source, dest))
    }
}
