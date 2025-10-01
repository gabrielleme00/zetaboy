use crate::utils::bits::*;
use serde::{Deserialize, Serialize};

use super::{
    DutyCycle, Envelope, EnvelopeDirection, LengthCounter, PulsePhaseTimer, Sweep, SweepDirection,
    as_dac_output,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PulseChannel {
    pub channel_enabled: bool,
    dac_enabled: bool,
    sweep: Option<Sweep>,
    timer: PulsePhaseTimer,
    duty_cycle: DutyCycle,
    length_counter: LengthCounter,
    envelope: Envelope,
}

impl PulseChannel {
    pub fn new() -> Self {
        Self {
            channel_enabled: false,
            dac_enabled: false,
            sweep: None,
            timer: PulsePhaseTimer::new(),
            duty_cycle: DutyCycle::default(),
            length_counter: LengthCounter::new(),
            envelope: Envelope::default(),
        }
    }

    pub fn new_with_sweep() -> Self {
        Self {
            channel_enabled: false,
            dac_enabled: false,
            sweep: Some(Sweep::new()),
            timer: PulsePhaseTimer::new(),
            duty_cycle: DutyCycle::default(),
            length_counter: LengthCounter::new(),
            envelope: Envelope::default(),
        }
    }

    pub fn tick(&mut self) {
        self.timer.tick();
    }

    pub fn clock_sweep(&mut self) {
        if let Some(ref mut sweep) = self.sweep {
            if let Some(new_frequency) = sweep.clock() {
                self.timer.set_frequency(new_frequency);
            }
        }
    }

    pub fn clock_length_counter(&mut self) {
        self.length_counter.clock(&mut self.channel_enabled);
    }

    pub fn clock_envelope(&mut self) {
        self.envelope.clock();
    }

    fn digital_output(&self) -> f32 {
        if !self.channel_enabled || !self.dac_enabled {
            return 7.5;
        }

        let waveform_step = self.duty_cycle.waveform_step(self.timer.get_phase());
        let volume = self.envelope.volume;

        let output = waveform_step * volume;
        output as f32
    }

    pub fn analog_output(&self) -> f32 {
        as_dac_output(self.digital_output())
    }

    fn trigger(&mut self) {
        if self.dac_enabled {
            self.channel_enabled = true;
        }

        self.timer.trigger();

        if self.length_counter.get() == 0 {
            self.length_counter.trigger();
        }

        self.envelope.trigger();

        if let Some(ref mut sweep) = self.sweep {
            sweep.trigger(self.timer.get_frequency());
        }
    }

    pub fn get_sweep_settings(&self) -> u8 {
        if let Some(ref sweep) = self.sweep {
            ((sweep.configured_period & 0x07) << 4)
                | (match sweep.configured_direction {
                    SweepDirection::Increase => 0,
                    SweepDirection::Decrease => 1,
                } << 3)
                | (sweep.configured_shift & 0x07)
        } else {
            0xFF
        }
    }

    pub fn set_sweep_settings(&mut self, value: u8) {
        if let Some(ref mut sweep) = self.sweep {
            sweep.configured_period = (value >> 4) & 0x07;
            sweep.configured_direction = if (value & 0x08) != 0 {
                SweepDirection::Decrease
            } else {
                SweepDirection::Increase
            };
            sweep.configured_shift = value & 0x07;
        }
    }

    pub fn get_length_settings(&self) -> u8 {
        (self.duty_cycle as u8) << 6 | (self.length_counter.get() & 0x3F)
    }

    pub fn set_length_settings(&mut self, value: u8) {
        self.duty_cycle = DutyCycle::from_bits(value >> 6);
        let length_load = value & 0x3F;

        // If length is 0, set it to maximum (64)
        if length_load == 0 {
            self.length_counter.load(64);
        } else {
            self.length_counter.load(length_load);
        }
    }

    pub fn get_envelope_settings(&self) -> u8 {
        (self.envelope.starting_volume << 4)
            | ((match self.envelope.configured_direction {
                EnvelopeDirection::Increasing => 1,
                EnvelopeDirection::Decreasing => 0,
            }) << 3)
            | (self.envelope.configured_period & 0x07)
    }

    pub fn set_envelope_settings(&mut self, value: u8) {
        self.envelope.starting_volume = (value >> 4) & 0x0F;
        self.envelope.configured_direction = if (value & 0x08) != 0 {
            EnvelopeDirection::Increasing
        } else {
            EnvelopeDirection::Decreasing
        };
        self.envelope.configured_period = value & 0x07;
        self.dac_enabled = (value & 0xF8) != 0;

        if !self.dac_enabled {
            self.channel_enabled = false;
        }
    }

    pub fn set_period_low_settings(&mut self, value: u8) {
        self.timer.set_frequency_lsb(value);
    }

    pub fn get_period_high_control_settings(&self) -> u8 {
        (if self.length_counter.enabled {
            BIT_6
        } else {
            0
        }) | (if self.channel_enabled { BIT_7 } else { 0 })
            | (self.timer.get_frequency_msb() & 0x07)
    }

    pub fn set_period_high_control_settings(&mut self, value: u8) {
        if value & BIT_7 != 0 {
            self.trigger();
        }
        self.length_counter.enabled = (value & BIT_6) != 0;
        self.timer.set_frequency_msb(value);
    }
}
