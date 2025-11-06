use crate::emulator::Emulator;
use std::time::Instant;

pub fn handle_keyboard_input(
    ctx: &egui::Context,
    emulator: &mut Option<Emulator>,
    show_debug: &mut bool,
    paused: &mut bool,
) {
    if let Some(emulator) = emulator {
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
                *show_debug = !*show_debug;
            }
            if i.key_pressed(egui::Key::Escape) {
                let was_paused = *paused;
                *paused = !*paused;

                // Reset timing when unpausing to prevent catch-up
                if was_paused && !*paused {
                    emulator.next_step = Instant::now();
                }
            }
        });

        emulator.handle_input();
    }
}
