use std::{
    collections::VecDeque,
    sync::mpsc::{Receiver, SyncSender},
};

use super::AudioOutput;
use crate::CPU_FREQUENCY;

pub struct AudioManager {
    _output: AudioOutput,
}

impl AudioManager {
    pub fn new() -> Result<(Self, AudioSampleSender), Box<dyn std::error::Error>> {
        let (tx, rx) = std::sync::mpsc::sync_channel::<(f32, f32)>(16384);
        let output = AudioOutput::new(Self::create_sample_provider(rx))?;
        let sample_rate = output.sample_rate() as f32;
        let manager = Self { _output: output };
        let sender = AudioSampleSender::new(tx, sample_rate);

        Ok((manager, sender))
    }

    fn create_sample_provider(
        audio_rx: Receiver<(f32, f32)>,
    ) -> impl FnMut() -> (f32, f32) + Send + 'static {
        let mut buffer = VecDeque::new();
        let mut last_sample = (0.0f32, 0.0f32);

        move || {
            // Always fill buffer with available samples
            while let Ok(sample) = audio_rx.try_recv() {
                if buffer.len() < 8192 {
                    buffer.push_back(sample);
                }
            }

            if let Some(sample) = buffer.pop_front() {
                last_sample = sample;
            }
            last_sample
        }
    }
}

pub struct AudioSampleSender {
    sender: SyncSender<(f32, f32)>,
    sample_accumulator: f32,
    ticks_per_sample: f32,
    prev_sample: (f32, f32),
    current_sample: (f32, f32),
}

impl AudioSampleSender {
    fn new(sender: SyncSender<(f32, f32)>, sample_rate: f32) -> Self {
        Self {
            sender,
            sample_accumulator: 0.0,
            ticks_per_sample: CPU_FREQUENCY as f32 / sample_rate,
            prev_sample: (0.0, 0.0),
            current_sample: (0.0, 0.0),
        }
    }

    pub fn process_cpu_cycles(&mut self, t_cycles: u32, mut get_sample: impl FnMut() -> (f32, f32)) {
        for _ in 0..t_cycles {
            self.prev_sample = self.current_sample;
            self.current_sample = get_sample();
            
            self.sample_accumulator += 1.0;

            if self.sample_accumulator >= self.ticks_per_sample {
                self.sample_accumulator -= self.ticks_per_sample;
                
                // Calculate interpolation factor (0.0 to 1.0)
                let t = self.sample_accumulator / self.ticks_per_sample;
                
                // Linear interpolation: prev + t * (current - prev)
                let interp_l = self.prev_sample.0 + t * (self.current_sample.0 - self.prev_sample.0);
                let interp_r = self.prev_sample.1 + t * (self.current_sample.1 - self.prev_sample.1);
                
                let _ = self.sender.try_send((interp_l, interp_r));
            }
        }
    }
}
