/// Game Boy Timer implementation
/// 
/// The Game Boy has several timer-related registers:
/// - DIV (0xFF04): Divider Register - increments at 16384 Hz
/// - TIMA (0xFF05): Timer Counter - increments at rate specified by TAC
/// - TMA (0xFF06): Timer Modulo - loaded into TIMA when it overflows
/// - TAC (0xFF07): Timer Control - enables timer and sets frequency
pub struct Timer {
    /// Internal 16-bit divider counter (only upper 8 bits visible as DIV register)
    div_counter: u16,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div_counter: 0,
        }
    }

    /// Steps the timer forward by the given number of CPU cycles
    /// Returns true if a timer interrupt should be triggered
    pub fn step(&mut self, cycles: u8, div: &mut u8, tima: &mut u8, tma: u8, tac: u8) -> bool {
        let mut interrupt = false;
        
        // Update DIV register (increments every 256 CPU cycles = 16384 Hz)
        let old_div_counter = self.div_counter;
        self.div_counter = self.div_counter.wrapping_add(cycles as u16);
        *div = (self.div_counter >> 8) as u8; // DIV is upper 8 bits of 16-bit counter
        
        // Check if timer is enabled (bit 2 of TAC)
        if tac & 0x04 != 0 {
            // Determine timer frequency based on TAC bits 0-1
            let frequency_bit = match tac & 0x03 {
                0 => 9,  // 4096 Hz (CPU_FREQ / 1024)
                1 => 3,  // 262144 Hz (CPU_FREQ / 16) 
                2 => 5,  // 65536 Hz (CPU_FREQ / 64)
                3 => 7,  // 16384 Hz (CPU_FREQ / 256)
                _ => unreachable!(),
            };
            
            // Check if the relevant bit transitioned from 1 to 0 (falling edge detection)
            // This is how the real Game Boy timer works
            let old_bit = (old_div_counter >> frequency_bit) & 1;
            let new_bit = (self.div_counter >> frequency_bit) & 1;
            
            if old_bit == 1 && new_bit == 0 {
                // Increment TIMA
                if *tima == 0xFF {
                    // TIMA overflow - reset to TMA and request interrupt
                    *tima = tma;
                    interrupt = true;
                } else {
                    *tima = tima.wrapping_add(1);
                }
            }
        }
        
        interrupt
    }
    
    /// Resets the DIV register (writing any value to DIV resets it to 0)
    pub fn reset_div(&mut self, div: &mut u8) {
        self.div_counter = 0;
        *div = 0;
    }
}
