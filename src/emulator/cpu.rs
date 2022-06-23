mod bus;
mod instructions;
mod registers;
mod control_unit;

use core::panic;

use bus::*;
use instructions::*;
use registers::*;

pub struct CPU {
    reg: Registers,
    bus: MemoryBus,
    pc: u16, // Program Counter
    sp: u16, // Stack Pointer
    halted: bool,
    _stepping: bool,
}

impl CPU {
    pub fn new(cart_data: &Vec<u8>) -> Self {
        Self {
            reg: Registers::new(),
            bus: MemoryBus::new(cart_data),
            pc: 0x100,
            sp: 0,
            halted: false,
            _stepping: false,
        }
    }

    /// Emulates a CPU step/cycle.
    pub fn step(&mut self) -> Result<(), &'static str> {
        if self.halted {
            return Ok(());
        }

        let mut opcode = self.bus.read_byte(self.pc);
        let prefixed = opcode == 0xCB;
        if prefixed {
            opcode = self.read_next_byte();
        }
        let next_pc = if let Some(instruction) = Instruction::from_byte(opcode, prefixed) {
            let description = format!("0x{}{:02X}", if prefixed { "cb" } else { "" }, opcode);
            println!("Executing: [{:#04X}] -> {}", self.pc, description);
            self.execute(instruction)
        } else {
            let description = format!("0x{}{:02X}", if prefixed { "cb" } else { "" }, opcode);
            println!(
                "Unknown instruction found for: [{:#04X}] -> {}",
                self.pc, description
            );
            return Err("Unknown instruction");
        };

        self.pc = next_pc;

        Ok(())
    }

    // Helper functions

    /// Returns the next 1 byte.
    fn read_next_byte(&self) -> u8 {
        self.bus.read_byte(self.pc + 1)
    }

    /// Returns the next 2 bytes.
    fn read_next_word(&self) -> u16 {
        self.bus.read_word(self.pc + 1)
    }

    /// Returns the byte pointed by the `HL` register
    fn read_byte_hl(&self) -> u8 {
        self.bus.read_byte(self.reg.get_hl())
    }

    /// Evaluates the jump condition and returns a boolean result.
    fn test_jump_condition(&self, test: JumpTest) -> bool {
        match test {
            JumpTest::Always => true,
            JumpTest::Zero => self.reg.f.z,
            JumpTest::NotZero => !self.reg.f.z,
            JumpTest::Carry => self.reg.f.c,
            JumpTest::NotCarry => !self.reg.f.c,
        }
    }

    /// Executes a given `instruction`
    fn execute(&mut self, instruction: Instruction) -> u16 {
        use Instruction::*;

        match instruction {
            ADD(target) => control_unit::add(self, target),
            CALL(test) => self.alu_call(test),
            DEC(target) => control_unit::dec(self, target),
            HALT => self.pc,
            JP(test) => self.alu_jump(test),
            JPHL => self.reg.get_hl(),
            JR => self.alu_jr(),
            JRIF(condition) => self.alu_jr_if(condition),
            LD(load_type) => self.alu_ld(load_type),
            NOP => self.pc.wrapping_add(1),
            POP(target) => control_unit::pop(self, target),
            PUSH(target) => control_unit::push(self, target),
            RET(test) => self.alu_ret(test),
            XOR(target) => control_unit::xor(self, target),
            _ => self.pc, /* TODO: support more instructions */
        }
    }

    // --- ALU ---

    // Branch operations

    /// Jumps to the address given by the next 2 bytes if the condition is met.
    fn alu_jump(&self, test: JumpTest) -> u16 {
        if self.test_jump_condition(test) {
            // Game Boy is little endian so read pc + 2 as most significant byte
            // and pc + 1 as least significant byte
            let least_significant_byte = self.bus.read_byte(self.pc + 1) as u16;
            let most_significant_byte = self.bus.read_byte(self.pc + 2) as u16;
            (most_significant_byte << 8) | least_significant_byte
        } else {
            // Jump instructions are always 3 bytes wide
            self.pc.wrapping_add(3)
        }
    }

    fn alu_call(&mut self, test: JumpTest) -> u16 {
        let next_pc = self.pc.wrapping_add(3);
        if self.test_jump_condition(test) {
            self.alu_push(next_pc);
            self.read_next_word()
        } else {
            next_pc
        }
    }

    fn alu_ret(&mut self, test: JumpTest) -> u16 {
        if self.test_jump_condition(test) {
            self.alu_pop()
        } else {
            self.pc.wrapping_add(1)
        }
    }

    /// Adds the immediate next byte value to the current address and jumps
    /// to it.
    fn alu_jr(&mut self) -> u16 {
        ((self.pc as i32) + (self.read_next_byte() as i32)) as u16
    }

    /// Executes JR if a flag condition is met.
    fn alu_jr_if(&mut self, condition: FlagCondition) -> u16 {
        if match condition {
            FlagCondition::C => self.reg.f.c,
            FlagCondition::Z => self.reg.f.z,
            FlagCondition::NC => !self.reg.f.c,
            FlagCondition::NZ => !self.reg.f.z,
        } {
            self.alu_jr()
        } else {
            self.pc.wrapping_add(2)
        }
    }

    // 16-bit Load/Store/Move

    /// Pushes a `value` to the top of the stack.
    fn alu_push(&mut self, value: u16) -> u16 {
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, (value >> 8) as u8);

        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, (value & 0xFF) as u8);

        self.pc.wrapping_add(1)
    }

    /// Pops the last value from the stack.
    fn alu_pop(&mut self) -> u16 {
        let lsb = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        let msb = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        (msb << 8) | lsb
    }

    /// Loads a value into a register or address.
    fn alu_ld(&mut self, load_type: LoadType) -> u16 {
        use LoadByteSource as LBS;
        use LoadByteTarget as LBT;

        match load_type {
            LoadType::Byte(target, source) => {
                let source_value = match source {
                    LBS::A => self.reg.a,
                    LBS::B => self.reg.a,
                    LBS::C => self.reg.a,
                    LBS::D => self.reg.a,
                    LBS::E => self.reg.a,
                    LBS::H => self.reg.h,
                    LBS::L => self.reg.l,
                    LBS::D8 => self.read_next_byte(),
                    LBS::HLI => self.read_byte_hl(),
                };
                match target {
                    LBT::A => self.reg.a = source_value,
                    LBT::B => self.reg.b = source_value,
                    LBT::C => self.reg.c = source_value,
                    LBT::D => self.reg.d = source_value,
                    LBT::E => self.reg.e = source_value,
                    LBT::H => self.reg.h = source_value,
                    LBT::L => self.reg.l = source_value,
                    LBT::HLI => self.bus.write_byte(self.reg.get_hl(), source_value),
                };
                match source {
                    LBS::D8 => self.pc.wrapping_add(2),
                    _ => self.pc.wrapping_add(2),
                }
            }
            LoadType::Word(target, source) => {
                let source_value = match source {
                    LoadWordSource::D16 => self.read_next_word(),
                    _ => panic!("LoadWordSource not implemented"),
                };
                match target {
                    LoadWordTarget::HL => self.reg.set_hl(source_value),
                    _ => panic!("LoadWordTarget not implemented"),
                };
                match source {
                    LoadWordSource::D16 => self.pc.wrapping_add(3),
                    _ => panic!("LoadWord length not implemented"),
                }
            }
            LoadType::IndirectFromA(target) => {
                match target {
                    LoadIndirectTarget::BC => self.bus.write_byte(self.reg.get_bc(), self.reg.a),
                    LoadIndirectTarget::DE => self.bus.write_byte(self.reg.get_de(), self.reg.a),
                    LoadIndirectTarget::HLP => {
                        let hl = self.reg.get_hl();
                        self.bus.write_byte(hl, self.reg.a);
                        self.reg.set_hl(hl.wrapping_add(1));
                    }
                    LoadIndirectTarget::HLM => {
                        let hl = self.reg.get_hl();
                        self.bus.write_byte(hl, self.reg.a);
                        self.reg.set_hl(hl.wrapping_sub(1));
                    }
                }
                self.pc.wrapping_add(1)
            }
        }
    }

    // 8-bit Arithmetic Logic Unit

    /// Adds `value` to the A register (accumulator).
    fn alu_add(&mut self, value: u8) -> u16 {
        let (new_value, overflow) = self.reg.a.overflowing_add(value);
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = (self.reg.a & 0xF) + (value & 0xF) > 0xF;
        self.reg.f.c = overflow;
        self.reg.a = new_value;
        self.pc.wrapping_add(1)
    }

    /// XORs `value` to the A register (accumulator).
    fn alu_xor(&mut self, value: u8) -> u16 {
        let new_value = self.reg.a ^ value;
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = false;
        self.reg.f.c = false;
        self.reg.a = new_value;
        self.pc.wrapping_add(1)
    }

    /// Decrements 1 from the `value` and returns it. Updates flags Z, N and H.
    fn alu_dec(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_sub(1);
        self.reg.f.z = new_value == 0;
        self.reg.f.n = true;
        self.reg.f.h = value.trailing_zeros() >= 4;
        new_value
    }
}
