use crate::{
    emulator::cpu::memory_bus::InterruptBit,
    utils::bits::*,
};
use serde::{Deserialize, Serialize};

const BITS: [u8; 4] = [9, 3, 5, 7];
const OVERFLOW_INTERRUPT_TICK: u8 = 4;
const OVERFLOW_RELOAD_TICK: u8 = 5;
const OVERFLOW_RESET_TICK: u8 = 6;

#[derive(Clone, Deserialize, Serialize)]
pub struct Timer {
    pub div: u16, // 0xFF04
    pub tima: u8, // 0xFF05
    pub tma: u8,  // 0xFF06
    pub tac: u8,  // 0xFF07
    timer_enabled: bool,
    current_bit: u16,
    last_bit: bool,
    overflow: bool,
    ticks_since_overflow: u8,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            timer_enabled: false,
            current_bit: 1 << BITS[0],
            last_bit: false,
            overflow: false,
            ticks_since_overflow: 0,
        }
    }

    /// Read from timer registers.
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => (self.div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac | 0xF8,
            _ => 0,
        }
    }

    /// Write to timer registers.
    pub fn write(&mut self, addr: u16, value: u8, interrupt_flag: &mut u8) {
        match addr {
            0xFF04 => self.div = 0,
            0xFF05 => {
                if self.ticks_since_overflow != OVERFLOW_RELOAD_TICK {
                    self.tima = value;
                    self.overflow = false;
                    self.ticks_since_overflow = 0;
                }
            }
            0xFF06 => {
                self.tma = value;
                // If you write to TMA the same tick that TIMA is reloading,
                // TIMA is also set with the new value
                if self.ticks_since_overflow == OVERFLOW_RELOAD_TICK {
                    self.tima = value;
                }
            }
            0xFF07 => {
                let old_enabled = self.timer_enabled;
                let old_bit = self.current_bit;

                self.tac = value & 0b111;
                self.current_bit = 1 << BITS[(self.tac & 0b11) as usize];
                self.timer_enabled = (self.tac & BIT_2) != 0;

                self.tima_glitch(old_enabled, old_bit, interrupt_flag);
            }
            _ => {}
        }
    }

    /// Steps the timer forward by the given number of T-cycles.
    ///
    /// Returns true if a timer interrupt should be triggered.
    pub fn tick(&mut self, interrupt_flag: &mut u8) {
        self.div = self.div.wrapping_add(1);

        let bit = self.timer_enabled && (self.div & self.current_bit) != 0;

        // Detect falling-edge
        if self.last_bit && !bit {
            self.tima = self.tima.wrapping_add(1);
            if self.tima == 0 {
                self.overflow = true;
                self.ticks_since_overflow = 0;
            }
        }

        self.last_bit = bit;

        if self.overflow {
            self.ticks_since_overflow = self.ticks_since_overflow.wrapping_add(1);

            if self.ticks_since_overflow == OVERFLOW_INTERRUPT_TICK {
                *interrupt_flag |= InterruptBit::Timer as u8;
            } else if self.ticks_since_overflow == OVERFLOW_RELOAD_TICK {
                self.tima = self.tma;
            } else if self.ticks_since_overflow == OVERFLOW_RESET_TICK {
                self.overflow = false;
                self.ticks_since_overflow = 0;
            }
        }
    }

    fn tima_glitch(&mut self, old_enabled: bool, old_bit: u16, interrupt_flag: &mut u8) {
        if !old_enabled {
            return;
        }

        if (self.div & old_bit) != 0 {
            if !self.timer_enabled || !(self.div & self.current_bit) != 0 {
                self.tima = self.tima.wrapping_add(1);
                if self.tima == 0 {
                    self.tima = self.tma;
                    *interrupt_flag |= InterruptBit::Timer as u8;
                }
                self.last_bit = false;
            }
        }
    }
}
