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
    buffer: Vec<u32>,
    paused: bool,
    running: bool,
    ticks: u64,
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
            buffer: vec![0; WIDTH * HEIGHT],
            paused: false,
            running: true,
            ticks: 0,
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
            self.render();
            self.update()?;
        }
        Ok(())
    }

    fn render(&mut self) {
        for i in self.buffer.iter_mut() {
            *i = 0;
        }

        self.window.update_with_buffer(&self.buffer, WIDTH, HEIGHT).unwrap();
    }

    fn update(&mut self) -> Result<(), Box<dyn Error>> {
        self.cpu.step()?;
        self.ticks += 1;
        Ok(())
    }
}

fn limit_to_60fps(window: &mut Window) {
    let fps60 = std::time::Duration::from_micros(16600);
    window.limit_update_rate(Some(fps60));
}
