mod instructions;
mod registers;

use instructions::*;
use registers::*;

pub struct CPU {
    reg: Registers,
    halted: bool,
    _stepping: bool,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            reg: Registers::new(),
            halted: false,
            _stepping: false,
        }
    }

    /// Emulates a CPU step/cycle.
    pub fn step(&self) -> Result<(), &'static str> {
        if self.halted {
            return Ok(());
        }

        // let opcode = self.fetch_opcode();
        // self.fetch_data();
        // self.execute();

        Ok(())
    }

    /// Executes a given `instruction`
    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::ADD(target) => match target {
                ArithmeticTarget::C => {
                    self.reg.a = self.add(self.reg.c);
                }
                _ => { /* TODO: support more targets */ }
            },
            _ => { /* TODO: support more instructions */ }
        }
    }

    /// Adds `value` to the A register (accumulator).
    fn add(&mut self, value: u8) -> u8 {
        let (new_value, overflow) = self.reg.a.overflowing_add(value);
        self.reg.f.zero = new_value == 0;
        self.reg.f.sub = false;
        self.reg.f.half_carry = (self.reg.a & 0xF) + (value & 0xF) > 0xF;
        self.reg.f.carry = overflow;
        new_value
    }
}
