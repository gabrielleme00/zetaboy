use serde::{Deserialize, Serialize};

const TRANSFER_SIZE: u16 = 0xA0;
const DELAY_T_CYCLES: u8 = 4;
const DESTINATION_START: u16 = 0xFE00;

#[derive(Clone, Deserialize, Serialize)]
pub struct Dma {
    source: u16,
    copied: u16,
    enabled: bool,
    delay: u8,
    last_value: u8,
}

impl Dma {
    pub fn new() -> Self {
        Self {
            source: 0,
            copied: 0,
            enabled: false,
            delay: 0,
            last_value: 0xFF,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn read(&self) -> u8 {
        self.last_value
    }

    pub fn start(&mut self, source: u8) {
        self.last_value = source;
        self.source = (source as u16) << 8;
        self.copied = 0;
        self.enabled = true;
        self.delay = DELAY_T_CYCLES * 2; // Initial delay before first transfer
    }

    pub fn tick(&mut self) -> Option<(u16, u16)> {
        if !self.enabled {
            return None;
        }

        if self.delay > 0 {
            self.delay -= 1;
        }

        if self.delay == 0 {
            if self.copied >= TRANSFER_SIZE {
                self.enabled = false;
                return None;
            }

            // time to transfer a byte
            let source = self.source + self.copied;
            let destination = DESTINATION_START + self.copied;
            let transfer_request = (source, destination);

            self.copied += 1;
            self.delay = DELAY_T_CYCLES;

            return Some(transfer_request);
        }

        None
    }
}
