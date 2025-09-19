pub mod apu;
pub mod cart;
pub mod cpu;
pub mod ppu;
pub mod timer;

use winit::keyboard::KeyCode;

use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::time::{Instant};

use crate::emulator::cpu::memory_bus::io_registers::JoypadButton;
use crate::{PRINT_CART_INFO};
use cart::Cart;
use cpu::CPU;

pub const CPU_FREQUENCY: u32 = 4194304;

pub struct InputConfig {
    right: KeyCode,
    left: KeyCode,
    up: KeyCode,
    down: KeyCode,
    a: KeyCode,
    b: KeyCode,
    select: KeyCode,
    start: KeyCode,
}

impl InputConfig {
    fn new() -> Self {
        Self {
            right: KeyCode::ArrowRight,
            left: KeyCode::ArrowLeft,
            up: KeyCode::ArrowUp,
            down: KeyCode::ArrowDown,
            a: KeyCode::KeyS,
            b: KeyCode::KeyA,
            select: KeyCode::Space,
            start: KeyCode::Enter,
        }
    }
}

pub struct InputState {
    keys_pressed: HashSet<KeyCode>,
    can_change_state: bool,
}

impl InputState {
    fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            can_change_state: true,
        }
    }

    fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn set_key_state(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            self.keys_pressed.insert(key);
        } else {
            self.keys_pressed.remove(&key);
        }
    }
}

pub struct Emulator {
    pub paused: bool,
    pub running: bool,
    pub cpu: CPU,
    pub input_config: InputConfig,
    pub input_state: InputState,
    pub rom_path: PathBuf,
    pub next_frame: Instant,
    pub next_step: Instant,
}

impl Emulator {
    pub fn new(filename: &str) -> Result<Self, Box<dyn Error>> {
        let cart = Cart::new(filename)?;

        if PRINT_CART_INFO {
            cart.print_info();
        }

        let mut emulator = Self {
            paused: false,
            running: true,
            cpu: CPU::new(cart),
            input_config: InputConfig::new(),
            input_state: InputState::new(),
            rom_path: PathBuf::from(filename),
            next_frame: Instant::now(),
            next_step: Instant::now(),
        };

        if let Err(e) = emulator.load_sram() {
            eprintln!("Failed to load SRAM: {}", e);
        }

        Ok(emulator)
    }

    pub fn handle_input(&mut self) {
        use JoypadButton::*;

        // Gather emulator control input
        let save = self.input_state.is_key_down(KeyCode::F1);
        let load = self.input_state.is_key_down(KeyCode::F2);

        // Handle save/load state
        if self.input_state.can_change_state {
            if save {
                self.save_state().unwrap();
                self.input_state.can_change_state = false;
                return;
            } else if load {
                if let Err(e) = self.load_state() {
                    eprintln!("Failed to load state: {}", e);
                }
                self.input_state.can_change_state = false;
                return;
            }
        } else {
            if !save && !load {
                self.input_state.can_change_state = true;
                return;
            }
        }

        // Gather GameBoy input
        let right = self.input_state.is_key_down(self.input_config.right);
        let left = self.input_state.is_key_down(self.input_config.left);
        let up = self.input_state.is_key_down(self.input_config.up);
        let down = self.input_state.is_key_down(self.input_config.down);
        let a = self.input_state.is_key_down(self.input_config.a);
        let b = self.input_state.is_key_down(self.input_config.b);
        let select = self.input_state.is_key_down(self.input_config.select);
        let start = self.input_state.is_key_down(self.input_config.start);

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
