mod channel;
mod utils;

use crate::utils::bits::*;

// Modulation units
mod duty_cycle;
mod envelope;
mod length_counter;
mod pulse_phase_timer;
mod sweep;

use channel::{NoiseChannel, PulseChannel, WaveChannel};
use serde::{Deserialize, Serialize};
use utils::{
    apply_low_pass_filter, apply_volume_reduction, get_panned_output, mix_samples,
    panning_indexes::*,
};

#[derive(Clone, Deserialize, Serialize)]
pub struct Apu {
    enabled: bool,
    frame_sequencer_step: u8,
    frame_sequencer_counter: u16,
    master_volume: u8,
    sound_panning: u8,
    channel_1: PulseChannel,
    channel_2: PulseChannel,
    channel_3: WaveChannel,
    channel_4: NoiseChannel,
    last_sample: (f32, f32),
}

impl Apu {
    pub fn new() -> Self {
        Self {
            enabled: false,
            frame_sequencer_step: 0,
            frame_sequencer_counter: 0,
            master_volume: 0,
            sound_panning: 0,
            channel_1: PulseChannel::new_with_sweep(),
            channel_2: PulseChannel::new(),
            channel_3: WaveChannel::new(),
            channel_4: NoiseChannel::new(),
            last_sample: (0.0, 0.0),
        }
    }

    pub fn sample_stereo(&mut self) -> (f32, f32) {
        // Calculate the mixed output of all channels
        let (left, right) = {
            let ch1 = self.channel_1.analog_output();
            let ch2 = self.channel_2.analog_output();
            let ch3 = self.channel_3.analog_output();
            let ch4 = self.channel_4.analog_output();

            let left_sample = self.generate_left_sample(ch1, ch2, ch3, ch4);
            let right_sample = self.generate_right_sample(ch1, ch2, ch3, ch4);

            (left_sample, right_sample)
        };

        // Apply a simple low-pass filter to smooth the output
        let cutoff_hz = 8000.0;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz);
        let dt = 1.0 / 44100.0;
        let alpha = dt / (rc + dt);
        self.last_sample = (
            apply_low_pass_filter(left, self.last_sample.0, alpha),
            apply_low_pass_filter(right, self.last_sample.1, alpha),
        );

        self.last_sample
    }

    fn generate_left_sample(
        &self,
        ch1_dac_out: f32,
        ch2_dac_out: f32,
        ch3_dac_out: f32,
        ch4_dac_out: f32,
    ) -> f32 {
        let left_master_volume = (self.master_volume & 0b01110000) >> 4;
        let pan = self.sound_panning;
        let ch1_panned = get_panned_output(pan, CH1_PAN_L, ch1_dac_out);
        let ch2_panned = get_panned_output(pan, CH2_PAN_L, ch2_dac_out);
        let ch3_panned = get_panned_output(pan, CH3_PAN_L, ch3_dac_out);
        let ch4_panned = get_panned_output(pan, CH4_PAN_L, ch4_dac_out);
        let mix = mix_samples(ch1_panned, ch2_panned, ch3_panned, ch4_panned);
        apply_volume_reduction(mix, left_master_volume)
    }

    fn generate_right_sample(
        &self,
        ch1_dac_out: f32,
        ch2_dac_out: f32,
        ch3_dac_out: f32,
        ch4_dac_out: f32,
    ) -> f32 {
        let right_master_volume = self.master_volume & 0b111;
        let pan = self.sound_panning;
        let ch1_panned = get_panned_output(pan, CH1_PAN_R, ch1_dac_out);
        let ch2_panned = get_panned_output(pan, CH2_PAN_R, ch2_dac_out);
        let ch3_panned = get_panned_output(pan, CH3_PAN_R, ch3_dac_out);
        let ch4_panned = get_panned_output(pan, CH4_PAN_R, ch4_dac_out);
        let mix = mix_samples(ch1_panned, ch2_panned, ch3_panned, ch4_panned);
        apply_volume_reduction(mix, right_master_volume)
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF10 => self.channel_1.get_sweep_settings() | 0x80,
            0xFF11 => self.channel_1.get_length_settings() | 0x3F,
            0xFF12 => self.channel_1.get_envelope_settings(),
            0xFF14 => self.channel_1.get_period_high_control_settings() | 0xBF,
            0xFF16 => self.channel_2.get_length_settings() | 0x3F,
            0xFF17 => self.channel_2.get_envelope_settings(),
            0xFF19 => self.channel_2.get_period_high_control_settings() | 0xBF,
            0xFF1A => self.channel_3.get_dac_settings(),
            0xFF1C => self.channel_3.get_volume_settings(),
            0xFF1E => self.channel_3.get_period_high_control_settings() | 0xBF,
            0xFF21 => self.channel_4.get_envelope_settings(),
            0xFF22 => self.channel_4.get_frequency_settings(),
            0xFF23 => self.channel_4.get_control_settings() | 0xBF,
            0xFF24 => self.master_volume,
            0xFF25 => self.sound_panning,
            0xFF26 => self.get_master_control(),
            0xFF30..=0xFF3F => self.channel_3.read_wave_ram((addr - 0xFF30) as u8),
            _ if addr < 0xFF10 || addr > 0xFF3F => panic!("Invalid APU address: 0x{:04X}", addr),
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        if !self.enabled && addr != 0xFF26 {
            return;
        }

        match addr {
            0xFF10 => self.channel_1.set_sweep_settings(value),
            0xFF11 => self.channel_1.set_length_settings(value),
            0xFF12 => self.channel_1.set_envelope_settings(value),
            0xFF13 => self.channel_1.set_period_low_settings(value),
            0xFF14 => self.channel_1.set_period_high_control_settings(value),
            0xFF16 => self.channel_2.set_length_settings(value),
            0xFF17 => self.channel_2.set_envelope_settings(value),
            0xFF18 => self.channel_2.set_period_low_settings(value),
            0xFF19 => self.channel_2.set_period_high_control_settings(value),
            0xFF1A => self.channel_3.set_dac_settings(value),
            0xFF1B => self.channel_3.set_length_settings(value),
            0xFF1C => self.channel_3.set_volume_settings(value),
            0xFF1D => self.channel_3.set_period_low_settings(value),
            0xFF1E => self.channel_3.set_period_high_control_settings(value),
            0xFF20 => self.channel_4.set_length_settings(value),
            0xFF21 => self.channel_4.set_envelope_settings(value),
            0xFF22 => self.channel_4.set_frequency_settings(value),
            0xFF23 => self.channel_4.set_control_settings(value),
            0xFF24 => self.master_volume = value,
            0xFF25 => self.sound_panning = value,
            0xFF26 => self.set_master_control(value),
            0xFF30..=0xFF3F => self.channel_3.write_wave_ram((addr - 0xFF30) as u8, value),
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        self.channel_1.tick();
        self.channel_2.tick();
        self.channel_3.tick();
        self.channel_4.tick();

        // Clock frame sequencer at 512 Hz (every 2048 T-cycles)
        self.frame_sequencer_counter += 1;
        if self.frame_sequencer_counter >= 2048 {
            self.frame_sequencer_counter = 0;
            self.clock_frame_sequencer();
        }
    }

    fn clock_frame_sequencer(&mut self) {
        if self.frame_sequencer_step % 2 == 0 {
            // Length counters are clocked on steps 0, 2, 4, 6
            self.channel_1.clock_length_counter();
            self.channel_2.clock_length_counter();
            self.channel_3.clock_length_counter();
            self.channel_4.clock_length_counter();

            if matches!(self.frame_sequencer_step, 2 | 6) {
                // Sweep is clocked on steps 0 and 4
                self.channel_1.clock_sweep();
            }
        } else if self.frame_sequencer_step == 7 {
            // Envelopes are clocked on step 7
            self.channel_1.clock_envelope();
            self.channel_2.clock_envelope();
            self.channel_4.clock_envelope();
        }
        self.frame_sequencer_step = (self.frame_sequencer_step + 1) % 8;
    }

    fn get_master_control(&self) -> u8 {
        let mut value = 0;
        if self.enabled {
            value |= BIT_7;
        }
        if self.channel_4.channel_enabled {
            value |= BIT_3;
        }
        if self.channel_3.channel_enabled {
            value |= BIT_2;
        }
        if self.channel_2.channel_enabled {
            value |= BIT_1;
        }
        if self.channel_1.channel_enabled {
            value |= BIT_0;
        }
        value
    }

    fn set_master_control(&mut self, value: u8) {
        self.enabled = (value & BIT_7) != 0;
    }
}
