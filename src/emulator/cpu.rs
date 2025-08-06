mod control_unit;
mod instructions;
pub mod memory_bus;
mod registers;

use instructions::*;
use memory_bus::*;
use registers::*;

pub struct CPU {
    reg: Registers,
    pub bus: MemoryBus,
    halted: bool,
    stopped: bool,
    pub ime: bool,
    pub ei_delay: bool, // EI instruction has delayed effect
}

impl CPU {
    pub fn new(cart_data: &Vec<u8>) -> Self {
        Self {
            reg: Registers::new(),
            bus: MemoryBus::new(cart_data),
            halted: false,
            stopped: false,
            ime: false,
            ei_delay: false,
        }
    }

    /// Emulates a CPU step. Returns the number of cycles taken.
    pub fn step(&mut self) -> Result<u8, &'static str> {
        // Handle delayed EI: enable interrupts after the instruction following EI
        if self.ei_delay {
            self.ime = true;
            self.ei_delay = false;
        }

        // Check if CPU is in STOP mode
        if self.stopped {
            // STOP mode can only be exited by specific events (like joypad input)
            // For now, we'll just return 4 cycles and stay stopped
            // TODO: Implement proper STOP wake-up conditions
            return Ok(4);
        }

        if self.ime && !self.halted {
            if let Some(cycles) = self.handle_interrupts() {
                return Ok(cycles);
            }
        }

        if self.halted {
            // Check if any interrupt is pending to wake up from HALT
            if self.bus.io.int_enable & self.bus.io.int_flag != 0 {
                self.halted = false;
                // If IME is disabled, there's a halt bug where PC doesn't increment
                if !self.ime {
                    // HALT bug: when waking from HALT with IME=0, the next instruction is executed twice
                    // For now, we'll just continue normally but this should be properly implemented
                    panic!("HALT bug: PC should not increment when IME=0");
                }
            } else {
                return Ok(4);
            }
        }

        let mut opcode = self.bus.read_byte(self.reg.pc);
        let prefixed = opcode == 0xCB;
        if prefixed {
            opcode = self.read_next_byte();
        }

        let instruction = Instruction::from_byte(opcode, prefixed).unwrap_or_else(|| {
            panic!(
                "Unknown instruction at ${:04X}: {:#04X}, prefixed: {}",
                self.reg.pc, opcode, prefixed
            );
        });

        self.reg.pc = control_unit::execute(self, instruction);
        let cycles = instruction.cycles();

        // Step the timer and check for timer interrupt
        let timer_interrupt = self.bus.timer.step(
            cycles,
            &mut self.bus.io.div,
            &mut self.bus.io.tima,
            self.bus.io.tma,
            self.bus.io.tac,
        );

        if timer_interrupt {
            self.request_interrupt(0b100); // Timer interrupt bit
        }

        Ok(cycles)
    }

    fn handle_interrupts(&mut self) -> Option<u8> {
        let pending = self.bus.io.int_enable & self.bus.io.int_flag;
        if pending == 0 {
            return None;
        }

        self.halted = false; // Exit HALT state if an interrupt is being handled
        self.ime = false; // Disable further interrupts

        for i in 0..5 {
            let mask = 1 << i;
            if pending & mask != 0 {
                // Clear the interrupt flag
                self.bus.io.int_flag &= !mask;

                // Push the current PC onto the stack
                self.alu_push(self.reg.pc);

                // Jump to the interrupt vector
                self.reg.pc = match i {
                    0 => 0x40, // V-Blank
                    1 => 0x48, // LCD STAT
                    2 => 0x50, // Timer
                    3 => 0x58, // Serial
                    4 => 0x60, // Joypad
                    _ => unreachable!(),
                };

                return Some(20); // Interrupt handling takes 20 cycles
            }
        }

        None
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
            FlagCondition::Zero => self.reg.f.z == true,
            FlagCondition::NotZero => self.reg.f.z == false,
            FlagCondition::Carry => self.reg.f.c == true,
            FlagCondition::NotCarry => self.reg.f.c == false,
        }
    }

    /// Evaluates the jump condition and returns a boolean result.
    fn test_jump_condition(&self, test: Option<FlagCondition>) -> bool {
        match test {
            None => true,
            Some(fc) => self.test_flag_condition(fc),
        }
    }

    pub fn set_stopped(&mut self, stopped: bool) {
        if stopped {
            self.halted = false; // Stop state cannot be combined with HALT
        }
        self.stopped = stopped;
    }

    // --- ALU ---
    // TODO: create a separate module for the ALU

    /// Adds `value` as C flag value to the A register (accumulator).
    fn alu_adc(&mut self, value: u8) {
        let a = self.reg.a;
        let c = self.reg.f.c as u8;
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
    /// Updates flags as if subtraction occurred, but does not modify A.
    fn alu_cp(&mut self, value: u8) {
        let a = self.reg.a;
        let result = a.wrapping_sub(value);
        self.reg.f.z = result == 0;
        self.reg.f.n = true;
        self.reg.f.h = (a & 0xF) < (value & 0xF);
        self.reg.f.c = (a as u16) < (value as u16);
    }

    /// Adjusts the A register (accumulator) to a binary-coded decimal (BCD)
    /// number after BCD addition and subtraction operations.
    fn alu_daa(&mut self) {
        let mut a = self.reg.a;
        let mut correction = 0;
        let mut set_carry = false;

        if !self.reg.f.n {
            // After addition
            if self.reg.f.h || (a & 0x0F) > 0x09 {
                correction |= 0x06;
            }
            if self.reg.f.c || a > 0x99 {
                correction |= 0x60;
                set_carry = true;
            }
            a = a.wrapping_add(correction);
        } else {
            // After subtraction
            if self.reg.f.h {
                correction |= 0x06;
            }
            if self.reg.f.c {
                correction |= 0x60;
            }
            a = a.wrapping_sub(correction);
        }

        self.reg.f.z = a == 0;
        self.reg.f.h = false;
        if set_carry {
            self.reg.f.c = true;
        }
        self.reg.a = a;
    }

    /// Decrements 1 from the `value` and returns it. Updates flags Z, N and H.
    fn alu_dec(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_sub(1);
        self.reg.f.z = new_value == 0;
        self.reg.f.n = true;
        self.reg.f.h = (value & 0x0F) == 0;
        new_value
    }

    /// Increments 1 from the `value` and returns it. Updates flags Z, N and H.
    fn alu_inc(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_add(1);
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = (value & 0x0F) == 0x0F; // Half-carry when lower nibble overflows
        new_value
    }

    /// Performs a relative jump by adding the signed immediate value to the current PC.
    /// The offset is a signed 8-bit value, and the PC should point to the instruction after JR.
    fn alu_jr(&mut self) -> u16 {
        let offset = self.read_next_byte() as i8 as i16;
        let next_pc = self.reg.pc.wrapping_add(2); // PC after JR opcode and operand
        next_pc.wrapping_add(offset as u16)
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

    /// Performs a bitwise OR operation with the A register (accumulator).
    fn alu_or(&mut self, value: u8) {
        let new_value = self.reg.a | value;
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = false;
        self.reg.f.c = false;
        self.reg.a = new_value;
    }

    /// Performs a bitwise AND operation with the A register (accumulator).
    fn alu_and(&mut self, value: u8) {
        let new_value = self.reg.a & value;
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = true;
        self.reg.f.c = false;
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
    fn alu_push(&mut self, value: u16) {
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.bus.write_byte(self.reg.sp, (value >> 8) as u8);

        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.bus.write_byte(self.reg.sp, (value & 0xFF) as u8);
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

    pub fn request_interrupt(&mut self, interrupt: u8) {
        self.bus.io.int_flag |= interrupt;
    }

    /// Sets the CPU halted state
    pub fn set_halted(&mut self, halted: bool) {
        self.halted = halted;
    }

    /// Shifts `value` left into Carry. LSB of `value` set to 0.
    ///
    /// Updates flags Z, N, H and C.
    pub fn alu_sla(&mut self, value: u8) -> u8 {
        let old_bit_7 = (value & 0x80) >> 7;
        let new_value = value << 1;
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = false;
        self.reg.f.c = old_bit_7 == 1;
        new_value
    }

    /// Shifts `value` right into Carry. MSB of `value` set to 0.
    ///
    /// Updates flags Z, N, H and C.
    pub fn alu_sra(&mut self, value: u8) -> u8 {
        let old_bit_0 = value & 1;
        let msb = value & 0x80; // Preserve the most significant bit
        let new_value = (value >> 1) | msb;
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = false;
        self.reg.f.c = old_bit_0 == 1;
        new_value
    }

    /// Shifts `value` right into Carry. MSB of `value` set to 0.
    ///
    /// Updates flags Z, N, H and C.
    pub fn alu_srl(&mut self, value: u8) -> u8 {
        let old_bit_0 = value & 1;
        let new_value = value >> 1;
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = false;
        self.reg.f.c = old_bit_0 == 1;
        new_value
    }

    /// Swaps the nibbles of a byte.
    ///
    /// Updates flags Z, N, H and C.
    pub fn alu_swap(&mut self, value: u8) -> u8 {
        let new_value = ((value & 0xF0) >> 4) | ((value & 0x0F) << 4);
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = false;
        self.reg.f.c = false;
        new_value
    }

    /// Performs a bitwise AND operation with the A register (accumulator) and sets flags.
    ///
    /// Updates flags Z, N, and H.
    pub fn alu_bit(&mut self, bit: u8, value: u8) {
        if bit < 8 {
            let mask = 1 << bit;
            self.reg.f.z = (value & mask) == 0;
        } else {
            self.reg.f.z = true; // or handle as appropriate
        }
        self.reg.f.n = false;
        self.reg.f.h = true; // Half-carry is always set for BIT
        // Carry flag is not affected by BIT
    }
}
