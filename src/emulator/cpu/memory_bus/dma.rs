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
    oam_blocked: bool,
}

impl Dma {
    pub fn new() -> Self {
        Self {
            source: 0,
            copied: 0,
            enabled: false,
            delay: 0,
            last_value: 0xFF,
            oam_blocked: false,
        }
    }

    pub fn is_oam_blocked(&self) -> bool {
        self.oam_blocked
    }

    pub fn read(&self) -> u8 {
        self.last_value
    }

    pub fn start(&mut self, source: u8) {
        self.last_value = source;
        self.source = (source as u16) << 8;
        self.copied = 0;
        // If DMA was already running (and OAM was already blocked), keep it blocked
        // Otherwise, OAM is not blocked yet for the initial 8 T-cycles
        if !self.enabled {
            self.oam_blocked = false;
        }
        self.enabled = true;
        self.delay = DELAY_T_CYCLES * 2; // 8 T-cycles initial delay before first transfer
    }

    pub fn tick(&mut self) -> Option<(u16, u16)> {
        if !self.enabled {
            return None;
        }

        if self.delay > 0 {
            self.delay -= 1;
            // OAM becomes blocked at M=2 (when delay reaches 0, meaning 8 T-cycles have passed)
            if self.delay == 0 {
                self.oam_blocked = true;
            }
        }

        if self.delay == 0 {
            if self.copied >= TRANSFER_SIZE {
                self.enabled = false;
                self.oam_blocked = false;
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
