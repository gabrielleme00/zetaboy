use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub enum DutyCycle {
    #[default]
    Eighth,
    Fourth,
    Half,
    ThreeFourths,
}

impl DutyCycle {
    pub fn waveform_step(self, phase: u8) -> u8 {
        let waveform = match self {
            Self::Eighth => [0, 0, 0, 0, 0, 0, 0, 1],
            Self::Fourth => [1, 0, 0, 0, 0, 0, 0, 1],
            Self::Half => [1, 0, 0, 0, 0, 1, 1, 1],
            Self::ThreeFourths => [0, 1, 1, 1, 1, 1, 1, 0],
        };
        waveform[phase as usize]
    }
}
