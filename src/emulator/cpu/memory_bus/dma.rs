use serde::{Deserialize, Serialize};

const TRANSFER_SIZE: u16 = 160; // 160 bytes per OAM DMA transfer
const DELAY_T_CYCLES: u8 = 4; // 4 T-cycles delay between transfers
const DESTINATION_START: u16 = 0xFE00; // OAM start address

type TransferRequest = (u16, u16); // (source_address, destination_address)

/// Direct Memory Access (DMA) controller for OAM transfers.
#[derive(Clone, Deserialize, Serialize)]
pub struct Dma {
    source: u16,       // Source address high byte (shifted left by 8)
    copied: u16,       // Number of bytes copied so far
    enabled: bool,     // Whether a DMA transfer is currently active
    delay: u8,         // Delay in T-cycles before the next byte transfer
    last_value: u8,    // Last value written to DMA register
    oam_blocked: bool, // Whether OAM is currently blocked due to DMA
}

impl Dma {
    /// Creates a new DMA controller instance.
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

    /// Returns whether OAM is currently blocked due to an ongoing DMA transfer.
    pub fn is_oam_blocked(&self) -> bool {
        self.oam_blocked
    }

    /// Reads the last written DMA value.
    pub fn read(&self) -> u8 {
        self.last_value
    }

    /// Starts a DMA transfer from the specified source high byte.
    pub fn start(&mut self, source: u8) {
        // Setup DMA state
        self.last_value = source;
        self.source = (source as u16) << 8;
        self.copied = 0;

        // If DMA was already running (and OAM was already blocked), keep it blocked
        // Otherwise, OAM is not blocked yet for the initial 8 T-cycles
        if !self.enabled {
            self.oam_blocked = false;
        }

        self.enabled = true;
        self.delay = DELAY_T_CYCLES * 2; // Must wait 8 T-cycles (2 M-cycles) before first transfer!
    }

    /// Advances the DMA by one T-cycle.
    pub fn tick(&mut self) -> Option<TransferRequest> {
        if !self.enabled {
            return None;
        }

        // Handle delay before transfers start
        if self.delay > 0 {
            self.delay -= 1;
            if self.delay == 0 {
                self.oam_blocked = true;
            }
        }

        // If no delay, perform transfer
        if self.delay == 0 {
            // Check if transfer is complete
            if self.copied >= TRANSFER_SIZE {
                self.enabled = false;
                self.oam_blocked = false;
                return None;
            }

            // Generate transfer request
            let transfer_request = self.generate_transfer_request();

            // Update state for next byte
            self.copied += 1;
            self.delay = DELAY_T_CYCLES;

            return Some(transfer_request);
        }

        None
    }

    fn generate_transfer_request(&self) -> TransferRequest {
        let source = self.source + self.copied;
        let destination = DESTINATION_START + self.copied;
        (source, destination)
    }
}
