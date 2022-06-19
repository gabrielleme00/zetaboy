mod cart;
mod cpu;

use cart::Cart;
use cpu::CPU;
use piston_window::*;
use std::error::Error;

const SCALE_FACTOR: u32 = 4;
const NATIVE_SIZE: [u32; 2] = [160, 144];
const WINDOW_SIZE: [u32; 2] = [NATIVE_SIZE[0] * SCALE_FACTOR, NATIVE_SIZE[1] * SCALE_FACTOR];
const WINDOW_TITLE: &str = "ZetaBoy - Game Boy Emulator";

pub struct Emulator {
    pub window: PistonWindow,
    pub paused: bool,
    pub running: bool,
    ticks: u64,
    cpu: CPU,
    _cart: Cart,
}

impl Emulator {
    pub fn new(filename: &str) -> Result<Self, Box<dyn Error>> {
        let mut window: PistonWindow = WindowSettings::new(WINDOW_TITLE, WINDOW_SIZE)
            .exit_on_esc(true)
            .resizable(false)
            .build()
            .unwrap();
        window.set_ups(60);

        let cart =  Cart::new(filename)?;
        cart.print_info();

        Ok(Emulator {
            window,
            paused: false,
            running: true,
            ticks: 0,
            cpu: CPU::new(),
            _cart: cart,
        })
    }

    pub fn render(&mut self, e: &Event) {
        self.window.draw_2d(e, |_context, graphics, _device| {
            clear([0.; 4], graphics);
        });
    }

    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        self.cpu.step()?;
        self.ticks += 1;
        Ok(())
    }
}
