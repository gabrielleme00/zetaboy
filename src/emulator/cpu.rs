mod control_unit;
mod instructions;
pub mod memory_bus;
mod registers;

use crate::emulator::cart::Cart;
use crate::emulator::cpu::memory_bus::io_registers::REG_IF;
use crate::PRINT_STATE;
use instructions::*;
use memory_bus::*;
use registers::*;

#[derive(PartialEq)]
pub enum CpuMode {
    Normal,
    Halt,
    Stop,
    HaltBug,
    HaltDI,
    EnableIME,
}

pub struct CPU {
    pub reg: Registers,
    pub bus: MemoryBus,
    pub mode: CpuMode,
    pub ime: bool,
}

impl CPU {
    pub fn new(cart: Cart) -> Self {
        Self {
            reg: Registers::new(),
            bus: MemoryBus::new(cart),
            ime: false,
            mode: CpuMode::Normal,
        }
    }

    /// Emulates a CPU step. Returns the number of cycles taken.
    pub fn step(&mut self) -> u8 {
        use CpuMode::*;

        // Handle delayed EI: enable interrupts after the instruction following EI
        match self.mode {
            Normal => {}
            Halt | Stop => {
                self.tick4();
                if let Some(interrupt_cycles) = self.check_interrupts(self.reg.pc) {
                    return interrupt_cycles;
                }
                return 4;
            }
            HaltBug => {
                self.tick4();
                if let Some(interrupt_cycles) = self.check_interrupts(self.reg.pc) {
                    return interrupt_cycles;
                }
                return 4;
            },
            HaltDI => todo!("Implement Halt DI"),
            EnableIME => {
                self.ime = true;
                self.mode = Normal;
            }
        }

        // Fetch opcode
        let mut opcode = self.bus.read_byte(self.reg.pc);
        let prefixed = opcode == 0xCB;
        if prefixed {
            opcode = self.read_next_byte();
        }

        // Decode opcode
        let opcode_info = OpcodeInfo::from_byte(opcode, prefixed)
            .ok_or_else(|| {
                panic!(
                    "Unknown instruction at ${:04X}: {:#04X}, prefixed: {}",
                    self.reg.pc, opcode, prefixed
                );
            })
            .unwrap();

        let instruction = opcode_info.instruction;
        let (next_pc, var_cycles) = control_unit::execute(self, instruction);

        self.reg.pc = next_pc;

        if self.ime {
            if let Some(interrupt_cycles) = self.check_interrupts(self.reg.pc) {
                return interrupt_cycles;
            }
        }

        self.print_state();

        let cycles = match var_cycles {
            Some(var_cycles_taken) => var_cycles_taken,
            None => opcode_info.cycles,
        };

        for _ in 0..cycles {
            self.tick();
        }

        cycles
    }

    fn tick(&mut self) {
        // Timer
        self.bus.timer.tick(&mut self.bus.io);
        // PPU
        self.bus.ppu.tick(&mut self.bus.io);
    }

    fn tick4(&mut self) {
        for _ in 0..4 {
            self.tick();
        }
    }

    pub fn print_state(&self) {
        if !PRINT_STATE {
            return;
        }
        println!(
            "{} PCMEM:{:02X},{:02X},{:02X},{:02X}",
            self.reg,
            self.bus.read_byte(self.reg.pc),
            self.bus.read_byte(self.reg.pc.wrapping_add(1)),
            self.bus.read_byte(self.reg.pc.wrapping_add(2)),
            self.bus.read_byte(self.reg.pc.wrapping_add(3))
        );
    }

    fn check_interrupts(&mut self, next_pc: u16) -> Option<u8> {
        use CpuMode::*;

        let int_f = self.bus.get_interrupt_flags();
        let int_e = self.bus.get_interrupt_enable();

        // If no interrupts are pending, return early
        let pending = int_f & int_e;
        if pending == 0 {
            return None;
        }

        // If there are pending interrupts, always wake from HALT
        if self.mode == Halt || self.mode == Stop {
            self.mode = Normal;
        }

        // Disable further interrupts
        self.ime = false;

        for i in 0..5 {
            let mask = 1 << i;
            if pending & mask != 0 {
                // Clear the interrupt flag
                self.bus.write_byte(REG_IF, int_f & !mask);

                // Push the next PC onto the stack (where execution should resume)
                self.alu_push(next_pc);

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
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = false;
        self.reg.f.c = old_bit_0 == 1;
        new_value
    }

    /// Rotates A to the left. Old bit 7 is copied to Carry flag.
    fn alu_rlc(&mut self, value: u8) -> u8 {
        let old_bit_0 = (value & 0x80) >> 7;
        let new_value = (value << 1) | old_bit_0;
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = false;
        self.reg.f.c = old_bit_0 == 1;
        new_value
    }

    /// Rotates value to the right through Carry flag.
    /// Bit 0 is copied to Carry, previous Carry is copied to bit 7.
    fn alu_rr(&mut self, value: u8) -> u8 {
        let old_bit_0 = value & 1;
        let new_value = (value >> 1) | ((self.reg.f.c as u8) << 7);
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = false;
        self.reg.f.c = old_bit_0 == 1;
        new_value
    }

    /// Rotates A to the right. Old bit 0 is copied to Carry flag.
    fn alu_rrc(&mut self, value: u8) -> u8 {
        let old_bit_0 = value & 1;
        let new_value = (value >> 1) | (old_bit_0 << 7);
        self.reg.f.z = new_value == 0;
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

    /// Sets a specific bit in a value.
    pub fn alu_set(&mut self, bit: u8, value: u8) -> u8 {
        if bit < 8 {
            let mask = 1 << bit;
            value | mask
        } else {
            // Handle invalid bit as appropriate
            value
        }
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
