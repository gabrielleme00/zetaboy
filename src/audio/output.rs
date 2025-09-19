use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct AudioOutput {
    _stream: cpal::Stream,
    sample_rate: u32,
}

impl AudioOutput {
    pub fn new<F>(mut next_sample: F) -> Result<Self, Box<dyn std::error::Error>>
    where
        F: FnMut() -> (f32, f32) + Send + 'static,
    {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device");
        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0;

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _| {
                for frame in data.chunks_mut(2) {
                    let (left, right) = next_sample();
                    frame[0] = left;
                    frame[1] = right;
                }
            },
            err_fn,
            None,
        )?;

        stream.play()?;
        Ok(Self { _stream: stream, sample_rate })
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
