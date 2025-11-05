use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub enum EnvelopeDirection {
    #[default]
    Decreasing,
    Increasing,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct Envelope {
    pub volume: u8,
    pub counter: u8,
    pub period: u8,
    pub direction: EnvelopeDirection,

    pub starting_volume: u8,
    pub configured_period: u8,
    pub configured_direction: EnvelopeDirection,
}

impl Envelope {
    pub fn clock(&mut self) {
        use EnvelopeDirection::*;

        if self.configured_period == 0 {
            return;
        }

        if self.counter > 0 {
            self.counter -= 1;
        }

        if self.counter == 0 {
            self.counter = if self.period == 0 { 8 } else { self.period };

            match (self.direction, self.volume) {
                (Decreasing, 0) | (Increasing, 15) => {}
                (Decreasing, _) => self.volume -= 1,
                (Increasing, _) => self.volume += 1,
            }
        }
    }

    pub fn trigger(&mut self) {
        self.volume = self.starting_volume;
        self.direction = self.configured_direction;
        self.period = self.configured_period;
        self.counter = if self.period == 0 { 8 } else { self.period };
    }
}
