use crate::emulator::Emulator;
use std::time::Instant;

pub fn render_menu_bar(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    emulator: &mut Option<Emulator>,
    paused: &mut bool,
    force_dmg: &mut bool,
    show_debug: &mut bool,
    audio_mono: &mut bool,
    audio_volume: &mut f32,
) {
    egui::MenuBar::new().ui(ui, |ui| {
        render_file_menu(ui, ctx, emulator, paused);
        render_emulation_menu(ui, emulator, paused, force_dmg);
        render_audio_menu(ui, audio_mono, audio_volume);
        render_debug_menu(ui, show_debug);

        ui.separator();

        if *paused {
            ui.colored_label(egui::Color32::YELLOW, "⏸ PAUSED");
        } else {
            ui.colored_label(egui::Color32::GREEN, "▶ RUNNING");
        }
    });
}

fn render_file_menu(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    emulator: &mut Option<Emulator>,
    paused: &mut bool,
) {
    ui.menu_button("File", |ui| {
        if ui.button("Load ROM...").clicked() {
            // Reset timing when unpausing to prevent catch-up
            let was_paused = *paused;
            if was_paused && !*paused {
                if let Some(emulator) = emulator {
                    emulator.next_step = Instant::now();
                }
            }
            *paused = true;

            let file = rfd::FileDialog::new()
                .add_filter("Game Boy (Color) ROM", &["gb", "gbc"])
                .set_title("ZetaBoy - Open Game Boy (Color) ROM")
                .pick_file();

            if let Some(path) = file {
                if let Some(path_str) = path.to_str() {
                    *emulator = Emulator::new(path_str, false).ok();
                } else {
                    eprintln!("Failed to convert path to string");
                }
            }
            *paused = false;
        }
        ui.separator();
        if ui.button("Save State").clicked() {
            if let Some(emulator) = emulator {
                match emulator.save_state() {
                    Ok(path) => println!("Saved state to {}", path),
                    Err(e) => eprintln!("Failed to save state: {}", e),
                }
            }
            ui.close();
        }
        if ui.button("Load State").clicked() {
            if let Some(emulator) = emulator {
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
}

fn render_emulation_menu(
    ui: &mut egui::Ui,
    emulator: &mut Option<Emulator>,
    paused: &mut bool,
    force_dmg: &mut bool,
) {
    ui.menu_button("Emulation", |ui| {
        if ui
            .button(if *paused { "Resume" } else { "Pause" })
            .clicked()
        {
            let was_paused = *paused;
            *paused = !*paused;

            // Reset timing when unpausing to prevent catch-up
            if was_paused && !*paused {
                if let Some(emulator) = emulator {
                    emulator.next_step = Instant::now();
                }
            }
            ui.close();
        }
        if ui.button("Reset").clicked() {
            if let Some(_emulator) = emulator {
                // TODO: Implement reset
            }
            ui.close();
        }
        ui.separator();
        if ui.checkbox(force_dmg, "Force Game Boy (DMG)").clicked() {
            // Reload emulator if one is loaded
            if let Some(emulator) = emulator {
                let rom_path = emulator.rom_path.to_str().unwrap_or("").to_string();
                if !rom_path.is_empty() {
                    if let Ok(new_emulator) = Emulator::new(&rom_path, *force_dmg) {
                        *emulator = new_emulator;
                        println!(
                            "Reloaded ROM with {} mode",
                            if *force_dmg { "DMG" } else { "Auto" }
                        );
                    }
                }
            }
        }
    });
}

fn render_audio_menu(
    ui: &mut egui::Ui,
    audio_mono: &mut bool,
    audio_volume: &mut f32,
) {
    ui.menu_button("Audio", |ui| {
        ui.label("Volume:");
        let volume_percent = (*audio_volume * 100.0) as u8;
        ui.add(
            egui::Slider::new(audio_volume, 0.0..=1.0)
                .text(format!("{}%", volume_percent))
                .show_value(false),
        );
        ui.checkbox(audio_mono, "Mono");
    });
}

fn render_debug_menu(ui: &mut egui::Ui, show_debug: &mut bool) {
    ui.menu_button("Debug", |ui| {
        if ui.button("CPU").clicked() {
            *show_debug = !*show_debug;
            ui.close();
        }
        ui.separator();
        ui.label("Display:");
        ui.label(format!(
            "Resolution: {}x{}",
            crate::emulator::ppu::WIDTH,
            crate::emulator::ppu::HEIGHT
        ));
        ui.label("Scaling: Auto-fit");
    });
}
