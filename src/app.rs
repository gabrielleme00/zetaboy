use pixels::Pixels;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use std::{
    error::Error,
    time::{Duration, Instant},
};

use crate::{
    audio::AudioSampleSender,
    emulator::{
        ppu::{HEIGHT, WIDTH},
        Emulator, CPU_FREQUENCY,
    },
};

const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60);
const TIME_STEP: Duration = Duration::from_micros(2_000);
const CYCLES_PER_STEP: u64 = CPU_FREQUENCY as u64 / 500;

pub struct App {
    pub window: Option<&'static Window>,
    pub emulator: Option<Emulator>,
    pub audio_sender: Option<AudioSampleSender>,
    pub renderer: Option<Pixels<'static>>,
}

impl App {
    fn render_main(&mut self) {
        if let (Some(renderer), Some(emulator)) = (self.renderer.as_mut(), &self.emulator) {
            let frame = renderer.frame_mut();

            // Convert from u32 RGBA to u8 RGBA efficiently
            for (i, &src) in emulator.cpu.bus.ppu.buffer.iter().enumerate() {
                let dst_idx = i * 4;
                if dst_idx + 3 < frame.len() {
                    frame[dst_idx] = (src >> 16) as u8; // R
                    frame[dst_idx + 1] = (src >> 8) as u8; // G
                    frame[dst_idx + 2] = src as u8; // B
                    frame[dst_idx + 3] = 255; // A
                }
            }

            let _ = renderer.render();
        }
    }

    fn try_save_sram(&self) {
        if let Some(emulator) = &self.emulator {
            if let Err(e) = emulator.save_sram() {
                eprintln!("Failed to save SRAM: {}", e);
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            emulator: None,
            audio_sender: None,
            renderer: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(create_main_window(event_loop).unwrap());

        let window = self.window.unwrap();
        let window_size = window.inner_size();

        let surface_texture =
            pixels::SurfaceTexture::new(window_size.width, window_size.height, window);
        let pixels = pixels::PixelsBuilder::new(WIDTH as u32, HEIGHT as u32, surface_texture)
            .build()
            .unwrap();

        self.renderer = Some(pixels);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                // Resize the pixels surface when window is resized
                if let Some(renderer) = &mut self.renderer {
                    if let Err(err) = renderer.resize_surface(size.width, size.height) {
                        eprintln!("Failed to resize pixels surface: {err}");
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        ..
                    },
                ..
            } => {
                if let Some(emulator) = &mut self.emulator {
                    let pressed = state == ElementState::Pressed;
                    emulator.input_state.set_key_state(key_code, pressed);

                    // Handle escape key
                    if key_code == KeyCode::Escape && pressed {
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.render_main();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(emulator) = &mut self.emulator {
            let now = Instant::now();

            // Handle input and pause
            emulator.handle_input();
            if !emulator.running {
                return;
            }

            if emulator.paused {
                std::thread::sleep(Duration::from_millis(16));
                return;
            }

            // Emulate in larger chunks to reduce overhead
            while emulator.next_step <= now {
                let mut cycles_this_step = 0u64;
                while cycles_this_step < CYCLES_PER_STEP {
                    let t_cycles_taken = emulator.cpu.step();
                    cycles_this_step += t_cycles_taken as u64;

                    // Process audio for these cycles (batch processing)
                    if let Some(audio_sender) = &mut self.audio_sender {
                        audio_sender.process_cpu_cycles(t_cycles_taken as u32, || {
                            emulator.cpu.bus.apu.sample_stereo()
                        });
                    }
                }
                emulator.next_step += TIME_STEP;
            }

            // Render at 60Hz only when needed
            if emulator.next_frame <= now {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                emulator.next_frame += FRAME_DURATION;
            } else {
                // Sleep a bit to avoid busy waiting
                std::thread::sleep(Duration::from_micros(100));
            }
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {}
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: ()) {}
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        _event: winit::event::DeviceEvent,
    ) {
    }
    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {}
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.try_save_sram();
    }
    fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) {}
}

fn create_main_window(el: &ActiveEventLoop) -> Result<&'static Window, Box<dyn Error>> {
    use winit::dpi::{PhysicalSize, Size};

    let (width, height) = (WIDTH as u32, HEIGHT as u32);
    let scale = 4;

    let window = Box::new(
        el.create_window(
            Window::default_attributes()
                .with_title("ZetaBoy")
                .with_inner_size(Size::new(PhysicalSize::new(width * scale, height * scale)))
                .with_min_inner_size(Size::new(PhysicalSize::new(width, height))),
        )
        .unwrap(),
    );

    Ok(Box::leak(window))
}
