pub mod panning_indexes {
    pub const CH4_PAN_L: u8 = 7;
    pub const CH3_PAN_L: u8 = 6;
    pub const CH2_PAN_L: u8 = 5;
    pub const CH1_PAN_L: u8 = 4;
    pub const CH4_PAN_R: u8 = 3;
    pub const CH3_PAN_R: u8 = 2;
    pub const CH2_PAN_R: u8 = 1;
    pub const CH1_PAN_R: u8 = 0;
}

pub fn get_panned_output(sound_panning: u8, panning_bit_index: u8, output: f32) -> f32 {
    if sound_panning & (1 << panning_bit_index) != 0 {
        output
    } else {
        0.0
    }
}

pub fn mix_samples(ch1: f32, ch2: f32, ch3: f32, ch4: f32) -> f32 {
    (ch1 + ch2 + ch3 + ch4) / 4.0
}

pub fn apply_volume_reduction(sample: f32, master_volume: u8) -> f32 {
    let volume_reduction = (master_volume as f32 + 1.0) / 8.0;
    (sample * volume_reduction).clamp(-1.0, 1.0)
}

pub fn as_dac_output(dac_input: f32) -> f32 {
    (dac_input / 7.5) - 1.0
}

/// Simple first-order single-pole IIR low-pass filter
/// y[n] = a * x[n] + (1 - a) * y[n-1]
/// Where:
/// - y[n] is the current output sample
/// - x[n] is the current input sample
/// - y[n-1] is the previous output sample
/// - a is the smoothing factor (0 < a < 1)
pub fn apply_low_pass_filter(sample: f32, previous_sample: f32, alpha: f32) -> f32 {
    alpha * sample + (1.0 - alpha) * previous_sample
}
