const TRANSFER_SIZE: u16 = 0xA0;
const DELAY_T_CYCLES: u8 = 1;
const DESTINATION_START: u16 = 0xFE00;

pub struct Dma {
    source: u16,
    copied: u16,
    enabled: bool,
    delay: u8,
}

impl Dma {
    pub fn new() -> Self {
        Self {
            source: 0,
            copied: 0,
            enabled: false,
            delay: 0,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn start(&mut self, source: u8) {
        self.source = (source as u16) << 8;
        self.copied = 0;
        self.enabled = true;
        self.delay = DELAY_T_CYCLES; // 4 T-cycles delay before each transfer
    }

    pub fn tick(&mut self) -> Option<(u16, u16)> {
        if !self.enabled {
            return None;
        }

        self.delay -= 1;

        if self.delay == 0 {
            // Reset delay
            self.delay = DELAY_T_CYCLES; // Reset delay for next byte

            // Build transfer request
            let source = self.source + self.copied;
            let destination = DESTINATION_START + self.copied;
            let transfer_request = (source, destination);

            // Increment copied bytes and check if transfer is complete
            self.copied += 1;
            if self.copied >= TRANSFER_SIZE {
                self.enabled = false;
            }

            return Some(transfer_request);
        }

        None
    }
}
