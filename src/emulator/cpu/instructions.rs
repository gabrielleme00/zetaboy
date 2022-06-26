mod operands;

pub use operands::*;

pub enum Instruction {
    // Control (misc)
    NOP,
    // STOP,
    HALT,
    // DI
    // EI

    // Control (branch)
    JP(JumpCondition),
    JPHL,
    JR,
    JRIF(FlagCondition),
    CALL(JumpCondition),
    RET(JumpCondition),
    // RETI,
    // RST,

    // ALU (8-bit)
    ADD(ArithmeticSource8),
    ADC(ArithmeticSource8),
    SUB(ArithmeticSource8),
    SBC(ArithmeticSource8),
    // AND,
    XOR(ArithmeticSource8),
    OR(ArithmeticSource8),
    CP(ArithmeticSource8),
    // DAA,
    // SCF,
    // CPL,
    // CCF,

    // ALU (16-bit)
    ADDHL(ArithmeticSource16),
    // ADDSP,

    // LSM (8-bit)

    // LSM (16-bit)
    POP(StackTarget),
    PUSH(StackTarget),

    // RSB (8-bit)
    RLCA,
    RLA,
    RRCA,
    RRA,
    // BIT,
    // RLC,
    // RL,
    // RR,
    // SET,
    // SLA,
    // SRA,
    // SRL,
    // SWAP,

    // TODO: change param types
    INC(IncDecSource),
    DEC(IncDecSource),
    LD(LoadType),
}

impl Instruction {
    pub fn from_byte(byte: u8, prefixed: bool) -> Option<Self> {
        match prefixed {
            false => Instruction::from_byte_not_prefixed(byte),
            true => Instruction::from_byte_prefixed(byte),
        }
    }

    pub fn from_byte_not_prefixed(byte: u8) -> Option<Self> {
        use Instruction::*;
        use Some as S;

        use ArithmeticSource16 as AS16;
        use ArithmeticSource8 as AS8;
        use LoadByteSource as LBS;
        use LoadByteTarget as LBT;
        use LoadIndirect as LI;
        use LoadType as LT;
        use LoadWordSource as LWS;
        use LoadWordTarget as LWT;
        use IncDecSource as IDS;
        use JumpCondition as JC;
        use FlagCondition as FC;
        use StackTarget as ST;

        match byte {
            0x00 => S(NOP),
            0x01 => S(LD(LT::Word(LWT::BC, LWS::D16))),
            0x02 => S(LD(LT::IndirectFromA(LI::BC))),
            0x03 => S(INC(IDS::BC)),
            0x04 => S(INC(IDS::B)),
            0x05 => S(DEC(IDS::B)),
            0x06 => S(LD(LT::Byte(LBT::B, LBS::D8))),
            0x07 => S(RLCA),
            0x08 => S(LD(LT::Word(LWT::A16, LWS::SP))),
            0x09 => S(ADDHL(AS16::BC)),
            0x0A => S(LD(LT::AFromIndirect(LI::BC))),
            0x0B => S(DEC(IDS::BC)),
            0x0C => S(INC(IDS::C)),
            0x0D => S(DEC(IDS::C)),
            0x0E => S(LD(LT::Byte(LBT::C, LBS::D8))),
            0x0F => S(RRCA),

            0x11 => S(LD(LT::Word(LWT::DE, LWS::D16))),
            0x12 => S(LD(LT::IndirectFromA(LI::DE))),
            0x13 => S(INC(IDS::DE)),
            0x14 => S(INC(IDS::D)),
            0x15 => S(DEC(IDS::D)),
            0x16 => S(LD(LT::Byte(LBT::D, LBS::D8))),
            0x17 => S(RLA),
            0x18 => S(JR),
            0x19 => S(ADDHL(AS16::DE)),
            0x1A => S(LD(LT::AFromIndirect(LI::DE))),
            0x1B => S(DEC(IDS::DE)),
            0x1C => S(INC(IDS::E)),
            0x1D => S(DEC(IDS::E)),
            0x1E => S(LD(LT::Byte(LBT::E, LBS::D8))),
            0x1F => S(RRA),

            0x20 => S(JRIF(FC::NotZero)),
            0x21 => S(LD(LT::Word(LWT::HL, LWS::D16))),
            0x22 => S(LD(LT::IndirectFromA(LI::HLinc))),
            0x23 => S(INC(IDS::HL)),
            0x24 => S(INC(IDS::H)),
            0x25 => S(DEC(IDS::H)),
            0x26 => S(LD(LT::Byte(LBT::H, LBS::D8))),
            0x28 => S(JRIF(FC::Zero)),
            0x29 => S(ADDHL(AS16::HL)),
            0x2A => S(LD(LT::AFromIndirect(LI::HLinc))),
            0x2B => S(DEC(IDS::HL)),
            0x2C => S(INC(IDS::L)),
            0x2D => S(DEC(IDS::L)),
            0x2E => S(LD(LT::Byte(LBT::L, LBS::D8))),

            0x30 => S(JRIF(FC::NotCarry)),
            0x31 => S(LD(LT::Word(LWT::SP, LWS::D16))),
            0x32 => S(LD(LT::IndirectFromA(LI::HLdec))),
            0x33 => S(INC(IDS::SP)),
            0x34 => S(INC(IDS::HL)),
            0x35 => S(DEC(IDS::HLI)),
            0x36 => S(LD(LT::Byte(LBT::HLI, LBS::D8))),
            0x38 => S(JRIF(FC::Carry)),
            0x39 => S(ADDHL(AS16::SP)),
            0x3A => S(LD(LT::AFromIndirect(LI::HLdec))),
            0x3B => S(DEC(IDS::SP)),
            0x3C => S(INC(IDS::A)),
            0x3D => S(DEC(IDS::A)),
            0x3E => S(LD(LT::Byte(LBT::A, LBS::D8))),

            0x40 => S(LD(LT::Byte(LBT::B, LBS::B))),
            0x41 => S(LD(LT::Byte(LBT::B, LBS::C))),
            0x42 => S(LD(LT::Byte(LBT::B, LBS::D))),
            0x43 => S(LD(LT::Byte(LBT::B, LBS::E))),
            0x44 => S(LD(LT::Byte(LBT::B, LBS::H))),
            0x45 => S(LD(LT::Byte(LBT::B, LBS::L))),
            0x46 => S(LD(LT::Byte(LBT::B, LBS::HLI))),
            0x47 => S(LD(LT::Byte(LBT::B, LBS::A))),
            0x48 => S(LD(LT::Byte(LBT::C, LBS::B))),
            0x49 => S(LD(LT::Byte(LBT::C, LBS::C))),
            0x4A => S(LD(LT::Byte(LBT::C, LBS::D))),
            0x4B => S(LD(LT::Byte(LBT::C, LBS::E))),
            0x4C => S(LD(LT::Byte(LBT::C, LBS::H))),
            0x4D => S(LD(LT::Byte(LBT::C, LBS::L))),
            0x4E => S(LD(LT::Byte(LBT::C, LBS::HLI))),
            0x4F => S(LD(LT::Byte(LBT::C, LBS::A))),

            0x50 => S(LD(LT::Byte(LBT::D, LBS::B))),
            0x51 => S(LD(LT::Byte(LBT::D, LBS::C))),
            0x52 => S(LD(LT::Byte(LBT::D, LBS::D))),
            0x53 => S(LD(LT::Byte(LBT::D, LBS::E))),
            0x54 => S(LD(LT::Byte(LBT::D, LBS::H))),
            0x55 => S(LD(LT::Byte(LBT::D, LBS::L))),
            0x56 => S(LD(LT::Byte(LBT::D, LBS::HLI))),
            0x57 => S(LD(LT::Byte(LBT::D, LBS::A))),
            0x58 => S(LD(LT::Byte(LBT::E, LBS::B))),
            0x59 => S(LD(LT::Byte(LBT::E, LBS::C))),
            0x5A => S(LD(LT::Byte(LBT::E, LBS::D))),
            0x5B => S(LD(LT::Byte(LBT::E, LBS::E))),
            0x5C => S(LD(LT::Byte(LBT::E, LBS::H))),
            0x5D => S(LD(LT::Byte(LBT::E, LBS::L))),
            0x5E => S(LD(LT::Byte(LBT::E, LBS::HLI))),
            0x5F => S(LD(LT::Byte(LBT::E, LBS::A))),

            0x60 => S(LD(LT::Byte(LBT::H, LBS::B))),
            0x61 => S(LD(LT::Byte(LBT::H, LBS::C))),
            0x62 => S(LD(LT::Byte(LBT::H, LBS::D))),
            0x63 => S(LD(LT::Byte(LBT::H, LBS::E))),
            0x64 => S(LD(LT::Byte(LBT::H, LBS::H))),
            0x65 => S(LD(LT::Byte(LBT::H, LBS::L))),
            0x66 => S(LD(LT::Byte(LBT::H, LBS::HLI))),
            0x67 => S(LD(LT::Byte(LBT::H, LBS::A))),
            0x68 => S(LD(LT::Byte(LBT::L, LBS::B))),
            0x69 => S(LD(LT::Byte(LBT::L, LBS::C))),
            0x6A => S(LD(LT::Byte(LBT::L, LBS::D))),
            0x6B => S(LD(LT::Byte(LBT::L, LBS::E))),
            0x6C => S(LD(LT::Byte(LBT::L, LBS::H))),
            0x6D => S(LD(LT::Byte(LBT::L, LBS::L))),
            0x6E => S(LD(LT::Byte(LBT::L, LBS::HLI))),
            0x6F => S(LD(LT::Byte(LBT::L, LBS::A))),

            0x70 => S(LD(LT::Byte(LBT::HLI, LBS::B))),
            0x71 => S(LD(LT::Byte(LBT::HLI, LBS::C))),
            0x72 => S(LD(LT::Byte(LBT::HLI, LBS::D))),
            0x73 => S(LD(LT::Byte(LBT::HLI, LBS::E))),
            0x74 => S(LD(LT::Byte(LBT::HLI, LBS::H))),
            0x75 => S(LD(LT::Byte(LBT::HLI, LBS::L))),
            0x76 => S(HALT),
            0x77 => S(LD(LT::IndirectFromA(LI::HL))),
            0x78 => S(LD(LT::Byte(LBT::A, LBS::B))),
            0x79 => S(LD(LT::Byte(LBT::A, LBS::C))),
            0x7A => S(LD(LT::Byte(LBT::A, LBS::D))),
            0x7B => S(LD(LT::Byte(LBT::A, LBS::E))),
            0x7C => S(LD(LT::Byte(LBT::A, LBS::H))),
            0x7D => S(LD(LT::Byte(LBT::A, LBS::L))),
            0x7E => S(LD(LT::Byte(LBT::A, LBS::HLI))),
            0x7F => S(LD(LT::Byte(LBT::A, LBS::A))),

            0x80 => S(ADD(AS8::B)),
            0x81 => S(ADD(AS8::C)),
            0x82 => S(ADD(AS8::D)),
            0x83 => S(ADD(AS8::E)),
            0x84 => S(ADD(AS8::H)),
            0x85 => S(ADD(AS8::L)),
            0x86 => S(ADD(AS8::HLI)),
            0x87 => S(ADD(AS8::A)),
            0x88 => S(ADC(AS8::B)),
            0x89 => S(ADC(AS8::C)),
            0x8A => S(ADC(AS8::D)),
            0x8B => S(ADC(AS8::E)),
            0x8C => S(ADC(AS8::H)),
            0x8D => S(ADC(AS8::L)),
            0x8E => S(ADC(AS8::HLI)),
            0x8F => S(ADC(AS8::A)),

            0x90 => S(SUB(AS8::B)),
            0x91 => S(SUB(AS8::C)),
            0x92 => S(SUB(AS8::D)),
            0x93 => S(SUB(AS8::E)),
            0x94 => S(SUB(AS8::H)),
            0x95 => S(SUB(AS8::L)),
            0x96 => S(SUB(AS8::HLI)),
            0x97 => S(SUB(AS8::A)),
            0x98 => S(SBC(AS8::B)),
            0x99 => S(SBC(AS8::C)),
            0x9A => S(SBC(AS8::D)),
            0x9B => S(SBC(AS8::E)),
            0x9C => S(SBC(AS8::H)),
            0x9D => S(SBC(AS8::L)),
            0x9E => S(SBC(AS8::HLI)),
            0x9F => S(SBC(AS8::A)),

            0xA8 => S(XOR(AS8::B)),
            0xA9 => S(XOR(AS8::C)),
            0xAA => S(XOR(AS8::D)),
            0xAB => S(XOR(AS8::E)),
            0xAC => S(XOR(AS8::H)),
            0xAD => S(XOR(AS8::L)),
            0xAE => S(XOR(AS8::HLI)),
            0xAF => S(XOR(AS8::A)),

            0xB0 => S(OR(AS8::B)),
            0xB1 => S(OR(AS8::C)),
            0xB2 => S(OR(AS8::D)),
            0xB3 => S(OR(AS8::E)),
            0xB4 => S(OR(AS8::H)),
            0xB5 => S(OR(AS8::L)),
            0xB6 => S(OR(AS8::HLI)),
            0xB7 => S(OR(AS8::A)),
            0xB8 => S(CP(AS8::B)),
            0xB9 => S(CP(AS8::C)),
            0xBA => S(CP(AS8::D)),
            0xBB => S(CP(AS8::E)),
            0xBC => S(CP(AS8::H)),
            0xBD => S(CP(AS8::L)),
            0xBE => S(CP(AS8::HLI)),
            0xBF => S(CP(AS8::A)),

            0xC0 => S(RET(JC::Flag(FC::NotZero))),
            0xC1 => S(POP(ST::BC)),
            0xC2 => S(JP(JC::Flag(FC::NotZero))),
            0xC3 => S(JP(JC::Always)),
            0xC4 => S(CALL(JC::Flag(FC::NotZero))),
            0xC5 => S(PUSH(ST::BC)),
            // 0xC6 => S(ADD(ArithmeticTarget::D8)),
            0xC8 => S(RET(JC::Flag(FC::Zero))),
            0xC9 => S(RET(JC::Always)),
            0xCA => S(JP(JC::Flag(FC::Zero))),
            0xCC => S(CALL(JC::Flag(FC::Zero))),
            0xCD => S(CALL(JC::Always)),
            0xCE => S(ADC(AS8::D8)),

            0xD0 => S(RET(JC::Flag(FC::NotCarry))),
            0xD1 => S(POP(ST::DE)),
            0xD2 => S(JP(JC::Flag(FC::NotCarry))),
            0xD3 => None,
            0xD4 => S(CALL(JC::Flag(FC::NotCarry))),
            0xD5 => S(PUSH(ST::DE)),
            0xD8 => S(RET(JC::Flag(FC::Carry))),
            0xDA => S(JP(JC::Flag(FC::Carry))),
            0xDB => None,
            0xDC => S(CALL(JC::Flag(FC::Carry))),
            0xDD => None,

            0xE0 => S(LD(LT::IndirectFromA(LI::A8))),
            0xE1 => S(POP(ST::HL)),
            0xE2 => S(LD(LT::IndirectFromA(LI::C))),
            0xE3 => None,
            0xE4 => None,
            0xE5 => S(PUSH(ST::HL)),
            0xEB => None,
            0xEC => None,
            0xED => None,
            0xEE => S(XOR(AS8::D8)),
            // 0xE8 => S(ADDSP(ArithmeticTarget::D8)),
            0xE9 => S(JPHL),
            0xEA => S(LD(LT::IndirectFromA(LI::A16))),

            0xF1 => S(POP(ST::AF)),
            0xF4 => None,
            0xF5 => S(PUSH(ST::AF)),
            0xF6 => S(OR(AS8::D8)),
            0xF9 => S(LD(LT::Word(LWT::SP, LWS::HL))),
            0xFC => None,
            0xFD => None,
            0xFE => S(CP(AS8::D8)),

            _ => None,
        }
    }

    pub fn from_byte_prefixed(_byte: u8) -> Option<Self> {
        None
    }
}
