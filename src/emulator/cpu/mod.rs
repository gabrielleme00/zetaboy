mod bus;
mod instructions;
mod registers;

use core::panic;

use bus::*;
use instructions::{ArithmeticTarget as AT, *};
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
            println!("Executing: {}", description);
            self.execute(instruction)
        } else {
            let description = format!("0x{}{:02X}", if prefixed { "cb" } else { "" }, opcode);
            println!("Unknown instruction found for: {}", description);
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
            NOP => self.pc.wrapping_add(1),
            HALT => self.pc,
            ADD(target) => match target {
                AT::A => self.add(self.reg.a),
                AT::B => self.add(self.reg.b),
                AT::C => self.add(self.reg.c),
                AT::D => self.add(self.reg.d),
                AT::E => self.add(self.reg.e),
                AT::H => self.add(self.reg.h),
                AT::L => self.add(self.reg.l),
                AT::HLI => self.add(self.read_byte_hl()),
                _ => self.pc, /* TODO: support more targets */
            },
            JP(test) => self.jump(self.test_jump_condition(test)),
            JPHL => self.reg.get_hl(),
            LD(load_type) => match load_type {
                LoadType::Byte(target, source) => {
                    let source_value = match source {
                        LoadByteSource::A => self.reg.a,
                        LoadByteSource::B => self.reg.a,
                        LoadByteSource::C => self.reg.a,
                        LoadByteSource::D => self.reg.a,
                        LoadByteSource::E => self.reg.a,
                        LoadByteSource::H => self.reg.h,
                        LoadByteSource::L => self.reg.l,
                        LoadByteSource::D8 => self.read_next_byte(),
                        LoadByteSource::HLI => self.read_byte_hl(),
                    };
                    match target {
                        LoadByteTarget::A => self.reg.a = source_value,
                        LoadByteTarget::B => self.reg.b = source_value,
                        LoadByteTarget::C => self.reg.c = source_value,
                        LoadByteTarget::D => self.reg.d = source_value,
                        LoadByteTarget::E => self.reg.e = source_value,
                        LoadByteTarget::H => self.reg.h = source_value,
                        LoadByteTarget::L => self.reg.l = source_value,
                        LoadByteTarget::HLI => self.bus.write_byte(self.reg.get_hl(), source_value),
                    };
                    match source {
                        LoadByteSource::D8 => self.pc.wrapping_add(2),
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
                        LoadIndirectTarget::BC => {
                            self.bus.write_byte(self.reg.get_bc(), self.reg.a)
                        }
                        LoadIndirectTarget::DE => {
                            self.bus.write_byte(self.reg.get_de(), self.reg.a)
                        }
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
            },
            PUSH(target) => {
                self.push(match target {
                    StackTarget::AF => self.reg.get_af(),
                    StackTarget::BC => self.reg.get_bc(),
                    StackTarget::DE => self.reg.get_de(),
                    StackTarget::HL => self.reg.get_hl(),
                });
                self.pc.wrapping_add(1)
            }
            POP(target) => {
                let result = self.pop();
                match target {
                    StackTarget::AF => self.reg.set_af(result),
                    StackTarget::BC => self.reg.set_bc(result),
                    StackTarget::DE => self.reg.set_de(result),
                    StackTarget::HL => self.reg.set_hl(result),
                }
                self.pc.wrapping_add(1)
            }
            CALL(test) => {
                let jump_condition = self.test_jump_condition(test);
                self.call(jump_condition)
            }
            RET(test) => {
                let jump_condition = self.test_jump_condition(test);
                self.return_(jump_condition)
            }
            XOR(target) => match target {
                AT::A => self.xor(self.reg.a),
                AT::B => self.xor(self.reg.b),
                AT::C => self.xor(self.reg.c),
                AT::D => self.xor(self.reg.d),
                AT::E => self.xor(self.reg.e),
                AT::H => self.xor(self.reg.h),
                AT::L => self.xor(self.reg.l),
                AT::HLI => self.xor(self.read_byte_hl()),
                AT::D8 => self.xor(self.read_next_byte()),
            },
            _ => self.pc, /* TODO: support more instructions */
        }
    }

    // Branch operations

    fn jump(&self, should_jump: bool) -> u16 {
        if should_jump {
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

    fn call(&mut self, should_jump: bool) -> u16 {
        let next_pc = self.pc.wrapping_add(3);
        if should_jump {
            self.push(next_pc);
            self.read_next_word()
        } else {
            next_pc
        }
    }

    fn return_(&mut self, should_jump: bool) -> u16 {
        if should_jump {
            self.pop()
        } else {
            self.pc.wrapping_add(1)
        }
    }

    // 16-bit Load/Store/Move

    /// Pushes a `value` to the top of the stack.
    fn push(&mut self, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, (value >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, (value & 0xFF) as u8);
    }

    /// Pops the last value from the stack.
    fn pop(&mut self) -> u16 {
        let lsb = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        let msb = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        (msb << 8) | lsb
    }

    // 8-bit Arithmetic Logic Unit

    /// Adds `value` to the A register (accumulator).
    fn add(&mut self, value: u8) -> u16 {
        let (new_value, overflow) = self.reg.a.overflowing_add(value);
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = (self.reg.a & 0xF) + (value & 0xF) > 0xF;
        self.reg.f.c = overflow;
        self.reg.a = new_value;
        self.pc.wrapping_add(1)
    }

    /// XORs `value` to the A register (accumulator).
    fn xor(&mut self, value: u8) -> u16 {
        let new_value = self.reg.a ^ value;
        self.reg.f.z = new_value == 0;
        self.reg.f.n = false;
        self.reg.f.h = false;
        self.reg.f.c = false;
        self.reg.a = new_value;
        self.pc.wrapping_add(1)
    }
}
