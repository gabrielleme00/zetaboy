use serde::{Deserialize, Serialize};

// Serial I/O Registers:
// 0xFF01 (SB): Serial transfer data
// 0xFF02 (SC): Serial transfer control

#[derive(Clone, Deserialize, Serialize)]
pub struct Serial {
    sb: u8, // Serial transfer data (0xFF01)
    sc: u8, // Serial transfer control (0xFF02)
    print_serial: bool,
}

impl Serial {
    pub fn new(print_serial: bool) -> Self {
        Self {
            sb: 0,
            sc: 0,
            print_serial,
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
        self.sc = value;
        
        // Check if transfer is starting (bit 7 set and bit 0 set for internal clock)
        if value & 0x81 == 0x81 {
            // Print the character from SB register and mark transfer as complete
            if self.print_serial {
                print!("{}", self.sb as char);
            }
            // Clear bit 7 to indicate transfer complete
            self.sc &= 0x7F;
        }
    }
}
