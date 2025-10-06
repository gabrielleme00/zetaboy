use eframe::egui;
use std::time::{Duration, Instant};

use crate::{
    audio::AudioSampleSender,
    emulator::{
        CPU_FREQUENCY, Emulator,
        ppu::{HEIGHT, WIDTH},
    },
};

const TIME_STEP: Duration = Duration::from_micros(2_000);
const CYCLES_PER_STEP: u64 = CPU_FREQUENCY as u64 / 500;
const ASPECT_RATIO: f32 = WIDTH as f32 / HEIGHT as f32;

pub struct EmulatorApp {
    pub emulator: Option<Emulator>,
    pub audio_sender: Option<AudioSampleSender>,
    pub texture: Option<egui::TextureHandle>,
    pub show_debug: bool,
    pub paused: bool,
}

impl EmulatorApp {
    pub fn new(emulator: Option<Emulator>, audio_sender: Option<AudioSampleSender>) -> Self {
        Self {
            emulator,
            audio_sender,
            texture: None,
            show_debug: false,
            paused: false,
        }
    }

    fn update_emulator(&mut self) {
        if let Some(emulator) = &mut self.emulator {

            if !emulator.running || self.paused {
                return;
            }

            let now = Instant::now();

            // Emulate in chunks
            while emulator.next_step <= now {
                let mut cycles_this_step = 0u64;
                while cycles_this_step < CYCLES_PER_STEP {
                    let t_cycles_taken = emulator.cpu.step();
                    cycles_this_step += t_cycles_taken as u64;

                    // Process audio
                    if let Some(audio_sender) = &mut self.audio_sender {
                        audio_sender.process_cpu_cycles(t_cycles_taken as u32, || {
                            emulator.cpu.bus.apu.sample_stereo()
                        });
                    }
                }
                emulator.next_step += TIME_STEP;
            }
        }
    }

    fn update_texture(&mut self, ctx: &egui::Context) {
        if let Some(emulator) = &self.emulator {
            let image_buffer = emulator.cpu.bus.ppu.buffer.clone();

            // Convert from u32 RGBA to ColorImage
            let mut rgba_data = Vec::with_capacity(WIDTH * HEIGHT * 4);
            for &pixel in &image_buffer {
                rgba_data.push((pixel >> 16) as u8); // R
                rgba_data.push((pixel >> 8) as u8); // G
                rgba_data.push(pixel as u8); // B
                rgba_data.push(255); // A
            }

            let color_image = egui::ColorImage::from_rgba_unmultiplied([WIDTH, HEIGHT], &rgba_data);

            if let Some(texture) = &mut self.texture {
                texture.set(color_image, egui::TextureOptions::NEAREST);
            } else {
                self.texture = Some(ctx.load_texture(
                    "gameboy_screen",
                    color_image,
                    egui::TextureOptions::NEAREST,
                ));
            }
        }
    }

    fn handle_keyboard_input(&mut self, ctx: &egui::Context) {
        if let Some(emulator) = &mut self.emulator {
            ctx.input(|i| {
                // Gameboy controls
                emulator.input_state.a = i.key_down(egui::Key::S);
                emulator.input_state.b = i.key_down(egui::Key::A);
                emulator.input_state.up = i.key_down(egui::Key::ArrowUp);
                emulator.input_state.down = i.key_down(egui::Key::ArrowDown);
                emulator.input_state.left = i.key_down(egui::Key::ArrowLeft);
                emulator.input_state.right = i.key_down(egui::Key::ArrowRight);
                emulator.input_state.start = i.key_down(egui::Key::Enter);
                emulator.input_state.select = i.key_down(egui::Key::Space);

                // Emulator controls
                emulator.input_state.save = i.key_down(egui::Key::F1);
                emulator.input_state.load = i.key_down(egui::Key::F2);

                if i.key_pressed(egui::Key::F3) {
                    self.show_debug = !self.show_debug;
                }
                if i.key_pressed(egui::Key::Escape) {
                    let was_paused = self.paused;
                    self.paused = !self.paused;

                    // Reset timing when unpausing to prevent catch-up
                    if was_paused && !self.paused {
                        emulator.next_step = Instant::now();
                    }
                }
            });

            emulator.handle_input();
        }
    }
}

impl eframe::App for EmulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard_input(ctx);
        self.update_emulator();
        self.update_texture(ctx);

        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .button("Load ROM...")
                        .clicked()
                    {
                        // TODO: refac pause and file dialog
                        // Reset timing when unpausing to prevent catch-up
                        let was_paused = self.paused;
                        if was_paused && !self.paused {
                            if let Some(emulator) = &mut self.emulator {
                                emulator.next_step = Instant::now();
                            }
                        }
                        self.paused = true;

                        let file = rfd::FileDialog::new()
                            .add_filter("Gameboy ROM", &["gb"])
                            .set_directory("/")
                            .pick_file();

                        if let Some(path) = file {
                            if let Some(path_str) = path.to_str() {
                                self.emulator = Emulator::new(path_str).ok();
                            } else {
                                eprintln!("Failed to convert path to string");
                            }
                        }
                        self.paused = false;
                    }
                    ui.separator();
                    if ui.button("Save State").clicked() {
                        if let Some(emulator) = &mut self.emulator {
                            match emulator.save_state() {
                                Ok(path) => println!("Saved state to {}", path),
                                Err(e) => eprintln!("Failed to save state: {}", e),
                            }
                        }
                        ui.close();
                    }
                    if ui.button("Load State").clicked() {
                        if let Some(emulator) = &mut self.emulator {
                            match emulator.load_state() {
                                Ok(path) => println!("Loaded state from {}", path),
                                Err(e) => eprintln!("Failed to load state: {}", e),
                            }
                        }
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Emulation", |ui| {
                    if ui
                        .button(if self.paused { "Resume" } else { "Pause" })
                        .clicked()
                    {
                        let was_paused = self.paused;
                        self.paused = !self.paused;

                        // Reset timing when unpausing to prevent catch-up
                        if was_paused && !self.paused {
                            if let Some(emulator) = &mut self.emulator {
                                emulator.next_step = Instant::now();
                            }
                        }
                        ui.close();
                    }
                    if ui.button("Reset").clicked() {
                        if let Some(_emulator) = &mut self.emulator {
                            // TODO: Implement reset
                        }
                        ui.close();
                    }
                });

                ui.menu_button("Debug", |ui| {
                    if ui.button("CPU").clicked() {
                        self.show_debug = !self.show_debug;
                        ui.close();
                    }
                    ui.separator();
                    ui.label("Display:");
                    ui.label(format!("Resolution: {}x{}", WIDTH, HEIGHT));
                    ui.label("Scaling: Auto-fit");
                });

                ui.separator();

                if self.paused {
                    ui.colored_label(egui::Color32::YELLOW, "⏸ PAUSED");
                } else {
                    ui.colored_label(egui::Color32::GREEN, "▶ RUNNING");
                }
            });
        });

        // Main emulator display
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.texture {
                // Calculate the available space
                let available_rect = ui.available_rect_before_wrap();
                let available_size = available_rect.size();

                // Calculate the maximum size that fits while maintaining aspect ratio
                let max_width = available_size.x * 0.9;
                let max_height = available_size.y * 0.9;

                let (image_width, image_height) = if max_width / max_height > ASPECT_RATIO {
                    // Height is the limiting factor
                    let height = max_height;
                    let width = height * ASPECT_RATIO;
                    (width, height)
                } else {
                    // Width is the limiting factor
                    let width = max_width;
                    let height = width / ASPECT_RATIO;
                    (width, height)
                };

                let image_size = egui::Vec2::new(image_width, image_height);

                // Center the image
                ui.centered_and_justified(|ui| {
                    ui.image((texture.id(), image_size));
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                    ui.label("Loading emulator...");
                });
            }
        });

        // Debug panel
        if self.show_debug {
            egui::SidePanel::right("debug_panel")
                .default_width(140.0)
                .show(ctx, |ui| {
                    ui.heading("CPU");
                    ui.separator();

                    if let Some(emulator) = &self.emulator {
                        ui.label("16-bit Registers");
                        ui.monospace(format!("PC = ${:04X}", emulator.cpu.reg.pc));
                        ui.monospace(format!("SP = ${:04X}", emulator.cpu.reg.sp));
                        ui.separator();
                        ui.label("8-bit Registers");
                        ui.monospace(format!(
                            "A = ${:02X} F = ${:02X}",
                            emulator.cpu.reg.a,
                            u8::from(emulator.cpu.reg.f)
                        ));
                        ui.monospace(format!(
                            "B = ${:02X} C = ${:02X}",
                            emulator.cpu.reg.b, emulator.cpu.reg.c
                        ));
                        ui.monospace(format!(
                            "D = ${:02X} E = ${:02X}",
                            emulator.cpu.reg.d, emulator.cpu.reg.e
                        ));
                        ui.monospace(format!(
                            "H = ${:02X} L = ${:02X}",
                            emulator.cpu.reg.h, emulator.cpu.reg.l
                        ));
                        ui.separator();
                        ui.label("Flags");
                        ui.monospace(format!(
                            "Z = {} N = {}\nH = {} C = {}",
                            if emulator.cpu.reg.f.z { "1" } else { "0" },
                            if emulator.cpu.reg.f.n { "1" } else { "0" },
                            if emulator.cpu.reg.f.h { "1" } else { "0" },
                            if emulator.cpu.reg.f.c { "1" } else { "0" }
                        ));
                    }
                });
        }

        // Controls help
        egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Controls:");
                ui.separator();
                ui.label("Arrow Keys: D-Pad");
                ui.separator();
                ui.label("S/A: A/B Buttons");
                ui.separator();
                ui.label("Enter: Start");
                ui.separator();
                ui.label("Space: Select");
                ui.separator();
                ui.label("F1/F2: Save/Load State");
                ui.separator();
                ui.label("F3: Debug");
                ui.separator();
                ui.label("Esc: Pause");
            });
        });

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
