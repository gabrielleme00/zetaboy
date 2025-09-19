use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub enum EnvelopeDirection {
    #[default]
    Decreasing,
    Increasing,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct Envelope {
    // Active counter values
    pub volume: u8,
    pub counter: u8,
    pub period: u8,
    pub direction: EnvelopeDirection,
    // Configured values via registers
    pub starting_volume: u8,
    pub configured_period: u8,
    pub configured_direction: EnvelopeDirection,
}

impl Envelope {
    pub fn clock(&mut self) {
        // Period of 0 disables volume auto-increment/decrement
        if self.period == 0 {
            return;
        }

        self.counter -= 1;
        if self.counter == 0 {
            self.counter = self.period;

            match (self.direction, self.volume) {
                (EnvelopeDirection::Decreasing, 0) | (EnvelopeDirection::Increasing, 15) => {
                    // Volume cannot decrease past 0 or increase past 15; do nothing
                }
                (EnvelopeDirection::Decreasing, _) => {
                    self.volume -= 1;
                }
                (EnvelopeDirection::Increasing, _) => {
                    self.volume += 1;
                }
            }
        }
    }

    pub fn trigger(&mut self) {
        // Copy configured values and reload counter
        self.volume = self.starting_volume;
        self.direction = self.configured_direction;
        self.period = self.configured_period;

        self.counter = self.period;
    }
}
