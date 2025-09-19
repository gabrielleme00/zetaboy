mod control_unit;
mod instructions;
pub mod memory_bus;
mod registers;

use crate::emulator::cart::Cart;
use crate::emulator::cpu::memory_bus::io_registers::{REG_IE, REG_IF};
use crate::PRINT_STATE;
use instructions::*;
use memory_bus::*;
use registers::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub enum CpuMode {
    Normal,
    Halt,
    Stop,
    HaltBug,
    HaltDI,
    EnableIME,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CPU {
    pub reg: Registers,
    pub bus: MemoryBus,
    pub mode: CpuMode,
    pub ime: bool,
    total_cycles: u64,
}

impl CPU {
    pub fn new(cart: Cart) -> Self {
        Self {
            reg: Registers::new(),
            bus: MemoryBus::new(cart),
            ime: false,
            mode: CpuMode::Normal,
            total_cycles: 0,
        }
    }

    /// Emulates a CPU step. Returns the number of cycles taken.
    pub fn step(&mut self) -> u64 {
        use CpuMode::*;

        let cycles_before = self.total_cycles;
        let mut pending: u8 = 0;

        // Handle delayed EI: enable interrupts after the instruction following EI
        match self.mode {
            Normal => {
                let instr = self.read_instr();
                self.run_instr(instr);
                if self.ime {
                    pending = self.get_pending_interrupts();
                }
            }
            Halt | Stop => {
                self.tick4();
                pending = self.get_pending_interrupts();
            }
            HaltBug => {
                let instr = self.read_instr();
                self.reg.pc = self.reg.pc.wrapping_sub(1);
                self.run_instr(instr);

                self.mode = Normal;
                if self.ime {
                    pending = self.get_pending_interrupts();
                }
            }
            HaltDI => {
                self.tick4();
                if self.get_pending_interrupts() != 0 {
                    self.mode = Normal;
                } else {
                    // If no interrupts are pending, just return
                    return self.total_cycles - cycles_before;
                }
            }
            EnableIME => {
                self.ime = true;
                self.mode = Normal;

                let instr = self.read_instr();
                self.run_instr(instr);

                if self.ime {
                    pending = self.get_pending_interrupts();
                }
            }
        }

        if pending != 0 {
            self.execute_interrupts(pending);
        }

        self.print_state();
        self.total_cycles - cycles_before
    }

    fn read_instr(&mut self) -> Instruction {
        let mut opcode = self.read_byte_at_pc();
        let prefixed = opcode == 0xCB;
        if prefixed {
            opcode = self.read_byte_at_pc();
        }
        OpcodeInfo::from_byte(opcode, prefixed)
            .expect("Invalid instruction")
            .instruction
    }

    fn run_instr(&mut self, instruction: Instruction) {
        control_unit::execute(self, instruction);
    }

    /// Ticks the timers for 4 T-cycles (1 M-cycle)
    fn tick4(&mut self) {
        for _ in 0..4 {
            self.bus.tick();
        }
        self.total_cycles += 4;
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

    /// Returns the currently pending interrupts (IF & IE).
    fn get_pending_interrupts(&self) -> u8 {
        let int_f = self.bus.get_interrupt_flags();
        let int_e = self.bus.get_interrupt_enable();
        int_f & int_e
    }

    fn execute_interrupts(&mut self, pending: u8) {
        // Always wake from HALT when any interrupt is pending (even if IME=0)
        if self.mode == CpuMode::Halt || self.mode == CpuMode::Stop {
            self.mode = CpuMode::Normal;
        }

        if self.ime {
            self.ime = false;

            for i in 0..5 {
                let mask = 1 << i;
                if pending & mask != 0 {
                    // Push the current upper byte of PC onto the stack
                    self.reg.sp = self.reg.sp.wrapping_sub(1);
                    self.write_byte(self.reg.sp, (self.reg.pc >> 8) as u8);

                    // If the interrupt is not enabled, cancel it's dispatch
                    if self.reg.sp == REG_IE {
                        if self.bus.read_byte(REG_IE) & mask == 0 {
                            self.reg.pc = 0x0000;
                            continue;
                        }
                    }

                    // Push the current lower byte of PC onto the stack
                    self.reg.sp = self.reg.sp.wrapping_sub(1);
                    self.write_byte(self.reg.sp, (self.reg.pc & 0xFF) as u8);

                    // Clear the interrupt flag
                    let int_f = self.bus.read_byte(REG_IF);
                    self.bus.write_byte(REG_IF, int_f & !mask);

                    // Jump to the interrupt vector
                    self.reg.pc = match i {
                        0 => 0x40, // V-Blank
                        1 => 0x48, // LCD STAT
                        2 => 0x50, // Timer
                        3 => 0x58, // Serial
                        4 => 0x60, // Joypad
                        _ => unreachable!(),
                    };

                    // Interrupt handling takes 5 M-cycles total
                    // alu_push already accounts for 2 M-cycles (8 T-cycles)
                    // We need 3 more M-cycles (12 T-cycles)
                    self.tick4();
                    self.tick4();
                    self.tick4();

                    return; // Only handle one interrupt at a time
                }
            }
        }
    }

    // Helper functions

    fn read_byte(&mut self, address: u16) -> u8 {
        self.tick4();
        self.bus.read_byte(address)
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.tick4();
        self.bus.write_byte(address, value);
    }

    fn read_word(&mut self, address: u16) -> u16 {
        let a = self.read_byte(address) as u16;
        let b = self.read_byte(address + 1) as u16;
        (b << 8) | a
    }

    fn write_word(&mut self, address: u16, value: u16) {
        self.write_byte(address, (value & 0xFF) as u8);
        self.write_byte(address + 1, (value >> 8) as u8);
    }

    /// Returns the byte pointed by PC and increments PC.
    fn read_byte_at_pc(&mut self) -> u8 {
        let result = self.read_byte(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(1);
        result
    }

    /// Returns the next 2 bytes.
    fn read_word_at_pc(&mut self) -> u16 {
        let result = self.read_word(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(2);
        result
    }

    /// Returns the byte pointed by the `HL` register
    fn read_byte_at_hl(&mut self) -> u8 {
        self.read_byte(self.reg.get_hl())
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

    /// Adds unsigned 8-bit `value` to the A register (accumulator).
    fn alu_add(&mut self, value: u8) {
        let (new_value, overflow) = self.reg.a.overflowing_add(value);

        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = (self.reg.a & 0xF) + (value & 0xF) > 0xF;
        self.reg.f.c = overflow;

        self.reg.a = new_value;
    }

    /// Adds the unsigned 8-bit `value` to the HL register pair.
    fn alu_add_hl(&mut self, value: u16) {
        let old_hl = self.reg.get_hl();
        let (new_value, overflow) = old_hl.overflowing_add(value);

        self.reg.f.n = false;
        self.reg.f.h = (old_hl & 0x0FFF) + (value & 0x0FFF) > 0x0FFF;
        self.reg.f.c = overflow;

        self.reg.set_hl(new_value);

        self.tick4();
    }

    /// Adds the signed 8-bit `value` to SP.
    fn alu_add_sp(&mut self, value: i16) {
        self.tick4();
        self.tick4();

        let sp = self.reg.sp;
        let result = (sp as i16).wrapping_add(value) as u16;

        self.reg.f.z = false;
        self.reg.f.n = false;
        self.reg.f.h = ((sp & 0xF) + ((value as u16) & 0xF)) > 0xF;
        self.reg.f.c = ((sp & 0xFF) + ((value as u16) & 0xFF)) > 0xFF;

        self.reg.sp = result;
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

    /// Take the one's complement (i.e., flip all bits) of the contents of
    /// register A and sets the N and H flags.
    fn alu_cpl(&mut self) {
        self.reg.a = !self.reg.a;
        self.reg.f.n = true;
        self.reg.f.h = true;
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

    /// Decrements 1 from the `value` and returns it. Does not update any flags.
    fn alu_dec_16(&mut self, value: u16) -> u16 {
        let result = value.wrapping_sub(1);
        self.tick4();
        result
    }

    /// Increments 1 to the `value` and returns it. Updates flags Z, N and H.
    fn alu_inc(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_add(1);
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = (value & 0x0F) == 0x0F; // Half-carry when lower nibble overflows
        new_value
    }

    /// Increments 1 to the `value` and returns it. Does not update any flags.
    fn alu_inc_16(&mut self, value: u16) -> u16 {
        let result = value.wrapping_add(1);
        self.tick4();
        result
    }

    /// Points PC to the specified address.
    fn alu_jp(&mut self, address: u16) {
        self.reg.pc = address;
        self.tick4();
    }

    /// Performs a relative jump by adding the signed immediate value to the current PC.
    /// The offset is a signed 8-bit value, and the PC should point to the instruction after JR.
    fn alu_jr(&mut self, offset: i16) {
        let target_addr = self.reg.pc.wrapping_add(offset as u16);
        self.alu_jp(target_addr);
    }

    /// Rotates A to the left through Carry flag.
    fn alu_rl(&mut self, value: u8) -> u8 {
        let old_bit_0 = (value & 0x80) >> 7;
        let new_value = (value << 1) | (self.reg.f.c as u8);

        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = false;
        self.reg.f.c = old_bit_0 == 1;

        self.tick4();

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

        self.tick4();

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

        self.tick4();

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

        self.tick4();

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
        let lsb = self.read_byte(self.reg.sp) as u16;
        self.reg.sp = self.reg.sp.wrapping_add(1);

        let msb = self.read_byte(self.reg.sp) as u16;
        self.reg.sp = self.reg.sp.wrapping_add(1);

        (msb << 8) | lsb
    }

    /// Pushes a `value` to the top of the stack.
    fn alu_push(&mut self, value: u16) {
        self.tick4();

        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.write_byte(self.reg.sp, (value >> 8) as u8);

        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.write_byte(self.reg.sp, (value & 0xFF) as u8);
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

        self.tick4();

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

        self.tick4();

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

        self.tick4();

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

        self.tick4();

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
        self.reg.f.h = true;

        self.tick4();
    }

    /// Restarts the CPU by pushing the current program counter to the stack
    /// and setting it to a new value.
    pub fn alu_rst(&mut self, value: u16) {
        self.alu_push(self.reg.pc);
        self.reg.pc = value;
        self.tick4();
    }
}
