mod apu;
mod cart;
mod cpu;
pub mod ppu;
mod timer;

use crate::audio::AudioSampleSender;
use crate::emulator::cpu::memory_bus::io_registers::JoypadButton;
use crate::{CPU_FREQUENCY, PRINT_CART_INFO};
use cart::Cart;
use cpu::CPU;
use minifb::{Key, Window, WindowOptions};
use ppu::{HEIGHT, WIDTH};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60); // 16.67ms per frame

struct InputConfig {
    right: Key,
    left: Key,
    up: Key,
    down: Key,
    a: Key,
    b: Key,
    select: Key,
    start: Key,
}

impl InputConfig {
    fn new() -> Self {
        Self {
            right: Key::Right,
            left: Key::Left,
            up: Key::Up,
            down: Key::Down,
            a: Key::S,
            b: Key::A,
            select: Key::Space,
            start: Key::Enter,
        }
    }
}

pub struct Emulator {
    window: Window,
    paused: bool,
    running: bool,
    cpu: CPU,
    input_config: InputConfig,
    can_change_state: bool,
    rom_path: PathBuf,
}

impl Emulator {
    pub fn new(filename: &str) -> Result<Self, Box<dyn Error>> {
        let cart = Cart::new(filename)?;

        if PRINT_CART_INFO {
            cart.print_info();
        }

        let name = format!("ZetaBoy - {}", cart.get_title());
        let (width, height) = (WIDTH, HEIGHT);
        let options = WindowOptions {
            resize: true,
            scale: minifb::Scale::X4,
            scale_mode: minifb::ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        };
        let window: Window = Window::new(&name, width, height, options).unwrap_or_else(|e| {
            panic!("Error building window: {}", e);
        });

        let mut emulator = Self {
            window,
            paused: false,
            running: true,
            cpu: CPU::new(cart),
            input_config: InputConfig::new(),
            can_change_state: true,
            rom_path: PathBuf::from(filename),
        };

        if let Err(e) = emulator.load_sram() {
            eprintln!("Failed to load SRAM: {}", e);
        }

        Ok(emulator)
    }

    pub fn run_with_audio(
        &mut self,
        mut audio_sender: AudioSampleSender,
    ) -> Result<(), Box<dyn Error>> {
        self.cpu.print_state();

        const TIME_STEP: Duration = Duration::from_micros(1_000); // 1ms
        const CYCLES_PER_STEP: u64 = CPU_FREQUENCY as u64 / 1000; // ~4194 cycles per ms

        let mut next_frame = Instant::now();
        let mut next_step = Instant::now();

        while self.running && self.window.is_open() {
            let now = Instant::now();

            // Handle input and pause
            self.handle_input();
            if self.window.is_key_down(Key::Escape) || !self.running {
                self.running = false;
                break;
            }
            if self.paused {
                std::thread::sleep(Duration::from_millis(16));
                continue;
            }

            // Emulate in fixed time steps
            while next_step <= now {
                let mut cycles_this_step = 0u64;
                while cycles_this_step < CYCLES_PER_STEP {
                    let t_cycles_taken = self.cpu.step();
                    cycles_this_step += t_cycles_taken as u64;
                    // Process audio for these cycles
                    audio_sender
                        .process_cpu_cycles(t_cycles_taken as u32, || self.get_apu_sample());
                }
                next_step += TIME_STEP;
            }

            // Render at 60Hz
            if next_frame <= now {
                self.render();
                next_frame += FRAME_DURATION;
            }

            // Sleep a bit to avoid busy-waiting
            std::thread::sleep(Duration::from_micros(100));
        }
        if let Err(e) = self.save_sram() {
            eprintln!("Failed to save SRAM: {}", e);
        }
        Ok(())
    }

    fn get_apu_sample(&self) -> (f32, f32) {
        self.cpu.bus.apu.sample_stereo()
    }

    fn render(&mut self) {
        // Render the current frame to the window
        self.window
            .update_with_buffer(&self.cpu.bus.ppu.buffer, WIDTH, HEIGHT)
            .unwrap();
    }

    fn handle_input(&mut self) {
        use JoypadButton::*;

        // Gather emulator control input
        let save = self.is_key_down(Key::F1);
        let load = self.is_key_down(Key::F2);

        // Handle save/load state
        if self.can_change_state {
            if save {
                self.save_state().unwrap();
                self.can_change_state = false;
                return;
            } else if load {
                if let Err(e) = self.load_state() {
                    eprintln!("Failed to load state: {}", e);
                }
                self.can_change_state = false;
                return;
            }
        } else {
            if !save && !load {
                self.can_change_state = true;
                return;
            }
        }

        // Gather GameBoy input
        let right = self.is_key_down(self.input_config.right);
        let left = self.is_key_down(self.input_config.left);
        let up = self.is_key_down(self.input_config.up);
        let down = self.is_key_down(self.input_config.down);
        let a = self.is_key_down(self.input_config.a);
        let b = self.is_key_down(self.input_config.b);
        let select = self.is_key_down(self.input_config.select);
        let start = self.is_key_down(self.input_config.start);

        // Apply state
        self.set_button_state(Right, right);
        self.set_button_state(Left, left);
        self.set_button_state(Up, up);
        self.set_button_state(Down, down);
        self.set_button_state(A, a);
        self.set_button_state(B, b);
        self.set_button_state(Select, select);
        self.set_button_state(Start, start);
    }

    fn is_key_down(&self, key: Key) -> bool {
        self.window.is_key_down(key)
    }

    fn set_button_state(&mut self, button: JoypadButton, state: bool) {
        self.cpu.bus.io.set_button_state(button, state);
    }

    /// Save the current emulator state to a file.
    pub fn save_state(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = &self.get_state_path();
        let state = self.cpu.clone();

        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, &state)?;

        println!("Saved state to {}", path);
        Ok(())
    }

    /// Load the emulator state from a file.
    pub fn load_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let path = &self.get_state_path();
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path, e))?;
        let reader = BufReader::new(file);

        let state: CPU = bincode::deserialize_from(reader)
            .map_err(|e| format!("Failed to deserialize {}: {}", path, e))?;
        self.cpu = state;

        println!("Loaded state from {}", path);
        Ok(())
    }

    pub fn save_sram(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cart = &self.cpu.bus.cart;
        if cart.has_battery() {
            let path = &self.get_sram_path();
            let sram = &cart.ram_data;

            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);
            writer.write_all(sram)?;

            println!("Saved SRAM to {}", path);
        }
        Ok(())
    }

    pub fn load_sram(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.cpu.bus.cart.has_battery() {
            let path = &self.get_sram_path();
            let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path, e))?;

            let mut reader = BufReader::new(file);
            reader.read_exact(&mut self.cpu.bus.cart.ram_data)?;

            println!("Loaded SRAM from {}", path);
        }
        Ok(())
    }

    /// Get the save state file path based on the ROM path.
    fn get_state_path(&self) -> String {
        let mut path = self.rom_path.clone();
        path.set_extension("sav");
        path.to_string_lossy().to_string()
    }

    /// Get the SRAM file path based on the ROM path.
    fn get_sram_path(&self) -> String {
        let mut path = self.rom_path.clone();
        path.set_extension("srm");
        path.to_string_lossy().to_string()
    }
}
