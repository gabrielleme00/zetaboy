use serde::{Deserialize, Serialize};

use crate::emulator::apu::channels::Channel;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PulseChannel {
    pub channel_enabled: bool,
    // timer: PulsePhaseTimer,
    // duty_cycle: DutyCycle,
    // length_counter: LengthCounter,
    // envelope: Envelope,
    dac_on: bool,
}

impl PulseChannel {
    pub fn new() -> Self {
        Self {
            channel_enabled: false,
            // timer: PulsePhaseTimer::new(),
            // duty_cycle: DutyCycle::default(),
            // length_counter: LengthCounter::new(),
            // envelope: Envelope::new(),
            dac_on: false,
        }
    }

    pub fn tick(&mut self) {
        // self.timer.tick();
    }

    pub fn clock_length_counter(&mut self) {
        // self.length_counter.clock(&mut self.channel_enabled);
    }

    pub fn clock_envelope(&mut self) {
        // self.envelope.clock();
    }

    pub fn trigger(&mut self) {
        // self.channel_enabled = true;

        // self.timer.trigger();
        // self.length_counter.trigger();
        // self.envelope.trigger();
    }

    pub fn sample(&self) -> u8 {
        // if !self.channel_enabled {
        //     return 0;
        // }

        // let waveform_step = self.duty_cycle.waveform_step(self.timer.get_phase());
        // let volume = self.envelope.get_volume();

        // waveform_step * volume
        0
    }
}

impl Channel for PulseChannel {
    fn channel_enabled(&self) -> bool {
        self.channel_enabled
    }

    fn dac_enabled(&self) -> bool {
        self.dac_on
    }

    fn sample_digital(&self) -> Option<u8> {
        if !self.dac_on {
            // Return no signal if the DAC is disabled
            return None;
        }

        if !self.channel_enabled {
            // Return a constant 0 if the channel is disabled but the DAC is on
            return Some(0);
        }

        // Digital output is 0 if waveform sample is 0, {volume} otherwise
        // let wave_step = self.duty_cycle.waveform()[self.phase_position as usize];
        // Some(wave_step * self.volume_control.volume)
        None
    }
}
