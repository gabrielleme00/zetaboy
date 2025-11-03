use super::instructions::*;
use super::CpuMode;
use super::CPU;

use ArithmeticSource16 as AS16;
use ArithmeticSource8 as AS8;

/// Executes a given `instruction`. Returns the next program counter value.
pub fn execute(cpu: &mut CPU, instruction: Instruction) {
    use Instruction::*;

    match instruction {
        ADC(source) => adc(cpu, source),
        ADD(source) => add(cpu, source),
        ADDHL(value) => add_hl(cpu, value),
        ADDSP => add_sp(cpu),
        AND(source) => and(cpu, source),
        CALL(test) => call(cpu, test),
        CCF => ccf(cpu),
        CP(value) => cp(cpu, value),
        CPL => cpu.alu_cpl(),
        DAA => cpu.alu_daa(),
        DEC(value) => dec(cpu, value),
        DI => set_ime(cpu, false),
        EI => set_ime(cpu, true),
        HALT => halt(cpu),
        INC(value) => inc(cpu, value),
        JP(test) => jp(cpu, test),
        JPHL => cpu.reg.pc = cpu.reg.get_hl(),
        JR => jr(cpu),
        JRIF(condition) => jr_if(cpu, condition),
        LD(load_type) => ld(cpu, load_type),
        NOP => {},
        OR(value) => or(cpu, value),
        POP(target) => pop(cpu, target),
        PUSH(value) => push(cpu, value),
        RET(test) => ret(cpu, test),
        RETI => reti(cpu),
        RLA => rla(cpu),
        RLCA => rlca(cpu),
        RRA => rra(cpu),
        RRCA => rrca(cpu),
        RST(value) => cpu.alu_rst(value),
        SBC(source) => sbc(cpu, source),
        SCF => scf(cpu),
        STOP => stop(cpu),
        SUB(source) => sub(cpu, source),
        XOR(value) => xor(cpu, value),
        // Prefixed
        BIT(bit, target) => set_bit(cpu, bit, target),
        RES(bit, target) => res(cpu, bit, target),
        RLC(target) => rlc(cpu, target),
        RRC(target) => rrc(cpu, target),
        RL(target) => rl(cpu, target),
        RR(target) => rr(cpu, target),
        SET(bit, target) => set(cpu, bit, target),
        SLA(target) => sla(cpu, target),
        SRA(target) => sra(cpu, target),
        SRL(target) => srl(cpu, target),
        SWAP(source) => swap(cpu, source),
        // _ => cpu.reg.pc, /* TODO: support more instructions */
    };
}

/// Stops the CPU and enters a low-power mode.
/// In CGB mode, if KEY1 bit 0 is set, performs a speed switch instead.
fn stop(cpu: &mut CPU) {
    cpu.reg.pc = cpu.reg.pc.wrapping_add(1);
    
    if cpu.bus.is_speed_switch_prepared() {
        cpu.bus.perform_speed_switch();
    } else {
        cpu.mode = CpuMode::Stop;
    }
}

/// Halts the CPU until an interrupt occurs.
fn halt(cpu: &mut CPU) {
    if cpu.ime {
        cpu.mode = CpuMode::Halt;
    } else {
        let int_f = cpu.bus.read_byte(0xFF0F);
        let int_e = cpu.bus.read_byte(0xFFFF);
        if int_f & int_e & 0x1F != 0 {
            cpu.mode = CpuMode::HaltBug;
        } else {
            cpu.mode = CpuMode::HaltDI;
        }
    }
}

/// Calls a subroutine at the specified address if the given condition is met.
fn call(cpu: &mut CPU, test: Option<FlagCondition>) {
    let target_addr = cpu.read_word_at_pc();
    if cpu.test_jump_condition(test) {
        cpu.alu_push(cpu.reg.pc);
        cpu.reg.pc = target_addr;
    }
}

/// Adds the given `value` to register A.
fn adc(cpu: &mut CPU, source: AS8) {
    let value = match source {
        AS8::A => cpu.reg.a,
        AS8::B => cpu.reg.b,
        AS8::C => cpu.reg.c,
        AS8::D => cpu.reg.d,
        AS8::E => cpu.reg.e,
        AS8::H => cpu.reg.h,
        AS8::L => cpu.reg.l,
        AS8::HLI => cpu.read_byte_at_hl(),
        AS8::D8 => cpu.read_byte_at_pc(),
    };
    cpu.alu_adc(value);
}

/// Adds the given `value` to register A.
fn add(cpu: &mut CPU, source: AS8) {
    let value = match source {
        AS8::A => cpu.reg.a,
        AS8::B => cpu.reg.b,
        AS8::C => cpu.reg.c,
        AS8::D => cpu.reg.d,
        AS8::E => cpu.reg.e,
        AS8::H => cpu.reg.h,
        AS8::L => cpu.reg.l,
        AS8::HLI => cpu.read_byte_at_hl(),
        AS8::D8 => cpu.read_byte_at_pc(),
    };
    cpu.alu_add(value);
}

/// Subtracts the given `value` from register A.
fn sbc(cpu: &mut CPU, source: AS8) {
    let value = match source {
        AS8::A => cpu.reg.a,
        AS8::B => cpu.reg.b,
        AS8::C => cpu.reg.c,
        AS8::D => cpu.reg.d,
        AS8::E => cpu.reg.e,
        AS8::H => cpu.reg.h,
        AS8::L => cpu.reg.l,
        AS8::HLI => cpu.read_byte_at_hl(),
        AS8::D8 => cpu.read_byte_at_pc(),
    };
    cpu.alu_sbc(value);
}

/// Set the carry flag CY.
fn scf(cpu: &mut CPU) {
    cpu.reg.f.n = false;
    cpu.reg.f.h = false;
    cpu.reg.f.c = true;
}

// Flip the carry flag CY.
fn ccf(cpu: &mut CPU) {
    cpu.reg.f.n = false;
    cpu.reg.f.h = false;
    cpu.reg.f.c = !cpu.reg.f.c;
}

/// Subtracts the given `source` from register A.
fn sub(cpu: &mut CPU, source: AS8) {
    let value = match source {
        AS8::A => cpu.reg.a,
        AS8::B => cpu.reg.b,
        AS8::C => cpu.reg.c,
        AS8::D => cpu.reg.d,
        AS8::E => cpu.reg.e,
        AS8::H => cpu.reg.h,
        AS8::L => cpu.reg.l,
        AS8::HLI => cpu.read_byte_at_hl(),
        AS8::D8 => cpu.read_byte_at_pc(),
    };
    cpu.alu_sub(value);
}

/// Adds the given `source` to the HL register pair.
fn add_hl(cpu: &mut CPU, source: AS16) {
    match source {
        AS16::BC => cpu.alu_add_hl(cpu.reg.get_bc()),
        AS16::DE => cpu.alu_add_hl(cpu.reg.get_de()),
        AS16::HL => cpu.alu_add_hl(cpu.reg.get_hl()),
        AS16::SP => cpu.alu_add_hl(cpu.reg.sp),
    };
}

fn add_sp(cpu: &mut CPU) {
    let value = cpu.read_byte_at_pc() as i8 as i16;
    cpu.alu_add_sp(value);
}

/// Compares register A and the given `value` by calculating: A - `value`.
fn cp(cpu: &mut CPU, source: AS8) {
    let value = match source {
        AS8::A => cpu.reg.a,
        AS8::B => cpu.reg.b,
        AS8::C => cpu.reg.c,
        AS8::D => cpu.reg.d,
        AS8::E => cpu.reg.e,
        AS8::H => cpu.reg.h,
        AS8::L => cpu.reg.l,
        AS8::HLI => cpu.read_byte_at_hl(),
        AS8::D8 => cpu.read_byte_at_pc(),
    };
    cpu.alu_cp(value);
}

fn dec(cpu: &mut CPU, value: IncDecSource) {
    use IncDecSource as IDS;
    match value {
        // 8-bit
        IDS::A => cpu.reg.a = cpu.alu_dec(cpu.reg.a),
        IDS::B => cpu.reg.b = cpu.alu_dec(cpu.reg.b),
        IDS::C => cpu.reg.c = cpu.alu_dec(cpu.reg.c),
        IDS::D => cpu.reg.d = cpu.alu_dec(cpu.reg.d),
        IDS::E => cpu.reg.e = cpu.alu_dec(cpu.reg.e),
        IDS::H => cpu.reg.h = cpu.alu_dec(cpu.reg.h),
        IDS::L => cpu.reg.l = cpu.alu_dec(cpu.reg.l),
        IDS::HLI => {
            let hl = cpu.reg.get_hl();
            let old_value = cpu.read_byte(hl);
            let new_value = cpu.alu_dec(old_value);
            cpu.write_byte(hl, new_value);
        }
        // 16-bit (flags are not set)
        IDS::BC => {
            let result = cpu.alu_dec_16(cpu.reg.get_bc());
            cpu.reg.set_bc(result);
        }
        IDS::DE => {
            let result = cpu.alu_dec_16(cpu.reg.get_de());
            cpu.reg.set_de(result);
        }
        IDS::HL => {
            let result = cpu.alu_dec_16(cpu.reg.get_hl());
            cpu.reg.set_hl(result);
        }
        IDS::SP => cpu.reg.sp = cpu.alu_dec_16(cpu.reg.sp),
    };
}

fn inc(cpu: &mut CPU, value: IncDecSource) {
    use IncDecSource as IDS;
    match value {
        // 8-bit
        IDS::A => cpu.reg.a = cpu.alu_inc(cpu.reg.a),
        IDS::B => cpu.reg.b = cpu.alu_inc(cpu.reg.b),
        IDS::C => cpu.reg.c = cpu.alu_inc(cpu.reg.c),
        IDS::D => cpu.reg.d = cpu.alu_inc(cpu.reg.d),
        IDS::E => cpu.reg.e = cpu.alu_inc(cpu.reg.e),
        IDS::H => cpu.reg.h = cpu.alu_inc(cpu.reg.h),
        IDS::L => cpu.reg.l = cpu.alu_inc(cpu.reg.l),
        IDS::HLI => {
            let hl = cpu.reg.get_hl();
            let old_value = cpu.read_byte(hl);
            let new_value = cpu.alu_inc(old_value);
            cpu.write_byte(hl, new_value);
        }
        // 16-bit (flags are not set)
        IDS::BC => {
            let result = cpu.alu_inc_16(cpu.reg.get_bc());
            cpu.reg.set_bc(result);
        }
        IDS::DE => {
            let result = cpu.alu_inc_16(cpu.reg.get_de());
            cpu.reg.set_de(result);
        }
        IDS::HL => {
            let result = cpu.alu_inc_16(cpu.reg.get_hl());
            cpu.reg.set_hl(result);
        }
        IDS::SP => cpu.reg.sp = cpu.alu_inc_16(cpu.reg.sp),
    };
}

/// Jumps to the address given by the next 2 bytes if the condition is met.
fn jp(cpu: &mut CPU, test: Option<FlagCondition>) {
    let target_addr = cpu.read_word_at_pc();
    if cpu.test_jump_condition(test) {
        cpu.alu_jp(target_addr);
    };
}

/// Jumps to a relative address given by the next 2 bytes.
fn jr(cpu: &mut CPU) {
    let offset = cpu.read_byte_at_pc() as i8 as i16;
    cpu.alu_jr(offset);
}

/// Jumps to a relative address given by the next 2 bytes if a flag condition is met.
fn jr_if(cpu: &mut CPU, condition: FlagCondition) {
    let offset = cpu.read_byte_at_pc() as i8 as i16;
    if cpu.test_flag_condition(condition) {
        cpu.alu_jr(offset);
    };
}

/// Loads a value into a register or address.
fn ld(cpu: &mut CPU, load_type: LoadType) {
    use LoadByteSource as LBS;
    use LoadByteTarget as LBT;
    use LoadIndirect as LI;
    use LoadType as LT;
    use LoadWordSource as LWS;
    use LoadWordTarget as LWT;

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
                LBS::D8 => cpu.read_byte_at_pc(),
                LBS::HLI => cpu.read_byte_at_hl(),
            };
            match target {
                LBT::A => cpu.reg.a = source_value,
                LBT::B => cpu.reg.b = source_value,
                LBT::C => cpu.reg.c = source_value,
                LBT::D => cpu.reg.d = source_value,
                LBT::E => cpu.reg.e = source_value,
                LBT::H => cpu.reg.h = source_value,
                LBT::L => cpu.reg.l = source_value,
                LBT::HLI => cpu.write_byte(cpu.reg.get_hl(), source_value),
            };
        }
        LT::Word(target, source) => {
            let source_value = match source {
                LWS::D16 => cpu.read_word_at_pc(),
                LWS::HL => cpu.reg.get_hl(),
                LWS::SP => match target {
                    LWT::HL => {
                        // 0xF8 - LD HL, SP+s8
                        let r8 = cpu.read_byte_at_pc() as i8 as i16;
                        cpu.reg.f.z = false;
                        cpu.reg.f.n = false;
                        cpu.reg.f.h = ((cpu.reg.sp & 0xF) + (r8 as u16 & 0xF)) > 0xF;
                        cpu.reg.f.c = ((cpu.reg.sp & 0xFF) + (r8 as u16 & 0xFF)) > 0xFF;
                        cpu.reg.sp.wrapping_add(r8 as u16)
                    }
                    _ => cpu.reg.sp,
                },
            };
            match target {
                LWT::HL => cpu.reg.set_hl(source_value),
                LWT::BC => cpu.reg.set_bc(source_value),
                LWT::DE => cpu.reg.set_de(source_value),
                LWT::SP => cpu.reg.sp = source_value,
                LWT::A16 => {
                    let addr = cpu.read_word_at_pc();
                    cpu.write_word(addr, cpu.reg.sp);
                }
            };
        }
        LT::IndirectFromA(target) => {
            match target {
                LI::BC => cpu.write_byte(cpu.reg.get_bc(), cpu.reg.a),
                LI::DE => cpu.write_byte(cpu.reg.get_de(), cpu.reg.a),
                LI::HL => cpu.write_byte(cpu.reg.get_hl(), cpu.reg.a),
                LI::HLinc => {
                    let hl = cpu.reg.get_hl();
                    cpu.write_byte(hl, cpu.reg.a);
                    cpu.reg.set_hl(hl.wrapping_add(1));
                }
                LI::HLdec => {
                    let hl = cpu.reg.get_hl();
                    cpu.write_byte(hl, cpu.reg.a);
                    cpu.reg.set_hl(hl.wrapping_sub(1));
                }
                LI::A8 => {
                    let a8 = cpu.read_byte_at_pc() as u16;
                    let addr = 0xFF00 | a8;
                    cpu.write_byte(addr, cpu.reg.a);
                }
                LI::A16 => {
                    let addr = cpu.read_word_at_pc();
                    cpu.write_byte(addr, cpu.reg.a);
                }
                LI::C => {
                    let addr = 0xFF00 | (cpu.reg.c as u16);
                    cpu.write_byte(addr, cpu.reg.a);
                }
            };
        }
        LT::AFromIndirect(source) => match source {
            LI::A8 => {
                let a8 = cpu.read_byte_at_pc() as u16;
                let addr = 0xFF00 | a8;
                cpu.reg.a = cpu.read_byte(addr);
            }
            LI::A16 => {
                let addr = cpu.read_word_at_pc();
                cpu.reg.a = cpu.read_byte(addr);
            }
            LI::HLinc => {
                let hl = cpu.reg.get_hl();
                cpu.reg.a = cpu.read_byte(hl);
                cpu.reg.set_hl(hl.wrapping_add(1));
            }
            LI::C => {
                let addr = 0xFF00 | (cpu.reg.c as u16);
                cpu.reg.a = cpu.read_byte(addr);
            }
            LI::BC => {
                let addr = cpu.reg.get_bc();
                cpu.reg.a = cpu.read_byte(addr);
            }
            LI::DE => {
                let addr = cpu.reg.get_de();
                cpu.reg.a = cpu.read_byte(addr);
            }
            LI::HL => {
                let addr = cpu.reg.get_hl();
                cpu.reg.a = cpu.read_byte(addr);
            }
            LI::HLdec => {
                let hl = cpu.reg.get_hl();
                cpu.reg.a = cpu.read_byte(hl);
                cpu.reg.set_hl(hl.wrapping_sub(1));
            }
        },
    };
}

fn or(cpu: &mut CPU, value: AS8) {
    match value {
        AS8::A => cpu.alu_or(cpu.reg.a),
        AS8::B => cpu.alu_or(cpu.reg.b),
        AS8::C => cpu.alu_or(cpu.reg.c),
        AS8::D => cpu.alu_or(cpu.reg.d),
        AS8::E => cpu.alu_or(cpu.reg.e),
        AS8::H => cpu.alu_or(cpu.reg.h),
        AS8::L => cpu.alu_or(cpu.reg.l),
        AS8::HLI => {
            let value = cpu.read_byte_at_hl();
            cpu.alu_or(value);
        }
        AS8::D8 => {
            let value = cpu.read_byte_at_pc();
            cpu.alu_or(value);
        }
    };
}

fn and(cpu: &mut CPU, value: AS8) {
    match value {
        AS8::A => cpu.alu_and(cpu.reg.a),
        AS8::B => cpu.alu_and(cpu.reg.b),
        AS8::C => cpu.alu_and(cpu.reg.c),
        AS8::D => cpu.alu_and(cpu.reg.d),
        AS8::E => cpu.alu_and(cpu.reg.e),
        AS8::H => cpu.alu_and(cpu.reg.h),
        AS8::L => cpu.alu_and(cpu.reg.l),
        AS8::HLI => {
            let value = cpu.read_byte_at_hl();
            cpu.alu_and(value);
        }
        AS8::D8 => {
            let value = cpu.read_byte_at_pc();
            cpu.alu_and(value);
        }
    };
}

fn pop(cpu: &mut CPU, target: StackOperand) {
    use StackOperand as ST;
    let result = cpu.alu_pop();
    match target {
        ST::AF => cpu.reg.set_af(result),
        ST::BC => cpu.reg.set_bc(result),
        ST::DE => cpu.reg.set_de(result),
        ST::HL => cpu.reg.set_hl(result),
    };
}

fn push(cpu: &mut CPU, source: StackOperand) {
    use StackOperand as ST;
    cpu.alu_push(match source {
        ST::AF => cpu.reg.get_af(),
        ST::BC => cpu.reg.get_bc(),
        ST::DE => cpu.reg.get_de(),
        ST::HL => cpu.reg.get_hl(),
    });
}

fn ret(cpu: &mut CPU, test: Option<FlagCondition>) {
    if cpu.test_jump_condition(test) {
        if let Some(_) = test {
            cpu.tick4();
        }
        cpu.reg.pc = cpu.alu_pop();
        cpu.tick4();
    }
}

fn reti(cpu: &mut CPU) {
    // RETI enables interrupts immediately (not delayed like EI)
    cpu.ime = true;
    cpu.reg.pc = cpu.alu_pop();
}

fn rla(cpu: &mut CPU) {
    cpu.reg.a = cpu.alu_rl(cpu.reg.a);
    cpu.reg.f.z = false;
}

fn rlca(cpu: &mut CPU) {
    cpu.reg.a = cpu.alu_rlc(cpu.reg.a);
    cpu.reg.f.z = false;
}

fn rra(cpu: &mut CPU) {
    cpu.reg.a = cpu.alu_rr(cpu.reg.a);
    cpu.reg.f.z = false;
}

fn rrca(cpu: &mut CPU) {
    cpu.reg.a = cpu.alu_rrc(cpu.reg.a);
    cpu.reg.f.z = false;
}

fn set_ime(cpu: &mut CPU, value: bool) {
    if value {
        // EI: Enable interrupts after the next instruction
        cpu.mode = CpuMode::EnableIME;
    } else {
        // DI: Disable interrupts immediately
        cpu.ime = false;
    }
}

fn xor(cpu: &mut CPU, value: AS8) {
    match value {
        AS8::A => cpu.alu_xor(cpu.reg.a),
        AS8::B => cpu.alu_xor(cpu.reg.b),
        AS8::C => cpu.alu_xor(cpu.reg.c),
        AS8::D => cpu.alu_xor(cpu.reg.d),
        AS8::E => cpu.alu_xor(cpu.reg.e),
        AS8::H => cpu.alu_xor(cpu.reg.h),
        AS8::L => cpu.alu_xor(cpu.reg.l),
        AS8::HLI => {
            let value = cpu.read_byte_at_hl();
            cpu.alu_xor(value);
        }
        AS8::D8 => {
            let value = cpu.read_byte_at_pc();
            cpu.alu_xor(value);
        }
    };
}

fn rlc(cpu: &mut CPU, target: AS8) {
    match target {
        AS8::A => cpu.reg.a = cpu.alu_rlc(cpu.reg.a),
        AS8::B => cpu.reg.b = cpu.alu_rlc(cpu.reg.b),
        AS8::C => cpu.reg.c = cpu.alu_rlc(cpu.reg.c),
        AS8::D => cpu.reg.d = cpu.alu_rlc(cpu.reg.d),
        AS8::E => cpu.reg.e = cpu.alu_rlc(cpu.reg.e),
        AS8::H => cpu.reg.h = cpu.alu_rlc(cpu.reg.h),
        AS8::L => cpu.reg.l = cpu.alu_rlc(cpu.reg.l),
        AS8::HLI => {
            let addr = cpu.reg.get_hl();
            let old_value = cpu.read_byte(addr);
            let new_value = cpu.alu_rlc(old_value);
            cpu.write_byte(addr, new_value);
        }
        AS8::D8 => unreachable!("RLC D8 instruction does not exist"),
    };
}

fn rrc(cpu: &mut CPU, target: AS8) {
    match target {
        AS8::A => cpu.reg.a = cpu.alu_rrc(cpu.reg.a),
        AS8::B => cpu.reg.b = cpu.alu_rrc(cpu.reg.b),
        AS8::C => cpu.reg.c = cpu.alu_rrc(cpu.reg.c),
        AS8::D => cpu.reg.d = cpu.alu_rrc(cpu.reg.d),
        AS8::E => cpu.reg.e = cpu.alu_rrc(cpu.reg.e),
        AS8::H => cpu.reg.h = cpu.alu_rrc(cpu.reg.h),
        AS8::L => cpu.reg.l = cpu.alu_rrc(cpu.reg.l),
        AS8::HLI => {
            let addr = cpu.reg.get_hl();
            let old_value = cpu.read_byte(addr);
            let new_value = cpu.alu_rrc(old_value);
            cpu.write_byte(addr, new_value);
        }
        AS8::D8 => unreachable!("RRC D8 instruction does not exist"),
    };
}

fn rl(cpu: &mut CPU, target: AS8) {
    match target {
        AS8::A => cpu.reg.a = cpu.alu_rl(cpu.reg.a),
        AS8::B => cpu.reg.b = cpu.alu_rl(cpu.reg.b),
        AS8::C => cpu.reg.c = cpu.alu_rl(cpu.reg.c),
        AS8::D => cpu.reg.d = cpu.alu_rl(cpu.reg.d),
        AS8::E => cpu.reg.e = cpu.alu_rl(cpu.reg.e),
        AS8::H => cpu.reg.h = cpu.alu_rl(cpu.reg.h),
        AS8::L => cpu.reg.l = cpu.alu_rl(cpu.reg.l),
        AS8::HLI => {
            let addr = cpu.reg.get_hl();
            let old_value = cpu.read_byte(addr);
            let new_value = cpu.alu_rl(old_value);
            cpu.write_byte(addr, new_value);
        }
        AS8::D8 => unreachable!("RL D8 instruction does not exist"),
    };
}

fn rr(cpu: &mut CPU, target: AS8) {
    match target {
        AS8::A => cpu.reg.a = cpu.alu_rr(cpu.reg.a),
        AS8::B => cpu.reg.b = cpu.alu_rr(cpu.reg.b),
        AS8::C => cpu.reg.c = cpu.alu_rr(cpu.reg.c),
        AS8::D => cpu.reg.d = cpu.alu_rr(cpu.reg.d),
        AS8::E => cpu.reg.e = cpu.alu_rr(cpu.reg.e),
        AS8::H => cpu.reg.h = cpu.alu_rr(cpu.reg.h),
        AS8::L => cpu.reg.l = cpu.alu_rr(cpu.reg.l),
        AS8::HLI => {
            let addr = cpu.reg.get_hl();
            let old_value = cpu.read_byte(addr);
            let new_value = cpu.alu_rr(old_value);
            cpu.write_byte(addr, new_value);
        }
        AS8::D8 => unreachable!("RR D8 instruction does not exist"),
    };
}

fn set(cpu: &mut CPU, bit: u8, target: AS8) {
    match target {
        AS8::A => cpu.reg.a = cpu.alu_set(bit, cpu.reg.a),
        AS8::B => cpu.reg.b = cpu.alu_set(bit, cpu.reg.b),
        AS8::C => cpu.reg.c = cpu.alu_set(bit, cpu.reg.c),
        AS8::D => cpu.reg.d = cpu.alu_set(bit, cpu.reg.d),
        AS8::E => cpu.reg.e = cpu.alu_set(bit, cpu.reg.e),
        AS8::H => cpu.reg.h = cpu.alu_set(bit, cpu.reg.h),
        AS8::L => cpu.reg.l = cpu.alu_set(bit, cpu.reg.l),
        AS8::HLI => {
            let addr = cpu.reg.get_hl();
            let old_value = cpu.read_byte(addr);
            let new_value = cpu.alu_set(bit, old_value);
            cpu.write_byte(addr, new_value);
        }
        AS8::D8 => unreachable!("SET D8 instruction does not exist"),
    };
}

fn sla(cpu: &mut CPU, target: AS8) {
    match target {
        AS8::A => cpu.reg.a = cpu.alu_sla(cpu.reg.a),
        AS8::B => cpu.reg.b = cpu.alu_sla(cpu.reg.b),
        AS8::C => cpu.reg.c = cpu.alu_sla(cpu.reg.c),
        AS8::D => cpu.reg.d = cpu.alu_sla(cpu.reg.d),
        AS8::E => cpu.reg.e = cpu.alu_sla(cpu.reg.e),
        AS8::H => cpu.reg.h = cpu.alu_sla(cpu.reg.h),
        AS8::L => cpu.reg.l = cpu.alu_sla(cpu.reg.l),
        AS8::HLI => {
            let addr = cpu.reg.get_hl();
            let old_value = cpu.read_byte(addr);
            let new_value = cpu.alu_sla(old_value);
            cpu.write_byte(addr, new_value)
        }
        AS8::D8 => unreachable!("SLA D8 instruction does not exist"),
    };
}

fn sra(cpu: &mut CPU, target: AS8) {
    match target {
        AS8::A => cpu.reg.a = cpu.alu_sra(cpu.reg.a),
        AS8::B => cpu.reg.b = cpu.alu_sra(cpu.reg.b),
        AS8::C => cpu.reg.c = cpu.alu_sra(cpu.reg.c),
        AS8::D => cpu.reg.d = cpu.alu_sra(cpu.reg.d),
        AS8::E => cpu.reg.e = cpu.alu_sra(cpu.reg.e),
        AS8::H => cpu.reg.h = cpu.alu_sra(cpu.reg.h),
        AS8::L => cpu.reg.l = cpu.alu_sra(cpu.reg.l),
        AS8::HLI => {
            let addr = cpu.reg.get_hl();
            let old_value = cpu.read_byte(addr);
            let new_value = cpu.alu_sra(old_value);
            cpu.write_byte(addr, new_value);
        }
        AS8::D8 => unreachable!("SRA D8 instruction does not exist"),
    };
}

fn srl(cpu: &mut CPU, target: AS8) {
    match target {
        AS8::A => cpu.reg.a = cpu.alu_srl(cpu.reg.a),
        AS8::B => cpu.reg.b = cpu.alu_srl(cpu.reg.b),
        AS8::C => cpu.reg.c = cpu.alu_srl(cpu.reg.c),
        AS8::D => cpu.reg.d = cpu.alu_srl(cpu.reg.d),
        AS8::E => cpu.reg.e = cpu.alu_srl(cpu.reg.e),
        AS8::H => cpu.reg.h = cpu.alu_srl(cpu.reg.h),
        AS8::L => cpu.reg.l = cpu.alu_srl(cpu.reg.l),
        AS8::HLI => {
            let addr = cpu.reg.get_hl();
            let old_value = cpu.read_byte(addr);
            let new_value = cpu.alu_srl(old_value);
            cpu.write_byte(addr, new_value);
        }
        AS8::D8 => unreachable!("SRL D8 instruction does not exist"),
    };
}

fn swap(cpu: &mut CPU, value: AS8) {
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
            let old_value = cpu.read_byte(addr);
            let new_value = cpu.alu_swap(old_value);
            cpu.write_byte(addr, new_value);
        }
        AS8::D8 => unreachable!("SWAP D8 instruction does not exist"),
    };
}

/// Tests a bit in the specified target register or memory location (BIT instruction).
fn set_bit(cpu: &mut CPU, bit: u8, target: AS8) {
    match target {
        AS8::A => cpu.alu_bit(bit, cpu.reg.a),
        AS8::B => cpu.alu_bit(bit, cpu.reg.b),
        AS8::C => cpu.alu_bit(bit, cpu.reg.c),
        AS8::D => cpu.alu_bit(bit, cpu.reg.d),
        AS8::E => cpu.alu_bit(bit, cpu.reg.e),
        AS8::H => cpu.alu_bit(bit, cpu.reg.h),
        AS8::L => cpu.alu_bit(bit, cpu.reg.l),
        AS8::HLI => {
            let addr = cpu.reg.get_hl();
            let value = cpu.read_byte(addr);
            cpu.alu_bit(bit, value);
        }
        AS8::D8 => unreachable!("BIT D8 instruction does not exist"),
    };
}

/// Resets a bit in the specified target register or memory location.
fn res(cpu: &mut CPU, bit: u8, target: AS8) {
    let inverted_mask = !(1 << bit);
    cpu.tick4();
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
            let value = cpu.read_byte(addr);
            cpu.write_byte(addr, value & inverted_mask);
        }
        AS8::D8 => unreachable!("RES D8 instruction does not exist"),
    };
}
