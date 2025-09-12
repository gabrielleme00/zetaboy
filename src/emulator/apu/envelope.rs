use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
enum Direction {
    #[default]
    Decreasing,
    Increasing,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct Envelope {
    // Active counter values
    volume: u8,
    counter: u8,
    period: u8,
    direction: Direction,
    // Configured values via registers
    starting_volume: u8,
    configured_period: u8,
    configured_direction: Direction,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            volume: 0,
            counter: 0,
            period: 0,
            direction: Direction::Decreasing,
            starting_volume: 0,
            configured_period: 0,
            configured_direction: Direction::Decreasing,
        }
    }

    pub fn clock(&mut self) {
        // Period of 0 disables volume auto-increment/decrement
        if self.period == 0 {
            return;
        }

        self.counter -= 1;
        if self.counter == 0 {
            self.counter = self.period;

            match (self.direction, self.volume) {
                (Direction::Decreasing, 0) | (Direction::Increasing, 15) => {
                    // Volume cannot decrease past 0 or increase past 15; do nothing
                }
                (Direction::Decreasing, _) => {
                    self.volume -= 1;
                }
                (Direction::Increasing, _) => {
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

    pub fn get_volume(&self) -> u8 {
        self.volume
    }
}
