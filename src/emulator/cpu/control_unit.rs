use super::instructions::*;
use super::CPU;

use ArithmeticSource16 as AS16;
use ArithmeticSource8 as AS8;

/// Executes a given `instruction`. Returns the next program counter value.
pub fn execute(cpu: &mut CPU, instruction: Instruction) -> u16 {
    use Instruction::*;
    match instruction {
        ADC(source) => adc(cpu, source),
        ADD(source) => add(cpu, source),
        ADDHL(value) => add_hl(cpu, value),
        AND(source) => and(cpu, source),
        CALL(test) => call(cpu, test),
        CP(value) => cp(cpu, value),
        CPL => cpl(cpu),
        DEC(value) => dec(cpu, value),
        DI => set_ime(cpu, false),
        EI => set_ime(cpu, true),
        HALT => cpu.reg.pc,
        INC(value) => inc(cpu, value),
        JP(test) => jp(cpu, test),
        JPHL => cpu.reg.get_hl(),
        JR => cpu.alu_jr(),
        JRIF(condition) => jr_if(cpu, condition),
        LD(load_type) => ld(cpu, load_type),
        NOP => cpu.reg.pc.wrapping_add(1),
        OR(value) => or(cpu, value),
        POP(target) => pop(cpu, target),
        PUSH(value) => push(cpu, value),
        RES(bit, target) => res(cpu, bit, target),
        RET(test) => ret(cpu, test),
        RLA => rla(cpu),
        RLCA => rlca(cpu),
        RRA => rra(cpu),
        RRCA => rrca(cpu),
        RST(value) => rst(cpu, value),
        SBC(source) => sbc(cpu, source),
        SUB(source) => sub(cpu, source),
        SWAP(source) => swap(cpu, source),
        XOR(value) => xor(cpu, value),
        // _ => cpu.reg.pc, /* TODO: support more instructions */
    }
}

fn adc(cpu: &mut CPU, source: AS8) -> u16 {
    let mut length = 1;
    cpu.alu_adc(match source {
        AS8::A => cpu.reg.a,
        AS8::B => cpu.reg.b,
        AS8::C => cpu.reg.c,
        AS8::D => cpu.reg.d,
        AS8::E => cpu.reg.e,
        AS8::H => cpu.reg.h,
        AS8::L => cpu.reg.l,
        AS8::HLI => cpu.read_byte_hl(),
        AS8::D8 => {
            length = 2;
            cpu.read_next_byte()
        }
    });
    cpu.reg.pc.wrapping_add(length)
}

fn add(cpu: &mut CPU, source: AS8) -> u16 {
    let mut length = 1;
    cpu.alu_add(match source {
        AS8::A => cpu.reg.a,
        AS8::B => cpu.reg.b,
        AS8::C => cpu.reg.c,
        AS8::D => cpu.reg.d,
        AS8::E => cpu.reg.e,
        AS8::H => cpu.reg.h,
        AS8::L => cpu.reg.l,
        AS8::HLI => cpu.read_byte_hl(),
        AS8::D8 => {
            length = 2;
            cpu.read_next_byte()
        }
    });
    cpu.reg.pc.wrapping_add(length)
}

fn sbc(cpu: &mut CPU, source: AS8) -> u16 {
    let mut length = 1;
    cpu.alu_sbc(match source {
        AS8::A => cpu.reg.a,
        AS8::B => cpu.reg.b,
        AS8::C => cpu.reg.c,
        AS8::D => cpu.reg.d,
        AS8::E => cpu.reg.e,
        AS8::H => cpu.reg.h,
        AS8::L => cpu.reg.l,
        AS8::HLI => cpu.read_byte_hl(),
        AS8::D8 => {
            length = 2;
            cpu.read_next_byte()
        }
    });
    cpu.reg.pc.wrapping_add(length)
}

fn sub(cpu: &mut CPU, source: AS8) -> u16 {
    let mut length = 1;
    cpu.alu_sub(match source {
        AS8::A => cpu.reg.a,
        AS8::B => cpu.reg.b,
        AS8::C => cpu.reg.c,
        AS8::D => cpu.reg.d,
        AS8::E => cpu.reg.e,
        AS8::H => cpu.reg.h,
        AS8::L => cpu.reg.l,
        AS8::HLI => cpu.read_byte_hl(),
        AS8::D8 => {
            length = 2;
            cpu.read_next_byte()
        }
    });
    cpu.reg.pc.wrapping_add(length)
}

fn add_hl(cpu: &mut CPU, value: AS16) -> u16 {
    match value {
        AS16::BC => cpu.alu_add_hl(cpu.reg.get_bc()),
        AS16::DE => cpu.alu_add_hl(cpu.reg.get_de()),
        AS16::HL => cpu.alu_add_hl(cpu.reg.get_hl()),
        AS16::SP => cpu.alu_add_hl(cpu.reg.sp),
    };
    cpu.reg.pc.wrapping_add(1)
}

fn call(cpu: &mut CPU, test: JumpCondition) -> u16 {
    let next_pc = cpu.reg.pc.wrapping_add(3);
    if cpu.test_jump_condition(test) {
        cpu.alu_push(next_pc);
        cpu.read_next_word()
    } else {
        next_pc
    }
}

/// Compares register A and the given `value` by calculating: A - `value`.
fn cp(cpu: &mut CPU, source: AS8) -> u16 {
    let mut length = 1;
    cpu.alu_cp(match source {
        AS8::A => cpu.reg.a,
        AS8::B => cpu.reg.b,
        AS8::C => cpu.reg.c,
        AS8::D => cpu.reg.d,
        AS8::E => cpu.reg.e,
        AS8::H => cpu.reg.h,
        AS8::L => cpu.reg.l,
        AS8::HLI => cpu.bus.read_byte(cpu.reg.get_hl()),
        AS8::D8 => {
            length = 2;
            cpu.read_next_byte()
        }
    });
    cpu.reg.pc.wrapping_add(length)
}

/// Take the one's complement (i.e., flip all bits) of the contents of
/// register A and sets the N and H flags.
fn cpl(cpu: &mut CPU) -> u16 {
    cpu.reg.a = !cpu.reg.a;
    cpu.reg.f.n = true;
    cpu.reg.f.h = true;
    cpu.reg.pc.wrapping_add(1)
}

fn dec(cpu: &mut CPU, value: IncDecSource) -> u16 {
    use IncDecSource as IDS;
    match value {
        IDS::A => cpu.reg.a = cpu.alu_dec(cpu.reg.a),
        IDS::B => cpu.reg.b = cpu.alu_dec(cpu.reg.b),
        IDS::C => cpu.reg.c = cpu.alu_dec(cpu.reg.c),
        IDS::D => cpu.reg.d = cpu.alu_dec(cpu.reg.d),
        IDS::E => cpu.reg.e = cpu.alu_dec(cpu.reg.e),
        IDS::H => cpu.reg.h = cpu.alu_dec(cpu.reg.h),
        IDS::L => cpu.reg.l = cpu.alu_dec(cpu.reg.l),
        IDS::HLI => {
            let addr = cpu.reg.get_hl();
            let new_value = cpu.alu_dec(cpu.bus.read_byte(addr));
            cpu.bus.write_byte(addr, new_value).unwrap();
        }
        IDS::BC => cpu.reg.set_bc(cpu.reg.get_bc().wrapping_sub(1)),
        IDS::DE => cpu.reg.set_de(cpu.reg.get_de().wrapping_sub(1)),
        IDS::HL => cpu.reg.set_hl(cpu.reg.get_hl().wrapping_sub(1)),
        IDS::SP => cpu.reg.sp = cpu.reg.sp.wrapping_sub(1),
    }
    cpu.reg.pc.wrapping_add(1)
}

fn inc(cpu: &mut CPU, value: IncDecSource) -> u16 {
    use IncDecSource as IDS;
    match value {
        IDS::A => cpu.reg.a = cpu.alu_inc(cpu.reg.a),
        IDS::B => cpu.reg.b = cpu.alu_inc(cpu.reg.b),
        IDS::C => cpu.reg.c = cpu.alu_inc(cpu.reg.c),
        IDS::D => cpu.reg.d = cpu.alu_inc(cpu.reg.d),
        IDS::E => cpu.reg.e = cpu.alu_inc(cpu.reg.e),
        IDS::H => cpu.reg.h = cpu.alu_inc(cpu.reg.h),
        IDS::L => cpu.reg.l = cpu.alu_inc(cpu.reg.l),
        IDS::HLI => {
            let addr = cpu.reg.get_hl();
            let new_value = cpu.alu_inc(cpu.bus.read_byte(addr));
            cpu.bus.write_byte(addr, new_value).unwrap();
        }
        IDS::BC => cpu.reg.set_bc(cpu.reg.get_bc().wrapping_add(1)),
        IDS::DE => cpu.reg.set_de(cpu.reg.get_de().wrapping_add(1)),
        IDS::HL => cpu.reg.set_hl(cpu.reg.get_hl().wrapping_add(1)),
        IDS::SP => cpu.reg.sp = cpu.reg.sp.wrapping_add(1),
    }
    cpu.reg.pc.wrapping_add(1)
}

/// Jumps to the address given by the next 2 bytes if the condition is met.
fn jp(cpu: &CPU, test: JumpCondition) -> u16 {
    if cpu.test_jump_condition(test) {
        // Game Boy is little endian so read pc + 1 as least significant byte
        // and pc + 2 as most significant byte
        let least_significant_byte = cpu.bus.read_byte(cpu.reg.pc + 1) as u16;
        let most_significant_byte = cpu.bus.read_byte(cpu.reg.pc + 2) as u16;
        (most_significant_byte << 8) | least_significant_byte
    } else {
        // Jump instructions are always 3 bytes wide
        cpu.reg.pc.wrapping_add(3)
    }
}

/// Executes JR if a flag condition is met.
fn jr_if(cpu: &mut CPU, condition: FlagCondition) -> u16 {
    if cpu.test_flag_condition(condition) {
        cpu.alu_jr()
    } else {
        cpu.reg.pc.wrapping_add(2)
    }
}

/// Loads a value into a register or address.
fn ld(cpu: &mut CPU, load_type: LoadType) -> u16 {
    use LoadByteSource as LBS;
    use LoadByteTarget as LBT;
    use LoadWordSource as LWS;
    use LoadWordTarget as LWT;
    use LoadIndirect as LI;
    use LoadType as LT;

    match load_type {
        LT::Byte(target, source) => {
            let source_value = match source {
                LBS::A => cpu.reg.a,
                LBS::B => cpu.reg.b,
                LBS::C => cpu.reg.c,
                LBS::D => cpu.reg.d,
                LBS::E => cpu.reg.e,
                LBS::H => cpu.reg.h,
                LBS::L => cpu.reg.l,
                LBS::D8 => cpu.read_next_byte(),
                LBS::HLI => cpu.read_byte_hl(),
            };
            match target {
                LBT::A => cpu.reg.a = source_value,
                LBT::B => cpu.reg.b = source_value,
                LBT::C => cpu.reg.c = source_value,
                LBT::D => cpu.reg.d = source_value,
                LBT::E => cpu.reg.e = source_value,
                LBT::H => cpu.reg.h = source_value,
                LBT::L => cpu.reg.l = source_value,
                LBT::HLI => {
                    let addr = cpu.reg.get_hl();
                    cpu.bus.write_byte(addr, source_value).unwrap();
                }
            };
            cpu.reg.pc.wrapping_add(match source {
                LBS::D8 => 2,
                _ => 1,
            })
        }
        LT::Word(target, source) => {
            let source_value = match source {
                LWS::D16 => cpu.read_next_word(),
                LWS::HL => cpu.reg.get_hl(),
                LWS::SP => cpu.reg.sp,
            };
            match target {
                LWT::HL => cpu.reg.set_hl(source_value),
                LWT::BC => cpu.reg.set_bc(source_value),
                LWT::DE => cpu.reg.set_de(source_value),
                LWT::SP => cpu.reg.sp = source_value,
                LWT::A16 => {
                    let addr = cpu.read_next_word();
                    cpu.bus.write_word(addr, cpu.reg.sp).unwrap();
                }
            };
            cpu.reg.pc.wrapping_add(match (target, source) {
                (LWT::A16, _) => 3, // A16 target always uses 3 bytes (opcode + 2-byte address)
                (_, LWS::D16) => 3, // D16 source uses 3 bytes (opcode + 2-byte immediate)
                _ => 1,                        // Register to register operations are 1 byte
            })
        }
        LT::IndirectFromA(target) => {
            let mut length = 1;
            match target {
                LI::BC => {
                    cpu.bus.write_byte(cpu.reg.get_bc(), cpu.reg.a).unwrap();
                }
                LI::DE => {
                    cpu.bus.write_byte(cpu.reg.get_de(), cpu.reg.a).unwrap();
                }
                LI::HLinc => {
                    let hl = cpu.reg.get_hl();
                    cpu.bus.write_byte(hl, cpu.reg.a).unwrap();
                    cpu.reg.set_hl(hl.wrapping_add(1));
                }
                LI::HLdec => {
                    let hl = cpu.reg.get_hl();
                    cpu.bus.write_byte(hl, cpu.reg.a).unwrap();
                    cpu.reg.set_hl(hl.wrapping_sub(1));
                }
                LI::HL => {
                    cpu.bus.write_byte(cpu.reg.get_hl(), cpu.reg.a).unwrap();
                }
                LI::A8 => {
                    let offset = cpu.read_next_byte() as u16;
                    let addr = 0xFF00 | offset;
                    cpu.bus.write_byte(addr, cpu.reg.a).unwrap();
                    length = 2;
                }
                LI::A16 => {
                    let addr = cpu.read_next_word();
                    cpu.bus.write_byte(addr, cpu.reg.a).unwrap();
                    length = 3;
                }
                LI::C => {
                    let addr = 0xFF00 | (cpu.reg.c as u16);
                    cpu.bus.write_byte(addr, cpu.reg.a).unwrap();
                }
            }
            cpu.reg.pc.wrapping_add(length)
        }
        LT::AFromIndirect(source) => {
            let mut length = 1;
            match source {
                LI::A8 => {
                    let offset = cpu.read_next_byte() as u16;
                    let addr = 0xFF00 | offset;
                    cpu.reg.a = cpu.bus.read_byte(addr);
                    length = 2;
                }
                LI::A16 => {
                    let addr = cpu.read_next_word();
                    cpu.reg.a = cpu.bus.read_byte(addr);
                    length = 3;
                }
                LI::HLinc => {
                    let hl = cpu.reg.get_hl();
                    cpu.reg.a = cpu.bus.read_byte(hl);
                    cpu.reg.set_hl(hl.wrapping_add(1));
                }
                LI::C => {
                    let addr = 0xFF00 | (cpu.reg.c as u16);
                    cpu.reg.a = cpu.bus.read_byte(addr);
                }
                LI::BC => {
                    let addr = cpu.reg.get_bc();
                    cpu.reg.a = cpu.bus.read_byte(addr);
                }
                LI::DE => {
                    let addr = cpu.reg.get_de();
                    cpu.reg.a = cpu.bus.read_byte(addr);
                }
                LI::HL => {
                    let addr = cpu.reg.get_hl();
                    cpu.reg.a = cpu.bus.read_byte(addr);
                }
                LI::HLdec => {
                    let hl = cpu.reg.get_hl();
                    cpu.reg.a = cpu.bus.read_byte(hl);
                    cpu.reg.set_hl(hl.wrapping_sub(1));
                }
            }
            cpu.reg.pc.wrapping_add(length)
        }
    }
}

fn or(cpu: &mut CPU, value: AS8) -> u16 {
    let mut length = 1;
    match value {
        AS8::A => cpu.alu_or(cpu.reg.a),
        AS8::B => cpu.alu_or(cpu.reg.b),
        AS8::C => cpu.alu_or(cpu.reg.c),
        AS8::D => cpu.alu_or(cpu.reg.d),
        AS8::E => cpu.alu_or(cpu.reg.e),
        AS8::H => cpu.alu_or(cpu.reg.h),
        AS8::L => cpu.alu_or(cpu.reg.l),
        AS8::HLI => cpu.alu_or(cpu.read_byte_hl()),
        AS8::D8 => {
            cpu.alu_or(cpu.read_next_byte());
            length = 2;
        }
    };
    cpu.reg.pc.wrapping_add(length)
}

fn and(cpu: &mut CPU, value: AS8) -> u16 {
    let mut length = 1;
    match value {
        AS8::A => cpu.alu_and(cpu.reg.a),
        AS8::B => cpu.alu_and(cpu.reg.b),
        AS8::C => cpu.alu_and(cpu.reg.c),
        AS8::D => cpu.alu_and(cpu.reg.d),
        AS8::E => cpu.alu_and(cpu.reg.e),
        AS8::H => cpu.alu_and(cpu.reg.h),
        AS8::L => cpu.alu_and(cpu.reg.l),
        AS8::HLI => cpu.alu_and(cpu.read_byte_hl()),
        AS8::D8 => {
            cpu.alu_and(cpu.read_next_byte());
            length = 2;
        }
    };
    cpu.reg.pc.wrapping_add(length)
}

fn pop(cpu: &mut CPU, target: StackOperand) -> u16 {
    use StackOperand as ST;
    let result = cpu.alu_pop();
    match target {
        ST::AF => cpu.reg.set_af(result),
        ST::BC => cpu.reg.set_bc(result),
        ST::DE => cpu.reg.set_de(result),
        ST::HL => cpu.reg.set_hl(result),
    }
    cpu.reg.pc.wrapping_add(1)
}

fn push(cpu: &mut CPU, source: StackOperand) -> u16 {
    use StackOperand as ST;
    cpu.alu_push(match source {
        ST::AF => cpu.reg.get_af(),
        ST::BC => cpu.reg.get_bc(),
        ST::DE => cpu.reg.get_de(),
        ST::HL => cpu.reg.get_hl(),
    });
    cpu.reg.pc.wrapping_add(1)
}

fn ret(cpu: &mut CPU, test: JumpCondition) -> u16 {
    if cpu.test_jump_condition(test) {
        cpu.alu_pop()
    } else {
        cpu.reg.pc.wrapping_add(1)
    }
}

fn rla(cpu: &mut CPU) -> u16 {
    cpu.reg.a = cpu.alu_rl(cpu.reg.a);
    cpu.reg.f.z = false;
    cpu.reg.pc.wrapping_add(1)
}

fn rlca(cpu: &mut CPU) -> u16 {
    cpu.reg.a = cpu.alu_rlc(cpu.reg.a);
    cpu.reg.f.z = false;
    cpu.reg.pc.wrapping_add(1)
}

fn rra(cpu: &mut CPU) -> u16 {
    cpu.reg.a = cpu.alu_rr(cpu.reg.a);
    cpu.reg.f.z = false;
    cpu.reg.pc.wrapping_add(1)
}

fn rrca(cpu: &mut CPU) -> u16 {
    cpu.reg.a = cpu.alu_rrc(cpu.reg.a);
    cpu.reg.f.z = false;
    cpu.reg.pc.wrapping_add(1)
}

fn set_ime(cpu: &mut CPU, value: bool) -> u16 {
    cpu.ime = value;
    cpu.reg.pc.wrapping_add(1)
}

fn xor(cpu: &mut CPU, value: AS8) -> u16 {
    let mut length = 1;
    match value {
        AS8::A => cpu.alu_xor(cpu.reg.a),
        AS8::B => cpu.alu_xor(cpu.reg.b),
        AS8::C => cpu.alu_xor(cpu.reg.c),
        AS8::D => cpu.alu_xor(cpu.reg.d),
        AS8::E => cpu.alu_xor(cpu.reg.e),
        AS8::H => cpu.alu_xor(cpu.reg.h),
        AS8::L => cpu.alu_xor(cpu.reg.l),
        AS8::HLI => cpu.alu_xor(cpu.read_byte_hl()),
        AS8::D8 => {
            cpu.alu_xor(cpu.read_next_byte());
            length = 2;
        }
    };
    cpu.reg.pc.wrapping_add(length)
}

fn swap(cpu: &mut CPU, value: AS8) -> u16 {
    let mut length = 1;
    match value {
        AS8::A => cpu.reg.a = cpu.alu_swap(cpu.reg.a),
        AS8::B => cpu.reg.b = cpu.alu_swap(cpu.reg.b),
        AS8::C => cpu.reg.c = cpu.alu_swap(cpu.reg.c),
        AS8::D => cpu.reg.d = cpu.alu_swap(cpu.reg.d),
        AS8::E => cpu.reg.e = cpu.alu_swap(cpu.reg.e),
        AS8::H => cpu.reg.h = cpu.alu_swap(cpu.reg.h),
        AS8::L => cpu.reg.l = cpu.alu_swap(cpu.reg.l),
        AS8::HLI => {
            let addr = cpu.reg.get_hl();
            let value = cpu.bus.read_byte(addr);
            let new_value = cpu.alu_swap(value);
            cpu.bus.write_byte(addr, new_value).unwrap();
        }
        AS8::D8 => {
            cpu.alu_swap(cpu.read_next_byte());
            length = 2;
        }
    };
    cpu.reg.pc.wrapping_add(length)
}

fn rst(cpu: &mut CPU, value: u16) -> u16 {
    cpu.alu_push(cpu.reg.pc.wrapping_add(1));
    value
}

/// Resets a bit in the specified target register or memory location.
fn res(cpu: &mut CPU, bit: u8, target: AS8) -> u16 {
    let inverted_mask = !(1 << bit);
    match target {
        AS8::A => cpu.reg.a &= inverted_mask,
        AS8::B => cpu.reg.b &= inverted_mask,
        AS8::C => cpu.reg.c &= inverted_mask,
        AS8::D => cpu.reg.d &= inverted_mask,
        AS8::E => cpu.reg.e &= inverted_mask,
        AS8::H => cpu.reg.h &= inverted_mask,
        AS8::L => cpu.reg.l &= inverted_mask,
        AS8::HLI => {
            let addr = cpu.reg.get_hl();
            let value = cpu.bus.read_byte(addr);
            cpu.bus.write_byte(addr, value & inverted_mask).unwrap();
        }
        _ => panic!("Unsupported RES target: {:?}", target),
    };
    cpu.reg.pc.wrapping_add(1)
}
