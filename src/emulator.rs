pub mod apu;
pub mod cart;
pub mod cpu;
pub mod joypad;
pub mod ppu;
pub mod serial;
pub mod timer;

use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::time::Instant;

use crate::PRINT_CART_INFO;
use crate::emulator::joypad::JoypadButton;
use cart::Cart;
use cpu::CPU;

pub const CPU_FREQUENCY: u32 = 4194304;

pub struct InputState {
    // Gameboy buttons
    pub right: bool,
    pub left: bool,
    pub up: bool,
    pub down: bool,
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
    // Emulator controls
    pub save: bool,
    pub load: bool,
    // Internal state for save/load handling
    can_change_state: bool,
}

impl InputState {
    fn new() -> Self {
        Self {
            right: false,
            left: false,
            up: false,
            down: false,
            a: false,
            b: false,
            select: false,
            start: false,
            save: false,
            load: false,
            can_change_state: true,
        }
    }
}

pub struct Emulator {
    pub running: bool,
    pub cpu: CPU,
    pub input_state: InputState,
    pub rom_path: PathBuf,
    pub next_step: Instant,
}

impl Emulator {
    pub fn new(filename: &str, force_dmg: bool) -> Result<Self, Box<dyn Error>> {
        let cart = Cart::new(filename)?;

        if PRINT_CART_INFO {
            cart.print_info();
        }

        let mut emulator = Self {
            running: true,
            cpu: CPU::new(cart, force_dmg),
            input_state: InputState::new(),
            rom_path: PathBuf::from(filename),
            next_step: Instant::now(),
        };

        if let Err(e) = emulator.load_sram() {
            eprintln!("Failed to load SRAM: {}", e);
        }

        Ok(emulator)
    }

    pub fn handle_input(&mut self) {
        use JoypadButton::*;

        // Handle save/load state
        if self.input_state.can_change_state {
            if self.input_state.save {
                match self.save_state() {
                    Ok(path) => println!("Saved state to {}", path),
                    Err(e) => eprintln!("Failed to save state: {}", e),
                }
                self.input_state.can_change_state = false;
                return;
            } else if self.input_state.load {
                match self.load_state() {
                    Ok(path) => println!("Loaded state from {}", path),
                    Err(e) => eprintln!("Failed to load state: {}", e),
                }
                self.input_state.can_change_state = false;
                return;
            }
        } else {
            if !self.input_state.save && !self.input_state.load {
                self.input_state.can_change_state = true;
                return;
            }
        }

        // Apply GameBoy input states directly
        self.set_button_state(Right, self.input_state.right);
        self.set_button_state(Left, self.input_state.left);
        self.set_button_state(Up, self.input_state.up);
        self.set_button_state(Down, self.input_state.down);
        self.set_button_state(A, self.input_state.a);
        self.set_button_state(B, self.input_state.b);
        self.set_button_state(Select, self.input_state.select);
        self.set_button_state(Start, self.input_state.start);
    }

    fn set_button_state(&mut self, button: JoypadButton, state: bool) {
        self.cpu.bus.set_button_state(button, state);
    }

    /// Save the current emulator state to a file.
    ///
    /// If the state was successfully saved, returns the path to the save file.
    /// Otherwise, returns an error.
    pub fn save_state(&self) -> Result<String, Box<dyn std::error::Error>> {
        let path = &self.get_state_path();
        let state = self.cpu.clone();

        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, &state)?;

        Ok(path.to_string())
    }

    /// Load the emulator state from a file.
    ///
    /// If the state was successfully saved, returns the path to the save file.
    /// Otherwise, returns an error.
    pub fn load_state(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let path = &self.get_state_path();
        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path, e))?;
        let reader = BufReader::new(file);

        let state: CPU = bincode::deserialize_from(reader)
            .map_err(|e| format!("Failed to deserialize {}: {}", path, e))?;
        self.cpu = state;

        Ok(path.to_string())
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
