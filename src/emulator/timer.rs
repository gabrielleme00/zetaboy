/// Game Boy Timer implementation
///
/// The Game Boy has several timer-related registers:
/// - DIV (0xFF04): Divider Register - increments at 16384 Hz
/// - TIMA (0xFF05): Timer Counter - increments at rate specified by TAC
/// - TMA (0xFF06): Timer Modulo - loaded into TIMA when it overflows
/// - TAC (0xFF07): Timer Control - enables timer and sets frequency
pub struct Timer {
    /// Internal 16-bit divider counter (only upper 8 bits visible as DIV register)
    pub div: u8, // 0xFF04
    pub tima: u8, // 0xFF05
    pub tma: u8,  // 0xFF06
    pub tac: u8,  // 0xFF07
    div_counter: u16,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            div_counter: 0,
        }
    }

    /// Read from timer registers.
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => self.div,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac | 0xF8,
            _ => 0,
        }
    }

    /// Write to timer registers.
    ///
    /// Returns true if timer interrupt was triggered.
    pub fn write(&mut self, addr: u16, value: u8) -> bool {
        let mut interrupt = false;
        match addr {
            0xFF04 => {
                if self.reset_div() {
                    // Timer interrupt triggered by DIV reset
                    interrupt = true;
                }
            }
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => {
                let value = value & 0x07; // Only lower 3 bits are writable
                let current_tac = self.tac;
                if (current_tac & 0x03) != (value & 0x03) {
                    // If frequency changed, reset the timer
                    self.tima = self.tma;
                }
                self.tac = value;
            }
            _ => {}
        }
        interrupt
    }

    /// Steps the timer forward by the given number of T-cycles.
    ///
    /// Returns true if a timer interrupt should be triggered.
    pub fn tick(&mut self) -> bool {
        let mut interrupt = false;

        // Store old counter state
        let old_counter = self.div_counter;

        // Update counter
        self.div_counter = self.div_counter.wrapping_add(1);

        // Update DIV register (upper 8 bits of internal 16-bit counter)
        self.div = (self.div_counter >> 8) as u8;

        // If TIMA is running
        if self.tac & 0x04 != 0 {
            // Get timer_bit based on TAC register
            let timer_bit = match self.tac & 0x03 {
                0 => 9, // Bit 9 (4096 Hz)
                1 => 3, // Bit 3 (262144 Hz)
                2 => 5, // Bit 5 (65536 Hz)
                3 => 7, // Bit 7 (16384 Hz)
                _ => unreachable!(),
            };

            // Check for falling edges by iterating from old to new counter value
            let mut current = old_counter;
            while current != self.div_counter {
                let next = current.wrapping_add(1);

                // Check if timer_bit transitions from 1 to 0 (falling edge)
                let old_bit = (current >> timer_bit) & 1;
                let new_bit = (next >> timer_bit) & 1;

                if old_bit == 1 && new_bit == 0 {
                    // TIMA should increment on falling edge
                    if self.tima == 0xFF {
                        self.tima = self.tma; // Reload from TMA
                        interrupt = true; // Trigger interrupt
                    } else {
                        self.tima = self.tima.wrapping_add(1);
                    }
                }

                current = next;
            }
        }

        interrupt
    }

    /// Resets the DIV register and internal cycle counter (called when DIV is written to)
    pub fn reset_div(&mut self) -> bool {
        let old_counter = self.div_counter;
        self.div_counter = 0;
        self.div = 0;

        // Check for DIV reset glitch: if timer is enabled and the selected bit was 1,
        // TIMA increments when DIV is reset
        if self.tac & 0x04 != 0 {
            let timer_bit = match self.tac & 0x03 {
                0 => 9, // Bit 9
                1 => 3, // Bit 3
                2 => 5, // Bit 5
                3 => 7, // Bit 7
                _ => unreachable!(),
            };

            // If the selected bit was 1, TIMA increments due to falling edge
            if (old_counter >> timer_bit) & 1 == 1 {
                if self.tima == 0xFF {
                    self.tima = self.tma;
                    return true; // Return true to indicate interrupt should be triggered
                } else {
                    self.tima = self.tima.wrapping_add(1);
                }
            }
        }

        false
    }
}
