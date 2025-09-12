mod duty_cycle;
mod envelope;
mod lengh_counter;
mod channels;

pub use channels::PulseChannel;
use std::{collections::VecDeque, sync::OnceLock};

use crate::{
    emulator::{apu::channels::Channel, cpu::memory_bus::io_registers::*},
    utils::bits::{BIT_0, BIT_4, BIT_7},
};
use serde::{Deserialize, Serialize};

pub const OUTPUT_FREQUENCY: u64 = 48000; // Audio output frequency in Hz
pub const APU_CLOCK_SPEED: u64 = 4_194_304; // Game Boy APU clock speed in Hz

const ALL_AUDIO_REGISTERS: [u16; 21] = [
    REG_NR10, REG_NR11, REG_NR12, REG_NR13, REG_NR14, REG_NR21, REG_NR22, REG_NR23, REG_NR24,
    REG_NR30, REG_NR31, REG_NR32, REG_NR33, REG_NR34, REG_NR41, REG_NR42, REG_NR43, REG_NR44,
    REG_NR50, REG_NR51, REG_NR52,
];

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Apu {
    enabled: bool,
    last_divider: u8,
    divider_ticks: u64,
    clock_ticks: u64,
    channel_1: PulseChannel,
    channel_2: PulseChannel,
    #[serde(skip)]
    sample_queue: VecDeque<f32>,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            enabled: false,
            last_divider: 0,
            divider_ticks: 0,
            clock_ticks: 0,
            channel_1: PulseChannel::new(),
            channel_2: PulseChannel::new(),
            sample_queue: VecDeque::new(),
        }
    }

    pub fn tick(&mut self, io_registers: &mut IORegisters) {
        // Update APU enabled state from NR52 register
        let nr52 = io_registers.read(REG_NR52);
        let enabled = (nr52 & BIT_7) != 0;

        // Tick clock timers
        let prev_clock = self.clock_ticks;
        self.tick_clock();

        // If APU is disabled, disable all channels
        if !self.enabled {
            if self.enabled {
                for audio_reg in ALL_AUDIO_REGISTERS {
                    io_registers.force_write(audio_reg, 0);
                }
                self.disable();
            }

            if self.should_output_sample(prev_clock) {
                self.sample_queue.push_back(0.0);
                self.sample_queue.push_back(0.0);
            }

            return;
        }

        self.enabled = enabled;

        // Tick 512Hz timers every time DIV bit 4 flips from 1 to 0 (bit 5 in double speed mode)
        let divider = io_registers.read(REG_DIV);
        let divider_mask = BIT_4;
        if self.last_divider & divider_mask != 0 && divider & divider_mask == 0 {
            self.tick_divider();
        }
        self.last_divider = divider;

        self.process_register_updates(io_registers);
    }

    fn tick_clock(&mut self) {
        self.clock_ticks += 1;

        if self.enabled {
            self.channel_1.tick();
            self.channel_2.tick();
        }
    }

    fn tick_divider(&mut self) {
        self.divider_ticks += 1;

        if self.enabled {
            // self.channel_1.tick_divider(self.divider_ticks);
            // self.channel_2.tick_divider(self.divider_ticks);
        }
    }

    pub fn sample(&self, nr50: u8, nr51: u8) {
        let mut sample_l = 0.0;
        let mut sample_r = 0.0;

        // Sample channel 1
        let ch1_sample = self.channel_1.sample_analog();
        let ch1_l = ch1_sample * f64::from(nr51 & BIT_4 != 0);
        let ch1_r = ch1_sample * f64::from(nr51 & BIT_0 != 0);
        sample_l += ch1_l;
        sample_r += ch1_r;

        // Sample channel 2
        let ch2_sample = self.channel_2.sample_analog();
        let ch2_l = ch2_sample * f64::from(nr51 & BIT_4 != 0);
        let ch2_r = ch2_sample * f64::from(nr51 & BIT_0 != 0);
        sample_l += ch2_l;
        sample_r += ch2_r;

        // Master volume multipliers range from [1, 8]
        let l_volume = ((nr50 & 0x70) >> 4) + 1;
        let r_volume = (nr50 & 0x07) + 1;

        // Apply L/R volume multipliers; signals now range from [-32, 32]
        let sample_l = sample_l * f64::from(l_volume);
        let sample_r = sample_r * f64::from(r_volume);

        // Map [-32, 32] to [-1, 1] before applying high-pass filter
        let mut sample_l = sample_l / 32.0;
        let mut sample_r = sample_r / 32.0;
    }

    fn disable(&mut self) {
        self.enabled = false;
        self.divider_ticks = 0;
        self.channel_1.channel_enabled = false;
        self.channel_2.channel_enabled = false;
    }

    // Return whether the APU emulator should output audio samples during the current tick.
    // This is currently just a naive "output every 4.194304 MHz / <output_frequency> clock cycles"
    fn should_output_sample(&self, prev_clock_ticks: u64) -> bool {
        static SAMPLE_RATE: OnceLock<f64> = OnceLock::new();

        let sample_rate =
            *SAMPLE_RATE.get_or_init(|| OUTPUT_FREQUENCY as f64 / APU_CLOCK_SPEED as f64);

        let prev_period = (prev_clock_ticks as f64 * sample_rate).round() as u64;
        let current_period = (self.clock_ticks as f64 * sample_rate).round() as u64;

        prev_period != current_period
    }

    fn process_register_updates(&mut self, io_registers: &mut IORegisters) {
        if self.enabled {
            // self.channel_1
            //     .process_register_updates(io_registers, self.divider_ticks);
            // self.channel_2
            //     .process_register_updates(io_registers, self.divider_ticks);
        }
    }
}
