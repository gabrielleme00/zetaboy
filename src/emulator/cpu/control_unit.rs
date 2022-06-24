use super::instructions::*;
use super::CPU;

use ArithmeticSource8 as AS8;

/// Executes a given `instruction`
pub fn execute(cpu: &mut CPU, instruction: Instruction) -> u16 {
    use Instruction::*;
    match instruction {
        ADD(value) => add(cpu, value),
        CALL(test) => call(cpu, test),
        DEC(value) => dec(cpu, value),
        HALT => cpu.pc,
        JP(test) => jump(cpu, test),
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
        XOR(value) => xor(cpu, value),
        _ => cpu.pc, /* TODO: support more instructions */
    }
}

fn add(cpu: &mut CPU, value: AS8) -> u16 {
    match value {
        AS8::A => cpu.alu_add(cpu.reg.a),
        AS8::B => cpu.alu_add(cpu.reg.b),
        AS8::C => cpu.alu_add(cpu.reg.c),
        AS8::D => cpu.alu_add(cpu.reg.d),
        AS8::E => cpu.alu_add(cpu.reg.e),
        AS8::H => cpu.alu_add(cpu.reg.h),
        AS8::L => cpu.alu_add(cpu.reg.l),
        AS8::HLI => cpu.alu_add(cpu.read_byte_hl()),
        _ => cpu.pc, /* TODO: support more targets */
    }
}

fn call(cpu: &mut CPU, test: JumpTest) -> u16 {
    let next_pc = cpu.pc.wrapping_add(3);
    if cpu.test_jump_condition(test) {
        cpu.alu_push(next_pc);
        cpu.read_next_word()
    } else {
        next_pc
    }
}

fn dec(cpu: &mut CPU, value: IncDecTarget) -> u16 {
    use IncDecTarget as IDT;
    match value {
        IDT::A => cpu.reg.a = cpu.alu_dec(cpu.reg.a),
        IDT::B => cpu.reg.b = cpu.alu_dec(cpu.reg.b),
        IDT::C => cpu.reg.c = cpu.alu_dec(cpu.reg.c),
        IDT::D => cpu.reg.d = cpu.alu_dec(cpu.reg.d),
        IDT::E => cpu.reg.e = cpu.alu_dec(cpu.reg.e),
        IDT::H => cpu.reg.h = cpu.alu_dec(cpu.reg.h),
        IDT::L => cpu.reg.l = cpu.alu_dec(cpu.reg.l),
        IDT::HLI => {
            let addr = cpu.reg.get_hl();
            let new_value = cpu.alu_dec(cpu.bus.read_byte(addr));
            cpu.bus.write_byte(addr, new_value);
        }
        IDT::BC => cpu.reg.set_bc(cpu.reg.get_bc().wrapping_sub(1)),
        IDT::DE => cpu.reg.set_de(cpu.reg.get_de().wrapping_sub(1)),
        IDT::HL => cpu.reg.set_hl(cpu.reg.get_hl().wrapping_sub(1)),
        IDT::SP => cpu.sp = cpu.sp.wrapping_sub(1),
    }
    cpu.pc.wrapping_add(1)
}

/// Jumps to the address given by the next 2 bytes if the condition is met.
fn jump(cpu: &CPU, test: JumpTest) -> u16 {
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
        FlagCondition::C => cpu.reg.f.c,
        FlagCondition::Z => cpu.reg.f.z,
        FlagCondition::NC => !cpu.reg.f.c,
        FlagCondition::NZ => !cpu.reg.f.z,
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
        LoadType::Byte(value, source) => {
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
            match value {
                LBT::A => cpu.reg.a = source_value,
                LBT::B => cpu.reg.b = source_value,
                LBT::C => cpu.reg.c = source_value,
                LBT::D => cpu.reg.d = source_value,
                LBT::E => cpu.reg.e = source_value,
                LBT::H => cpu.reg.h = source_value,
                LBT::L => cpu.reg.l = source_value,
                LBT::HLI => cpu.bus.write_byte(cpu.reg.get_hl(), source_value),
            };
            match source {
                LBS::D8 => cpu.pc.wrapping_add(2),
                _ => cpu.pc.wrapping_add(2),
            }
        }
        LoadType::Word(value, source) => {
            let source_value = match source {
                LoadWordSource::D16 => cpu.read_next_word(),
                _ => panic!("LoadWordSource not implemented"),
            };
            match value {
                LoadWordTarget::HL => cpu.reg.set_hl(source_value),
                _ => panic!("LoadWordTarget not implemented"),
            };
            match source {
                LoadWordSource::D16 => cpu.pc.wrapping_add(3),
                _ => panic!("LoadWord length not implemented"),
            }
        }
        LoadType::IndirectFromA(value) => {
            match value {
                LoadIndirectTarget::BC => cpu.bus.write_byte(cpu.reg.get_bc(), cpu.reg.a),
                LoadIndirectTarget::DE => cpu.bus.write_byte(cpu.reg.get_de(), cpu.reg.a),
                LoadIndirectTarget::HLP => {
                    let hl = cpu.reg.get_hl();
                    cpu.bus.write_byte(hl, cpu.reg.a);
                    cpu.reg.set_hl(hl.wrapping_add(1));
                }
                LoadIndirectTarget::HLM => {
                    let hl = cpu.reg.get_hl();
                    cpu.bus.write_byte(hl, cpu.reg.a);
                    cpu.reg.set_hl(hl.wrapping_sub(1));
                }
            }
            cpu.pc.wrapping_add(1)
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

fn ret(cpu: &mut CPU, test: JumpTest) -> u16 {
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
