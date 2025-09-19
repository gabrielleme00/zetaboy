use super::*;

const SAMPLE_COUNT: u8 = 32;
const SAMPLES_PER_BYTE: u8 = 2;
const WAVE_RAM_SIZE: u8 = SAMPLE_COUNT / SAMPLES_PER_BYTE;

#[derive(Clone, Deserialize, Serialize)]
pub struct WaveChannel {
    pub channel_enabled: bool,
    dac_enabled: bool,
    wave_ram: [u8; 16],
    sample_position: u8,
    timer: u16,
    timer_period: u16,
    volume_shift: u8,
    length_counter: LengthCounter,
}

impl WaveChannel {
    pub fn new() -> Self {
        Self {
            channel_enabled: false,
            dac_enabled: false,
            wave_ram: [0; WAVE_RAM_SIZE as usize],
            sample_position: 0,
            timer: 0,
            timer_period: 0,
            volume_shift: 0,
            length_counter: LengthCounter::new(),
        }
    }

    pub fn tick(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        } else {
            self.timer = self.timer_period;
            if self.channel_enabled && self.dac_enabled {
                self.sample_position = (self.sample_position + 1) % SAMPLE_COUNT;
            }
        }
    }

    pub fn clock_length_counter(&mut self) {
        self.length_counter.clock(&mut self.channel_enabled);
    }

    fn digital_output(&self) -> f32 {
        if !self.channel_enabled || !self.dac_enabled {
            return 0.0;
        }

        // Get current 4-bit sample from wave RAM
        let byte_index = (self.sample_position / 2) as usize;
        let nibble = if self.sample_position % 2 == 0 {
            // High nibble (first sample in byte)
            (self.wave_ram[byte_index] >> 4) & 0x0F
        } else {
            // Low nibble (second sample in byte)
            self.wave_ram[byte_index] & 0x0F
        };

        // Apply volume shift
        let shifted_sample = match self.volume_shift {
            0 => 0,           // Mute
            1 => nibble,      // 100%
            2 => nibble >> 1, // 50%
            3 => nibble >> 2, // 25%
            _ => 0,           // Invalid, treat as mute
        };

        shifted_sample as f32
    }

    pub fn analog_output(&self) -> f32 {
        as_dac_output(self.digital_output())
    }

    fn trigger(&mut self) {
        if self.dac_enabled {
            self.channel_enabled = true;
        }

        self.sample_position = 0;
        self.timer = self.timer_period;
        self.length_counter.trigger();
    }

    pub fn get_dac_settings(&self) -> u8 {
        if self.dac_enabled {
            BIT_7
        } else {
            0
        }
    }

    pub fn set_dac_settings(&mut self, value: u8) {
        self.dac_enabled = (value & BIT_7) != 0;
        if !self.dac_enabled {
            self.channel_enabled = false;
        }
    }

    pub fn set_length_settings(&mut self, value: u8) {
        self.length_counter.load(value);
    }

    pub fn get_volume_settings(&self) -> u8 {
        (self.volume_shift & 0x03) << 5
    }

    pub fn set_volume_settings(&mut self, value: u8) {
        self.volume_shift = (value >> 5) & 0x03;
    }

    pub fn set_period_low_settings(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | (value as u16);
    }

    pub fn get_period_high_control_settings(&self) -> u8 {
        (if self.length_counter.enabled {
            BIT_6
        } else {
            0
        }) | ((self.timer_period >> 8) as u8 & 0x07)
    }

    pub fn set_period_high_control_settings(&mut self, value: u8) {
        if value & BIT_7 != 0 {
            self.trigger();
        }
        self.length_counter.enabled = (value & BIT_6) != 0;
        let frequency = (self.timer_period & 0x00FF) | (((value & 0x07) as u16) << 8);
        self.timer_period = (2048 - frequency) * 2;
    }

    pub fn read_wave_ram(&self, offset: u8) -> u8 {
        if offset < 16 {
            if self.channel_enabled {
                self.wave_ram[self.current_sample_index()]
            } else {
                self.wave_ram[offset as usize]
            }
        } else {
            0xFF
        }
    }

    pub fn write_wave_ram(&mut self, offset: u8, value: u8) {
        if offset < WAVE_RAM_SIZE as u8 {
            if self.channel_enabled {
                self.wave_ram[self.current_sample_index()] = value;
            } else {
                self.wave_ram[offset as usize] = value;
            }
        }
    }

    fn current_sample_index(&self) -> usize {
        (self.sample_position / 2) as usize
    }
}
