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
    ticks: u64,
    cycles: u64,
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
            ticks: 0,
            cycles: 0,
            cpu: CPU::new(cart),
            prev_vblank: false,
            input_config: InputConfig::new(),
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        while self.running && self.window.is_open() {
            self.handle_input();

            if self.window.is_key_down(Key::Escape) || !self.running {
                return Ok(());
            }
            if self.paused {
                continue;
            }
            self.update()?;
        }
        Ok(())
    }

    fn update(&mut self) -> Result<(), Box<dyn Error>> {
        let cycles = self.cpu.step()?;

        let bgp = self.cpu.bus.read_byte(0xFF47);
        self.cpu.bus.ppu.step(cycles, bgp, &mut self.cpu.bus.io);

        if self.cpu.bus.ppu.int != 0 {
            // Handle VBlank interrupt (bit 0)
            if self.cpu.bus.ppu.int & 0b1 != 0 {
                self.cpu.request_interrupt(0b1);
            }
            // Handle LCD STAT interrupt (bit 1)
            if self.cpu.bus.ppu.int & 0b10 != 0 {
                self.cpu.request_interrupt(0b10);
            }
            // Clear all PPU interrupt flags
            self.cpu.bus.ppu.int = 0;
        }
        let lcd_enabled = self.cpu.bus.io.read(REG_LCDC) & 0x80 != 0;
        let vblank = self.cpu.bus.ppu.is_vblank();

        // Only update buffer on the rising edge of VBlank
        if lcd_enabled && vblank && !self.prev_vblank {
            self.window
                .update_with_buffer(&self.cpu.bus.ppu.buffer, WIDTH, HEIGHT)
                .unwrap();
        }
        self.prev_vblank = vblank;

        self.cycles += cycles as u64;
        self.ticks += 1;

        Ok(())
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
