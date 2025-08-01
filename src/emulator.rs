pub mod ppu;
mod cart;
mod cpu;

use cart::Cart;
use cpu::CPU;
use minifb::{Key, Window, WindowOptions};
use std::error::Error;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

pub struct Emulator {
    window: Window,
    paused: bool,
    running: bool,
    ticks: u64,
    cycles: u64,
    cpu: CPU,
}

impl Emulator {
    pub fn new(filename: &str) -> Result<Self, Box<dyn Error>> {
        let cart = Cart::new(filename)?;
        cart.print_info();

        let name = format!("ZetaBoy - {}", cart.get_title());
        let (width, height) = (WIDTH, HEIGHT);
        let options = WindowOptions {
            scale: minifb::Scale::X2,
            ..WindowOptions::default()
        };
        let mut window: Window = Window::new(&name, width, height, options)
            .unwrap_or_else(|e| {
                panic!("Error building window: {}", e);
            });
        limit_to_60fps(&mut window);

        Ok(Emulator {
            window,
            paused: false,
            running: true,
            ticks: 0,
            cycles: 0,
            cpu: CPU::new(&cart.rom_data),
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
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
        let bgp_value = self.cpu.bus.read_byte(0xFF47);
        self.cpu.bus.ppu.step(cycles, bgp_value, &mut self.cpu.bus.io);
        if self.cpu.bus.ppu.interrupt != 0 {
            self.cpu.request_interrupt(self.cpu.bus.ppu.interrupt);
            self.cpu.bus.ppu.interrupt = 0;
        }
        let lcd_enabled = self.cpu.bus.io.lcdc & 0x80 != 0;
        let vblank = self.cpu.bus.ppu.mode == ppu::PPUMode::VBlank;
        if lcd_enabled && vblank {
            self.window.update_with_buffer(&self.cpu.bus.ppu.buffer, WIDTH, HEIGHT).unwrap();
        }
        self.cycles += cycles as u64;
        self.ticks += 1;
        Ok(())
    }
}

fn limit_to_60fps(window: &mut Window) {
    let fps60 = std::time::Duration::from_micros(16600);
    window.limit_update_rate(Some(fps60));
}
