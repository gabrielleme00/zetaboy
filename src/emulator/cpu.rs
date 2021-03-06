mod bus;
mod control_unit;
mod instructions;
mod registers;

use bus::*;
use instructions::*;
use registers::*;

pub struct CPU {
    reg: Registers,
    bus: MemoryBus,
    halted: bool,
    ei: bool,
    _stepping: bool,
}

impl CPU {
    pub fn new(cart_data: &Vec<u8>) -> Self {
        Self {
            reg: Registers::new(),
            bus: MemoryBus::new(cart_data),
            halted: false,
            ei: true,
            _stepping: false,
        }
    }

    /// Emulates a CPU step/cycle.
    pub fn step(&mut self) -> Result<(), &'static str> {
        if self.halted {
            return Ok(());
        }

        let mut opcode = self.bus.read_byte(self.reg.pc);
        let prefixed = opcode == 0xCB;
        if prefixed {
            opcode = self.read_next_byte();
        }
        let next_pc = if let Some(instruction) = Instruction::from_byte(opcode, prefixed) {
            if self.reg.pc == 0x237 {
                let description = format!("0x{}{:02X}", if prefixed { "cb" } else { "" }, opcode);
                println!("Executing: [{:#04X}] -> {}", self.reg.pc, description);
                println!("A: {}", self.reg.a);
            }
            control_unit::execute(self, instruction)
        } else {
            let description = format!("0x{}{:02X}", if prefixed { "cb" } else { "" }, opcode);
            println!(
                "Unknown instruction found for: [{:#04X}] -> {}",
                self.reg.pc, description
            );
            return Err("Unknown instruction");
        };

        self.reg.pc = next_pc;

        Ok(())
    }

    // Helper functions

    /// Returns the next 1 byte.
    fn read_next_byte(&self) -> u8 {
        self.bus.read_byte(self.reg.pc + 1)
    }

    /// Returns the next 2 bytes.
    fn read_next_word(&self) -> u16 {
        self.bus.read_word(self.reg.pc + 1)
    }

    /// Returns the byte pointed by the `HL` register
    fn read_byte_hl(&self) -> u8 {
        self.bus.read_byte(self.reg.get_hl())
    }

    /// Evaluates the flag condition and returns a boolean result.
    fn test_flag_condition(&self, test: FlagCondition) -> bool {
        match test {
            FlagCondition::Zero => self.reg.f.z,
            FlagCondition::NotZero => !self.reg.f.z,
            FlagCondition::Carry => self.reg.f.c,
            FlagCondition::NotCarry => !self.reg.f.c,
        }
    }

    /// Evaluates the jump condition and returns a boolean result.
    fn test_jump_condition(&self, test: JumpCondition) -> bool {
        match test {
            JumpCondition::Always => true,
            JumpCondition::Flag(fc) => self.test_flag_condition(fc),
        }
    }

    // --- ALU ---
    // TODO: create a separate module for the ALU

    /// Adds `value` as C flag value to the A register (accumulator).
    fn alu_adc(&mut self, value: u8) {
        let a = self.reg.a;
        let c = self.reg.c as u8;
        let new_value = a.wrapping_add(value).wrapping_add(c);
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = (a & 0xF) + (value & 0xF) + (c & 0xF) > 0xF;
        self.reg.f.c = (a as u16) + (value as u16) + (c as u16) > 0xFF;
        self.reg.a = new_value;
    }

    /// Adds `value` to the A register (accumulator).
    fn alu_add(&mut self, value: u8) {
        let (new_value, overflow) = self.reg.a.overflowing_add(value);
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = (self.reg.a & 0xF) + (value & 0xF) > 0xF;
        self.reg.f.c = overflow;
        self.reg.a = new_value;
    }

    /// Adds `value` to the HL register pair.
    fn alu_add_hl(&mut self, value: u16) {
        let old_hl = self.reg.get_hl();
        let (new_value, overflow) = old_hl.overflowing_add(value);
        self.reg.f.n = false;
        self.reg.f.h = (old_hl & 0x0FFF) + (value & 0x0FFF) > 0x0FFF;
        self.reg.f.c = overflow;
        self.reg.set_hl(new_value);
    }

    /// Compares register A and the given `value` by calculating: A - `value`.
    fn alu_cp(&mut self, value: u8) {
        let a = self.reg.a;
        self.alu_sub(value); 
        self.reg.a = a;
    }

    /// Decrements 1 from the `value` and returns it. Updates flags Z, N and H.
    fn alu_dec(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_sub(1);
        self.reg.f.z = new_value == 0;
        self.reg.f.n = true;
        self.reg.f.h = value.trailing_zeros() >= 4;
        new_value
    }

    /// Increments 1 from the `value` and returns it. Updates flags Z, N and H.
    fn alu_inc(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_add(1);
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = value.trailing_zeros() >= 4;
        new_value
    }

    /// Adds the immediate next byte value to the current address and jumps
    /// to it.
    fn alu_jr(&mut self) -> u16 {
        let pc = self.reg.pc as i32;
        let value = (self.read_next_byte() as i8) as i32;
        (pc + value + 2) as u16
    }

    /// Rotates A to the left through Carry flag.
    fn alu_rl(&mut self, value: u8) -> u8 {
        let old_bit_0 = (value & 0x80) >> 7;
        let new_value = (value << 1) | (self.reg.f.c as u8);
        self.reg.f.z = false;
        self.reg.f.n = false;
        self.reg.f.h = false;
        self.reg.f.c = old_bit_0 == 1;
        new_value
    }

    /// Rotates A to the left. Old bit 7 is copied to Carry flag.
    fn alu_rlc(&mut self, value: u8) -> u8 {
        let old_bit_0 = (value & 0x80) >> 7;
        let new_value = (value << 1) | old_bit_0;
        self.reg.f.z = false;
        self.reg.f.n = false;
        self.reg.f.h = false;
        self.reg.f.c = old_bit_0 == 1;
        new_value
    }

    /// Rotates A to the right through Carry flag.
    fn alu_rr(&mut self, value: u8) -> u8 {
        let old_bit_0 = value & 1;
        let new_value = (value >> 1) | ((self.reg.f.c as u8) << 7);
        self.reg.f.z = false;
        self.reg.f.n = false;
        self.reg.f.h = false;
        self.reg.f.c = old_bit_0 == 1;
        new_value
    }

    /// Rotates A to the right. Old bit 0 is copied to Carry flag.
    fn alu_rrc(&mut self, value: u8) -> u8 {
        let old_bit_0 = value & 1;
        let new_value = (value >> 1) | (old_bit_0 << 7);
        self.reg.f.z = false;
        self.reg.f.n = false;
        self.reg.f.h = false;
        self.reg.f.c = old_bit_0 == 1;
        new_value
    }

    /// Subtracts `value` and C flag value from the A register (accumulator).
    fn alu_sbc(&mut self, value: u8) {
        let a = self.reg.a;
        let c = self.reg.f.c as u8;
        let new_value = a.wrapping_sub(value).wrapping_sub(c);
        self.reg.f.z = new_value == 0;
        self.reg.f.n = true;
        self.reg.f.h = (a & 0xF) < (value & 0xF) + c;
        self.reg.f.c = (a as u16) < (value as u16) + (c as u16);
        self.reg.a = new_value;
    }

    /// Subtracts `value` from the A register (accumulator).
    fn alu_sub(&mut self, value: u8) {
        let old_a = self.reg.a;
        let new_a = self.reg.a.wrapping_sub(value);
        self.reg.f.z = new_a == 0;
        self.reg.f.n = true;
        self.reg.f.h = (old_a & 0xF) < (value & 0xF);
        self.reg.f.c = (old_a as u16) < (value as u16);
        self.reg.a = new_a;
    }

    fn alu_or(&mut self, value: u8) {
        let new_value = self.reg.a | value;
        self.reg.f.z = false;
        self.reg.f.n = false;
        self.reg.f.h = false;
        self.reg.f.c = new_value == 0;
        self.reg.a = new_value;
    }

    /// Pops the last value from the stack.
    fn alu_pop(&mut self) -> u16 {
        let lsb = self.bus.read_byte(self.reg.sp) as u16;
        self.reg.sp = self.reg.sp.wrapping_add(1);

        let msb = self.bus.read_byte(self.reg.sp) as u16;
        self.reg.sp = self.reg.sp.wrapping_add(1);

        (msb << 8) | lsb
    }

    /// Pushes a `value` to the top of the stack.
    fn alu_push(&mut self, value: u16) -> u16 {
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.bus.write_byte(self.reg.sp, (value >> 8) as u8);

        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.bus.write_byte(self.reg.sp, (value & 0xFF) as u8);

        self.reg.pc.wrapping_add(1)
    }

    /// XORs `value` to the A register (accumulator).
    fn alu_xor(&mut self, value: u8) {
        let new_value = self.reg.a ^ value;
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = false;
        self.reg.f.c = false;
        self.reg.a = new_value;
    }
}
