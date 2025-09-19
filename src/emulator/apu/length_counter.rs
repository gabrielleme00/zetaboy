use serde::{Deserialize, Serialize};

const MAX_LENGTH: u8 = 64;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct LengthCounter {
    pub enabled: bool,
    counter: u8,
}

impl LengthCounter {
    pub fn new() -> Self {
        Self { enabled: false, counter: MAX_LENGTH }
    }

    pub fn get(&self) -> u8 {
        self.counter
    }

    pub fn load(&mut self, length: u8) {
        self.counter = MAX_LENGTH - length;
    }

    pub fn clock(&mut self, channel_enabled: &mut bool) {
        if !self.enabled || self.counter == 0 {
            return;
        }

        self.counter -= 1;
        if self.counter == 0 {
            *channel_enabled = false;
        }
    }

    pub fn trigger(&mut self) {
        if self.counter == 0 {
            self.counter = MAX_LENGTH;
        }
    }
}
