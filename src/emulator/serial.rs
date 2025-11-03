use serde::{Deserialize, Serialize};

// Serial transfer takes 8192 cycles (512 cycles per bit * 8 bits)
// at 4.194304 MHz clock speed
const SERIAL_TRANSFER_CYCLES: u16 = 512;

#[derive(Clone, Deserialize, Serialize)]
pub struct Serial {
    sb: u8,                // Serial transfer data (0xFF01)
    sc: u8,                // Serial transfer control (0xFF02)
    print_serial: bool,    // Whether to print serial output to console
    transfer_counter: u16, // Counter for serial transfer timing
    bits_transferred: u8,  // Number of bits transferred (0-8)
}

impl Serial {
    pub fn new(print_serial: bool) -> Self {
        Self {
            sb: 0,
            sc: 0,
            print_serial,
            transfer_counter: 0,
            bits_transferred: 0,
        }
    }

    pub fn read_sb(&self) -> u8 {
        self.sb
    }

    pub fn read_sc(&self) -> u8 {
        self.sc
    }

    pub fn write_sb(&mut self, value: u8) {
        self.sb = value;
    }

    pub fn write_sc(&mut self, value: u8) {
        let was_transferring = self.sc & 0x80 != 0;
        self.sc = value;

        // Check if transfer is starting (bit 7 set and bit 0 set for internal clock)
        if !was_transferring && value & 0x81 == 0x81 {
            // Start a new transfer
            self.transfer_counter = 0;
            self.bits_transferred = 0;
        } else if was_transferring && value & 0x80 == 0 {
            // Transfer was cancelled
            self.transfer_counter = 0;
            self.bits_transferred = 0;
        }
    }

    /// Tick the serial transfer. Returns true if transfer completed and interrupt should be requested.
    pub fn tick(&mut self) -> bool {
        // Check if transfer is active (bit 7) and using internal clock (bit 0)
        if self.sc & 0x81 != 0x81 {
            return false;
        }

        self.transfer_counter += 1;

        // Each bit takes 512 cycles to transfer
        if self.transfer_counter >= SERIAL_TRANSFER_CYCLES {
            self.transfer_counter = 0;
            self.bits_transferred += 1;

            // After 8 bits, transfer is complete
            if self.bits_transferred >= 8 {
                // Print the character if enabled
                if self.print_serial {
                    print!("{}", self.sb as char);
                }

                // Transfer complete: clear bit 7 and reset counters
                self.sc &= 0x7F;
                self.bits_transferred = 0;
                self.transfer_counter = 0;

                // When no external device is connected, shift in 0xFF
                self.sb = 0xFF;

                // Request serial interrupt
                return true;
            }
        }

        false
    }
}
