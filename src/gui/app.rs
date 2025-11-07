use eframe::egui_glow;
use std::time::{Duration, Instant};

use crate::{
    audio::AudioSampleSender,
    emulator::{
        CPU_FREQUENCY, Emulator,
        ppu::{HEIGHT, WIDTH},
    },
    rendering::{GlContext, init_gl_context, render_with_shader},
};

use super::{input, menu, panels};

const TIME_STEP: Duration = Duration::from_micros(2_000);
const CYCLES_PER_STEP: u64 = (CPU_FREQUENCY as u64 * TIME_STEP.as_micros() as u64) / 1_000_000;

pub struct EmulatorApp {
    pub emulator: Option<Emulator>,
    pub audio_sender: Option<AudioSampleSender>,
    pub show_debug: bool,
    pub paused: bool,
    pub force_dmg: bool,
    pub audio_mono: bool,
    pub audio_volume: f32,
    gl_context: Option<GlContext>,
}

impl EmulatorApp {
    pub fn new(emulator: Option<Emulator>, audio_sender: Option<AudioSampleSender>) -> Self {
        Self {
            emulator,
            audio_sender,
            show_debug: false,
            paused: false,
            force_dmg: false,
            audio_mono: false,
            audio_volume: 0.5,
            gl_context: None,
        }
    }

    fn update_emulator(&mut self) {
        if let Some(emulator) = &mut self.emulator {
            if !emulator.running || self.paused {
                return;
            }

            let now = Instant::now();

            // Emulate in chunks (TIME_STEP) to keep timing consistent
            while emulator.next_step <= now {
                let mut cycles_this_step = 0u64;
                while cycles_this_step < CYCLES_PER_STEP {
                    let t_cycles_taken = emulator.cpu.step();
                    cycles_this_step += t_cycles_taken as u64;

                    // Process audio
                    if let Some(audio_sender) = &mut self.audio_sender {
                        let audio_mono = self.audio_mono;
                        let audio_volume = self.audio_volume;
                        audio_sender.process_cpu_cycles(t_cycles_taken as u32, || {
                            let (left, right) = if audio_mono {
                                emulator.cpu.bus.apu.sample_mono()
                            } else {
                                emulator.cpu.bus.apu.sample_stereo()
                            };
                            (left * audio_volume, right * audio_volume)
                        });
                    }
                }
                emulator.next_step += TIME_STEP;
            }
        }
    }

    fn render_emulator(&mut self, ui: &mut egui::Ui) {
        if let Some(emulator) = &self.emulator {
            let image_buffer = emulator.cpu.bus.ppu.buffer.clone();

            let available_size = ui.available_size();
            let scale_x = (available_size.x / WIDTH as f32).floor().max(1.0);
            let scale_y = (available_size.y / HEIGHT as f32).floor().max(1.0);
            let scale = scale_x.min(scale_y); // Use the smaller scale to fit in both dimensions
            
            let display_size = egui::vec2(
                WIDTH as f32 * scale,
                HEIGHT as f32 * scale,
            );

            // Center the allocated space
            ui.centered_and_justified(|ui| {
                let (rect, _response) = ui.allocate_exact_size(display_size, egui::Sense::hover());

                if let Some(gl_context) = &self.gl_context {
                    let program = gl_context.program;
                    let vao = gl_context.vao;
                    let texture = gl_context.texture;

                    let gl_context_clone = GlContext {
                        program,
                        vao,
                        vbo: gl_context.vbo,
                        ebo: gl_context.ebo,
                        texture,
                    };

                    let callback = egui::PaintCallback {
                        rect,
                        callback: std::sync::Arc::new(egui_glow::CallbackFn::new(
                            move |_info, painter| {
                                render_with_shader(painter.gl(), &gl_context_clone, &image_buffer);
                            },
                        )),
                    };

                    ui.painter().add(callback);
                }
            });
        }
    }
}

impl eframe::App for EmulatorApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Initialize GL context on first frame
        if self.gl_context.is_none() {
            if let Some(gl) = frame.gl() {
                match init_gl_context(gl) {
                    Ok(gl_context) => self.gl_context = Some(gl_context),
                    Err(e) => eprintln!("Failed to initialize GL context: {}", e),
                }
            }
        }

        input::handle_keyboard_input(
            ctx,
            &mut self.emulator,
            &mut self.show_debug,
            &mut self.paused,
        );
        self.update_emulator();

        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            menu::render_menu_bar(
                ctx,
                ui,
                &mut self.emulator,
                &mut self.paused,
                &mut self.force_dmg,
                &mut self.show_debug,
                &mut self.audio_mono,
                &mut self.audio_volume,
            );
        });

        // Main emulator display
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_emulator(ui);
        });

        // Debug panel
        if self.show_debug {
            panels::render_debug_panel(ctx, &self.emulator);
        }

        // Controls help bar
        panels::render_controls_panel(ctx);

        // Request repaint for smooth animation
        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(emulator) = &self.emulator {
            if let Err(e) = emulator.save_sram() {
                eprintln!("Failed to save SRAM: {}", e);
            }
        }
    }
}
