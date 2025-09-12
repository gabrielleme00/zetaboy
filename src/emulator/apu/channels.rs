mod pulse;

pub use pulse::PulseChannel;

pub(crate) trait Channel {
    fn channel_enabled(&self) -> bool;

    fn dac_enabled(&self) -> bool;

    // Digital sample in the range [0, 15]
    fn sample_digital(&self) -> Option<u8>;

    // "Analog" sample in the range [-1, 1]
    fn sample_analog(&self) -> f64 {
        let Some(digital_sample) = self.sample_digital() else {
            return 0.0;
        };

        (f64::from(digital_sample) - 7.5) / 7.5
    }
}
