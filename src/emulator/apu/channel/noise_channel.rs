use super::*;

const LFSR_INITIAL: u16 = 0x7FFF;

#[derive(Clone, Deserialize, Serialize)]
pub struct NoiseChannel {
    pub channel_enabled: bool,
    dac_enabled: bool,
    lfsr: u16,
    clock_divider: u8,
    width_mode: bool,
    timer: u16,
    timer_period: u16,
    length_counter: LengthCounter,
    envelope: Envelope,
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self {
            channel_enabled: false,
            dac_enabled: false,
            lfsr: LFSR_INITIAL,
            clock_divider: 0,
            width_mode: false,
            timer: 0,
            timer_period: 0,
            length_counter: LengthCounter::new(),
            envelope: Envelope::default(),
        }
    }

    pub fn tick(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        } else {
            self.timer = self.timer_period;
            self.clock_lfsr();
        }
    }

    fn clock_lfsr(&mut self) {
        let bit0 = self.lfsr & 1;
        let bit1 = (self.lfsr >> 1) & 1;
        let feedback = bit0 ^ bit1;

        self.lfsr >>= 1;
        self.lfsr |= feedback << 14;

        if self.width_mode {
            // 7-bit mode: also put feedback in bit 6
            self.lfsr &= !0x40;
            self.lfsr |= feedback << 6;
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
            return 0.0;
        }

        // LFSR bit 0 inverted determines output
        let noise_output = if (self.lfsr & 1) == 0 { 1 } else { 0 };
        (noise_output * self.envelope.volume) as f32
    }

    pub fn analog_output(&self) -> f32 {
        as_dac_output(self.digital_output())
    }

    fn trigger(&mut self) {
        if self.dac_enabled {
            self.channel_enabled = true;
        }

        self.lfsr = LFSR_INITIAL;
        self.timer = self.timer_period;
        self.length_counter.trigger();
        self.envelope.trigger();
    }

    pub fn set_length_settings(&mut self, value: u8) {
        self.length_counter.load(value & 0x3F);
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

    pub fn get_frequency_settings(&self) -> u8 {
        (self.clock_divider & 0x07) | (if self.width_mode { 0x08 } else { 0x00 })
    }

    pub fn set_frequency_settings(&mut self, value: u8) {
        self.clock_divider = value & 0x07;
        self.width_mode = (value & 0x08) != 0;

        let divisor = match self.clock_divider {
            0 => 8,
            n => (n as u16) * 16,
        };
        let shift_amount = (value >> 4) & 0x0F;
        self.timer_period = divisor << shift_amount;
    }

    pub fn get_control_settings(&self) -> u8 {
        if self.length_counter.enabled {
            BIT_6
        } else {
            0
        }
    }

    pub fn set_control_settings(&mut self, value: u8) {
        if value & BIT_7 != 0 {
            self.trigger();
        }
        self.length_counter.enabled = (value & BIT_6) != 0;
    }
}