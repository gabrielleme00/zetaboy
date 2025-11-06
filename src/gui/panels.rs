use crate::emulator::Emulator;

pub fn render_debug_panel(ctx: &egui::Context, emulator: &Option<Emulator>) {
    egui::SidePanel::right("debug_panel")
        .default_width(140.0)
        .show(ctx, |ui| {
            ui.heading("CPU");
            ui.separator();

            if let Some(emulator) = emulator {
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

pub fn render_controls_panel(ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Controls:");
            ui.separator();
            ui.label("WASD: D-Pad");
            ui.separator();
            ui.label("J/K: A/B Buttons");
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
}
