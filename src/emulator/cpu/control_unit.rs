use super::instructions::*;
use super::CPU;

use ArithmeticSource8 as AS8;
use ArithmeticSource16 as AS16;

/// Executes a given `instruction`
pub fn execute(cpu: &mut CPU, instruction: Instruction) -> u16 {
    use Instruction::*;
    match instruction {
        ADC(source) => adc(cpu, source),
        ADD(source) => add(cpu, source),
        ADDHL(value) => add_hl(cpu, value),
        CALL(test) => call(cpu, test),
        DEC(value) => dec(cpu, value),
        HALT => cpu.pc,
        INC(value) => inc(cpu, value),
        JP(test) => jp(cpu, test),
        JPHL => cpu.reg.get_hl(),
        JR => cpu.alu_jr(),
        JRIF(condition) => jr_if(cpu, condition),
        LD(load_type) => ld(cpu, load_type),
        NOP => cpu.pc.wrapping_add(1),
        OR(value) => or(cpu, value),
        POP(value) => pop(cpu, value),
        PUSH(value) => push(cpu, value),
        RET(test) => ret(cpu, test),
        RLA => rla(cpu),
        RLCA => rlca(cpu),
        RRA => rra(cpu),
        RRCA => rrca(cpu),
        SBC(source) => sbc(cpu, source),
        SUB(source) => sub(cpu, source),
        XOR(value) => xor(cpu, value),
        CP(value) => cp(cpu, value),
        // _ => cpu.pc, /* TODO: support more instructions */
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
        },
    });
    cpu.pc.wrapping_add(length)
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
        },
    });
    cpu.pc.wrapping_add(length)
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
        },
    });
    cpu.pc.wrapping_add(length)
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
        },
    });
    cpu.pc.wrapping_add(length)
}

fn add_hl(cpu: &mut CPU, value: AS16) -> u16 {
    match value {
        AS16::BC => cpu.alu_add_hl(cpu.reg.get_bc()),
        AS16::DE => cpu.alu_add_hl(cpu.reg.get_de()),
        AS16::HL => cpu.alu_add_hl(cpu.reg.get_hl()),
        AS16::SP => cpu.alu_add_hl(cpu.sp),
    };
    cpu.pc.wrapping_add(1)
}

fn call(cpu: &mut CPU, test: JumpCondition) -> u16 {
    let next_pc = cpu.pc.wrapping_add(3);
    if cpu.test_jump_condition(test) {
        cpu.alu_push(next_pc);
        cpu.read_next_word()
    } else {
        next_pc
    }
}

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
        },
    });
    cpu.pc.wrapping_add(length)
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
            cpu.bus.write_byte(addr, new_value);
        }
        IDS::BC => cpu.reg.set_bc(cpu.reg.get_bc().wrapping_sub(1)),
        IDS::DE => cpu.reg.set_de(cpu.reg.get_de().wrapping_sub(1)),
        IDS::HL => cpu.reg.set_hl(cpu.reg.get_hl().wrapping_sub(1)),
        IDS::SP => cpu.sp = cpu.sp.wrapping_sub(1),
    }
    cpu.pc.wrapping_add(1)
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
            cpu.bus.write_byte(addr, new_value);
        }
        IDS::BC => cpu.reg.set_bc(cpu.reg.get_bc().wrapping_add(1)),
        IDS::DE => cpu.reg.set_de(cpu.reg.get_de().wrapping_add(1)),
        IDS::HL => cpu.reg.set_hl(cpu.reg.get_hl().wrapping_add(1)),
        IDS::SP => cpu.sp = cpu.sp.wrapping_add(1),
    }
    cpu.pc.wrapping_add(1)
}

/// Jumps to the address given by the next 2 bytes if the condition is met.
fn jp(cpu: &CPU, test: JumpCondition) -> u16 {
    if cpu.test_jump_condition(test) {
        // Game Boy is little endian so read pc + 2 as most significant byte
        // and pc + 1 as least significant byte
        let least_significant_byte = cpu.bus.read_byte(cpu.pc + 1) as u16;
        let most_significant_byte = cpu.bus.read_byte(cpu.pc + 2) as u16;
        (most_significant_byte << 8) | least_significant_byte
    } else {
        // Jump instructions are always 3 bytes wide
        cpu.pc.wrapping_add(3)
    }
}

/// Executes JR if a flag condition is met.
fn jr_if(cpu: &mut CPU, condition: FlagCondition) -> u16 {
    if match condition {
        FlagCondition::Carry => cpu.reg.f.c,
        FlagCondition::Zero => cpu.reg.f.z,
        FlagCondition::NotCarry => !cpu.reg.f.c,
        FlagCondition::NotZero => !cpu.reg.f.z,
    } {
        cpu.alu_jr()
    } else {
        cpu.pc.wrapping_add(2)
    }
}

/// Loads a value into a register or address.
fn ld(cpu: &mut CPU, load_type: LoadType) -> u16 {
    use LoadByteSource as LBS;
    use LoadByteTarget as LBT;

    match load_type {
        LoadType::Byte(target, source) => {
            let source_value = match source {
                LBS::A => cpu.reg.a,
                LBS::B => cpu.reg.a,
                LBS::C => cpu.reg.a,
                LBS::D => cpu.reg.a,
                LBS::E => cpu.reg.a,
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
                LBT::HLI => cpu.bus.write_byte(cpu.reg.get_hl(), source_value),
            };
            cpu.pc.wrapping_add(match source {
                LBS::D8 => 2,
                _ => 1,
            })
        }
        LoadType::Word(target, source) => {
            let source_value = match source {
                LoadWordSource::D16 => cpu.read_next_word(),
                LoadWordSource::HL => cpu.reg.get_hl(),
                LoadWordSource::SP => cpu.sp,
            };
            match target {
                LoadWordTarget::HL => cpu.reg.set_hl(source_value),
                LoadWordTarget::BC => cpu.reg.set_bc(source_value),
                LoadWordTarget::DE => cpu.reg.set_de(source_value),
                LoadWordTarget::SP => cpu.sp = source_value,
                LoadWordTarget::A16 => {
                    let addr = cpu.read_next_word();
                    cpu.bus.write_byte(addr, cpu.sp as u8);
                    cpu.bus.write_byte(addr + 1, (cpu.sp >> 8) as u8);
                },
            };
            cpu.pc.wrapping_add(3)
        }
        LoadType::IndirectFromA(target) => {
            let mut length = 1;
            match target {
                LoadIndirect::BC => cpu.bus.write_byte(cpu.reg.get_bc(), cpu.reg.a),
                LoadIndirect::DE => cpu.bus.write_byte(cpu.reg.get_de(), cpu.reg.a),
                LoadIndirect::HLinc => {
                    let hl = cpu.reg.get_hl();
                    cpu.bus.write_byte(hl, cpu.reg.a);
                    cpu.reg.set_hl(hl.wrapping_add(1));
                }
                LoadIndirect::HLdec => {
                    let hl = cpu.reg.get_hl();
                    cpu.bus.write_byte(hl, cpu.reg.a);
                    cpu.reg.set_hl(hl.wrapping_sub(1));
                }
                LoadIndirect::HL => cpu.bus.write_byte(cpu.reg.get_hl(), cpu.reg.a),
                LoadIndirect::A8 => {
                    let addr = 0xFF00 | (cpu.read_next_byte() as u16);
                    cpu.bus.write_byte(addr, cpu.reg.a);
                    length = 2;
                },
                LoadIndirect::A16 => {
                    let addr = cpu.read_next_word();
                    cpu.bus.write_byte(addr, cpu.reg.a);
                    length = 3;
                },
                LoadIndirect::C => {
                    let addr = 0xFF00 | (cpu.reg.c as u16);
                    cpu.bus.write_byte(addr, cpu.reg.a);
                },
            }
            cpu.pc.wrapping_add(length)
        }
        LoadType::AFromIndirect(_) => todo!(),
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
    cpu.pc.wrapping_add(length)
}

fn pop(cpu: &mut CPU, value: StackTarget) -> u16 {
    use StackTarget as ST;
    let result = cpu.alu_pop();
    match value {
        ST::AF => cpu.reg.set_af(result),
        ST::BC => cpu.reg.set_bc(result),
        ST::DE => cpu.reg.set_de(result),
        ST::HL => cpu.reg.set_hl(result),
    }
    cpu.pc.wrapping_add(1)
}

fn push(cpu: &mut CPU, value: StackTarget) -> u16 {
    use StackTarget as ST;
    cpu.alu_push(match value {
        ST::AF => cpu.reg.get_af(),
        ST::BC => cpu.reg.get_bc(),
        ST::DE => cpu.reg.get_de(),
        ST::HL => cpu.reg.get_hl(),
    })
}

fn ret(cpu: &mut CPU, test: JumpCondition) -> u16 {
    if cpu.test_jump_condition(test) {
        cpu.alu_pop()
    } else {
        cpu.pc.wrapping_add(1)
    }
}

fn rla(cpu: &mut CPU) -> u16 {
    cpu.reg.a = cpu.alu_rl(cpu.reg.a);
    cpu.reg.f.z = false;
    cpu.pc.wrapping_add(1)
}

fn rlca(cpu: &mut CPU) -> u16 {
    cpu.reg.a = cpu.alu_rlc(cpu.reg.a);
    cpu.reg.f.z = false;
    cpu.pc.wrapping_add(1)
}

fn rra(cpu: &mut CPU) -> u16 {
    cpu.reg.a = cpu.alu_rr(cpu.reg.a);
    cpu.reg.f.z = false;
    cpu.pc.wrapping_add(1)
}

fn rrca(cpu: &mut CPU) -> u16 {
    cpu.reg.a = cpu.alu_rrc(cpu.reg.a);
    cpu.reg.f.z = false;
    cpu.pc.wrapping_add(1)
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
    cpu.pc.wrapping_add(length)
}
