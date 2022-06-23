use super::instructions::*;
use super::CPU;

/// Executes a given `instruction`
pub fn execute(cpu: &mut CPU, instruction: Instruction) -> u16 {
    use Instruction::*;
    match instruction {
        ADD(target) => add(cpu, target),
        CALL(test) => cpu.alu_call(test),
        DEC(target) => dec(cpu, target),
        HALT => cpu.pc,
        JP(test) => cpu.alu_jump(test),
        JPHL => cpu.reg.get_hl(),
        JR => cpu.alu_jr(),
        JRIF(condition) => cpu.alu_jr_if(condition),
        LD(load_type) => cpu.alu_ld(load_type),
        NOP => cpu.pc.wrapping_add(1),
        POP(target) => pop(cpu, target),
        PUSH(target) => push(cpu, target),
        RET(test) => cpu.alu_ret(test),
        RLCA => rlca(cpu),
        RLA => rla(cpu),
        RRCA => rrca(cpu),
        RRA => rra(cpu),
        XOR(target) => xor(cpu, target),
        _ => cpu.pc, /* TODO: support more instructions */
    }
}

fn rlca(cpu: &mut CPU) -> u16 {
    cpu.reg.a = cpu.alu_rlc(cpu.reg.a);
    cpu.reg.f.z = false;
    cpu.pc.wrapping_add(1)
}

fn rla(cpu: &mut CPU) -> u16 {
    cpu.reg.a = cpu.alu_rl(cpu.reg.a);
    cpu.reg.f.z = false;
    cpu.pc.wrapping_add(1)
}

fn rrca(cpu: &mut CPU) -> u16 {
    cpu.reg.a = cpu.alu_rrc(cpu.reg.a);
    cpu.reg.f.z = false;
    cpu.pc.wrapping_add(1)
}

fn rra(cpu: &mut CPU) -> u16 {
    cpu.reg.a = cpu.alu_rr(cpu.reg.a);
    cpu.reg.f.z = false;
    cpu.pc.wrapping_add(1)
}

fn add(cpu: &mut CPU, target: ArithmeticTarget) -> u16 {
    use ArithmeticTarget as AT;
    match target {
        AT::A => cpu.alu_add(cpu.reg.a),
        AT::B => cpu.alu_add(cpu.reg.b),
        AT::C => cpu.alu_add(cpu.reg.c),
        AT::D => cpu.alu_add(cpu.reg.d),
        AT::E => cpu.alu_add(cpu.reg.e),
        AT::H => cpu.alu_add(cpu.reg.h),
        AT::L => cpu.alu_add(cpu.reg.l),
        AT::HLI => cpu.alu_add(cpu.read_byte_hl()),
        _ => cpu.pc, /* TODO: support more targets */
    }
}

fn dec(cpu: &mut CPU, target: IncDecTarget) -> u16 {
    use IncDecTarget as IDT;
    match target {
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

fn push(cpu: &mut CPU, target: StackTarget) -> u16 {
    use StackTarget as ST;
    cpu.alu_push(match target {
        ST::AF => cpu.reg.get_af(),
        ST::BC => cpu.reg.get_bc(),
        ST::DE => cpu.reg.get_de(),
        ST::HL => cpu.reg.get_hl(),
    })
}

fn pop(cpu: &mut CPU, target: StackTarget) -> u16 {
    use StackTarget as ST;
    let result = cpu.alu_pop();
    match target {
        ST::AF => cpu.reg.set_af(result),
        ST::BC => cpu.reg.set_bc(result),
        ST::DE => cpu.reg.set_de(result),
        ST::HL => cpu.reg.set_hl(result),
    }
    cpu.pc.wrapping_add(1)
}

fn xor(cpu: &mut CPU, target: ArithmeticTarget) -> u16 {
    use ArithmeticTarget as AT;
    match target {
        AT::A => cpu.alu_xor(cpu.reg.a),
        AT::B => cpu.alu_xor(cpu.reg.b),
        AT::C => cpu.alu_xor(cpu.reg.c),
        AT::D => cpu.alu_xor(cpu.reg.d),
        AT::E => cpu.alu_xor(cpu.reg.e),
        AT::H => cpu.alu_xor(cpu.reg.h),
        AT::L => cpu.alu_xor(cpu.reg.l),
        AT::HLI => cpu.alu_xor(cpu.read_byte_hl()),
        AT::D8 => cpu.alu_xor(cpu.read_next_byte()),
    }
}
