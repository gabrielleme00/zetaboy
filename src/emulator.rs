mod cart;
mod cpu;
pub mod ppu;
mod timer;

use cart::Cart;
use cpu::CPU;
use minifb::{Key, Window, WindowOptions};
use ppu::{HEIGHT, WIDTH};
use std::error::Error;

const PRINT_CART_INFO: bool = false;

pub struct Emulator {
    window: Window,
    paused: bool,
    running: bool,
    ticks: u64,
    cycles: u64,
    cpu: CPU,
    prev_vblank: bool,
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
            scale: minifb::Scale::X2,
            scale_mode: minifb::ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        };
        let window: Window = Window::new(&name, width, height, options).unwrap_or_else(|e| {
            panic!("Error building window: {}", e);
        });
        // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        Ok(Emulator {
            window,
            paused: false,
            running: true,
            ticks: 0,
            cycles: 0,
            cpu: CPU::new(cart),
            prev_vblank: false,
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.cpu.print_state();
        while self.window.is_open() {
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
        let lcd_enabled = self.cpu.bus.io.lcdc & 0x80 != 0;
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
}
