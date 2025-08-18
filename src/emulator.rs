mod cart;
mod cpu;
pub mod ppu;
mod timer;

use crate::emulator::cpu::memory_bus::io_registers::{JoypadButton, REG_LCDC};
use crate::PRINT_CART_INFO;
use cart::Cart;
use cpu::CPU;
use minifb::{Key, Window, WindowOptions};
use ppu::{HEIGHT, WIDTH};
use std::error::Error;
use std::time::{Duration, Instant};

// Add these constants at the top of the file
const GB_CPU_FREQ_HZ: u64 = 4_194_304; // Game Boy CPU frequency in Hz
const CYCLES_PER_FRAME: u64 = GB_CPU_FREQ_HZ / 60; // ~69905 cycles per frame at 60fps
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
    prev_vblank: bool,
    input_config: InputConfig,
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

        Ok(Emulator {
            window,
            paused: false,
            running: true,
            cpu: CPU::new(cart),
            prev_vblank: false,
            input_config: InputConfig::new(),
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut last_frame_time = Instant::now();
        let mut cycle_debt = 0u64;

        while self.running && self.window.is_open() {
            let frame_start = Instant::now();
            
            self.handle_input();

            if self.window.is_key_down(Key::Escape) || !self.running {
                return Ok(());
            }
            
            if self.paused {
                last_frame_time = Instant::now();
                std::thread::sleep(Duration::from_millis(16)); // Sleep while paused
                continue;
            }

            // Calculate how many cycles we should execute this frame
            let elapsed = frame_start.duration_since(last_frame_time);
            let elapsed_nanos = elapsed.as_nanos() as u64;
            let target_cycles = (elapsed_nanos * GB_CPU_FREQ_HZ) / 1_000_000_000 + cycle_debt;

            // Execute CPU cycles
            let mut executed_cycles = 0u64;
            let mut should_render = false;
            
            while executed_cycles < target_cycles {
                let cpu_cycles = self.cpu.step();
                executed_cycles += cpu_cycles as u64;
                
                // Check if we should render this frame
                if self.should_render() {
                    should_render = true;
                    break; // Break early to render
                }
            }

            // Update cycle debt
            cycle_debt = target_cycles.saturating_sub(executed_cycles);
            
            // Cap the debt to prevent spiral of doom
            if cycle_debt > CYCLES_PER_FRAME * 2 {
                cycle_debt = CYCLES_PER_FRAME;
            }

            // Always update last_frame_time, not just when rendering
            last_frame_time = frame_start;

            // Render if we hit VBlank
            if should_render {
                self.render();
            }

            // Maintain consistent frame rate
            let frame_time = frame_start.elapsed();
            if frame_time < FRAME_DURATION {
                std::thread::sleep(FRAME_DURATION - frame_time);
            }
        }
        Ok(())
    }

    fn should_render(&mut self) -> bool {
        let lcd_enabled = self.cpu.bus.io.read(REG_LCDC) & 0x80 != 0;
        let vblank = self.cpu.bus.ppu.is_vblank();

        // Only render on the rising edge of VBlank
        let should_render = lcd_enabled && vblank && !self.prev_vblank;
        self.prev_vblank = vblank;
        
        should_render
    }

    fn render(&mut self) {
        // Render the current frame to the window
        self.window
            .update_with_buffer(&self.cpu.bus.ppu.buffer, WIDTH, HEIGHT)
            .unwrap();
    }

    fn handle_input(&mut self) {
        use JoypadButton::*;

        // Gather input
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
}
