mod noise_channel;
mod pulse_channel;
mod wave_channel;

use serde::{Deserialize, Serialize};

use crate::utils::bits::*;

use super::{
    duty_cycle::DutyCycle,
    envelope::{Envelope, EnvelopeDirection},
    length_counter::LengthCounter,
    pulse_phase_timer::PulsePhaseTimer,
    sweep::{Sweep, SweepDirection},
    utils::as_dac_output,
};

pub use noise_channel::NoiseChannel;
pub use pulse_channel::PulseChannel;
pub use wave_channel::WaveChannel;
