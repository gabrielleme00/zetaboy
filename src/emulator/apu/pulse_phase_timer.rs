use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct PulsePhaseTimer {
    phase: u8,
    counter: u16,
    frequency: u16,
}

impl PulsePhaseTimer {
    pub fn new() -> Self {
        Self {
            phase: 0,
            counter: 4 * 2048,
            frequency: 0,
        }
    }

    pub fn tick(&mut self) {
        self.counter -= 1;
        if self.counter == 0 {
            // Reload counter and move one step in the waveform
            self.update_counter();
            self.phase = (self.phase + 1) % 8;
        }
    }

    pub fn trigger(&mut self) {
        self.update_counter();
    }

    fn update_counter(&mut self) {
        self.counter = 4 * (2048 - self.frequency);
    }

    pub fn get_phase(&self) -> u8 {
        self.phase
    }

    pub fn get_frequency(&self) -> u16 {
        self.frequency
    }

    pub fn set_frequency(&mut self, freq: u16) {
        self.frequency = freq & 0x07FF;
        self.update_counter();
    }

    pub fn set_frequency_lsb(&mut self, bits: u8) {
        self.frequency = (self.frequency & 0x0700) | (bits as u16);
    }

    pub fn get_frequency_msb(&self) -> u8 {
        ((self.frequency >> 8) & 0x07) as u8
    }

    pub fn set_frequency_msb(&mut self, bits: u8) {
        self.frequency = (self.frequency & 0x00FF) | (((bits as u16) & 0x07) << 8);
    }
}
