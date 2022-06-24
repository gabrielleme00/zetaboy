pub enum Instruction {
    // ADC,
    ADD(ArithmeticSource8),
    ADDHL(ArithmeticSource16),
    // ADDSP(ArithmeticTarget),
    // AND,
    // BIT,
    CALL(JumpTest),
    // CCF,
    CP(CPSource),
    // CPL,
    DEC(IncDecSource),
    // DI,
    HALT,
    INC(IncDecSource),
    JP(JumpTest),
    JPHL,
    JR,
    JRIF(FlagCondition),
    LD(LoadType),
    NOP,
    OR(ArithmeticSource8),
    POP(StackTarget),
    PUSH(StackTarget),
    RET(JumpTest),
    // RETI,
    // RST,
    // RL,
    // RLA,
    // RLC,
    // RR,
    RLCA,
    RLA,
    RRCA,
    RRA,
    // SBC,
    // SCF,
    // SET,
    // SLA,
    // SRA,
    // SRL,
    // STOP,
    // SUB,
    // SWAP,
    XOR(ArithmeticSource8),
}

pub enum FlagCondition {
    NZ,
    NC,
    Z,
    C,
}

pub enum JumpTest {
    Always,
    Zero,
    NotZero,
    Carry,
    NotCarry,
}

pub enum ArithmeticSource8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLI,
    D8,
}

pub enum ArithmeticSource16 {
    BC,
    DE,
    HL,
    SP,
}

pub enum IncDecSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    SP,
    BC,
    DE,
    HL,
    HLI,
}

pub enum LoadType {
    Byte(LoadByteTarget, LoadByteSource),
    Word(LoadWordTarget, LoadWordSource),
    AFromIndirect(LoadIndirect),
    IndirectFromA(LoadIndirect),
    // AFromByteAddress(_),
    // ByteAddressFromA(_),
}

pub enum LoadByteTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLI,
}

pub enum LoadByteSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLI,
    D8,
}

pub enum LoadWordTarget {
    BC,
    DE,
    HL,
    SP,
    D16I,
}

pub enum LoadWordSource {
    HL,
    SP,
    D16,
}

pub enum LoadIndirect {
    BC,
    DE,
    HL,
    HLinc,
    HLdec,
    D8,
    D16,
    C,
}

pub enum StackTarget {
    AF,
    BC,
    DE,
    HL,
}

pub enum CPSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLI,
    D8,
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

        use ArithmeticSource16 as AS16;
        use ArithmeticSource8 as AS8;
        use LoadByteSource as LBS;
        use LoadByteTarget as LBT;
        use LoadIndirect as LI;
        use LoadType as LT;
        use LoadWordSource as LWS;
        use LoadWordTarget as LWT;

        match byte {
            0x00 => Some(NOP),
            0x01 => Some(LD(LT::Word(LWT::BC, LWS::D16))),
            0x02 => Some(LD(LT::IndirectFromA(LI::BC))),
            0x03 => Some(INC(IncDecSource::BC)),
            0x04 => Some(INC(IncDecSource::B)),
            0x05 => Some(DEC(IncDecSource::B)),
            0x06 => Some(LD(LT::Byte(LBT::B, LBS::D8))),
            0x07 => Some(RLCA),

            0x08 => Some(LD(LT::Word(LWT::D16I, LWS::SP))),
            0x09 => Some(ADDHL(AS16::BC)),
            0x0A => Some(LD(LT::AFromIndirect(LI::BC))),
            0x0B => Some(DEC(IncDecSource::BC)),
            0x0C => Some(INC(IncDecSource::C)),
            0x0D => Some(DEC(IncDecSource::C)),
            0x0E => Some(LD(LT::Byte(LBT::C, LBS::D8))),
            0x0F => Some(RRCA),

            0x11 => Some(LD(LT::Word(LWT::DE, LWS::D16))),
            0x12 => Some(LD(LT::IndirectFromA(LI::DE))),
            0x13 => Some(INC(IncDecSource::DE)),
            0x14 => Some(INC(IncDecSource::D)),
            0x15 => Some(DEC(IncDecSource::D)),
            0x16 => Some(LD(LT::Byte(LBT::D, LBS::D8))),
            0x17 => Some(RLA),

            0x18 => Some(JR),
            0x19 => Some(ADDHL(AS16::DE)),
            0x1A => Some(LD(LT::AFromIndirect(LI::DE))),
            0x1B => Some(DEC(IncDecSource::DE)),
            0x1C => Some(INC(IncDecSource::E)),
            0x1D => Some(DEC(IncDecSource::E)),
            0x1E => Some(LD(LT::Byte(LBT::E, LBS::D8))),
            0x1F => Some(RRA),

            0x20 => Some(JRIF(FlagCondition::NZ)),
            0x21 => Some(LD(LT::Word(LWT::HL, LWS::D16))),
            0x22 => Some(LD(LT::IndirectFromA(LI::HLinc))),
            0x23 => Some(INC(IncDecSource::HL)),
            0x24 => Some(INC(IncDecSource::H)),
            0x25 => Some(DEC(IncDecSource::H)),
            0x26 => Some(LD(LT::Byte(LBT::H, LBS::D8))),

            0x28 => Some(JRIF(FlagCondition::Z)),
            0x29 => Some(ADDHL(AS16::HL)),
            0x2A => Some(LD(LT::AFromIndirect(LI::HLinc))),
            0x2B => Some(DEC(IncDecSource::HL)),
            0x2C => Some(INC(IncDecSource::L)),
            0x2D => Some(DEC(IncDecSource::L)),
            0x2E => Some(LD(LT::Byte(LBT::L, LBS::D8))),

            0x30 => Some(JRIF(FlagCondition::NC)),
            0x31 => Some(LD(LT::Word(LWT::SP, LWS::D16))),
            0x32 => Some(LD(LT::IndirectFromA(LI::HLdec))),
            0x33 => Some(INC(IncDecSource::SP)),
            0x34 => Some(INC(IncDecSource::HL)),
            0x35 => Some(DEC(IncDecSource::HLI)),
            0x36 => Some(LD(LT::Byte(LBT::HLI, LBS::D8))),

            0x38 => Some(JRIF(FlagCondition::C)),
            0x39 => Some(ADDHL(AS16::SP)),
            0x3A => Some(LD(LT::AFromIndirect(LI::HLdec))),
            0x3B => Some(DEC(IncDecSource::SP)),
            0x3C => Some(INC(IncDecSource::A)),
            0x3D => Some(DEC(IncDecSource::A)),
            0x3E => Some(LD(LT::Byte(LBT::A, LBS::D8))),

            0x40 => Some(LD(LT::Byte(LBT::B, LBS::B))),
            0x41 => Some(LD(LT::Byte(LBT::B, LBS::C))),
            0x42 => Some(LD(LT::Byte(LBT::B, LBS::D))),
            0x43 => Some(LD(LT::Byte(LBT::B, LBS::E))),
            0x44 => Some(LD(LT::Byte(LBT::B, LBS::H))),
            0x45 => Some(LD(LT::Byte(LBT::B, LBS::L))),
            0x46 => Some(LD(LT::Byte(LBT::B, LBS::HLI))),
            0x47 => Some(LD(LT::Byte(LBT::B, LBS::A))),

            0x48 => Some(LD(LT::Byte(LBT::C, LBS::B))),
            0x49 => Some(LD(LT::Byte(LBT::C, LBS::C))),
            0x4A => Some(LD(LT::Byte(LBT::C, LBS::D))),
            0x4B => Some(LD(LT::Byte(LBT::C, LBS::E))),
            0x4C => Some(LD(LT::Byte(LBT::C, LBS::H))),
            0x4D => Some(LD(LT::Byte(LBT::C, LBS::L))),
            0x4E => Some(LD(LT::Byte(LBT::C, LBS::HLI))),
            0x4F => Some(LD(LT::Byte(LBT::C, LBS::A))),

            0x50 => Some(LD(LT::Byte(LBT::D, LBS::B))),
            0x51 => Some(LD(LT::Byte(LBT::D, LBS::C))),
            0x52 => Some(LD(LT::Byte(LBT::D, LBS::D))),
            0x53 => Some(LD(LT::Byte(LBT::D, LBS::E))),
            0x54 => Some(LD(LT::Byte(LBT::D, LBS::H))),
            0x55 => Some(LD(LT::Byte(LBT::D, LBS::L))),
            0x56 => Some(LD(LT::Byte(LBT::D, LBS::HLI))),
            0x57 => Some(LD(LT::Byte(LBT::D, LBS::A))),

            0x58 => Some(LD(LT::Byte(LBT::E, LBS::B))),
            0x59 => Some(LD(LT::Byte(LBT::E, LBS::C))),
            0x5A => Some(LD(LT::Byte(LBT::E, LBS::D))),
            0x5B => Some(LD(LT::Byte(LBT::E, LBS::E))),
            0x5C => Some(LD(LT::Byte(LBT::E, LBS::H))),
            0x5D => Some(LD(LT::Byte(LBT::E, LBS::L))),
            0x5E => Some(LD(LT::Byte(LBT::E, LBS::HLI))),
            0x5F => Some(LD(LT::Byte(LBT::E, LBS::A))),

            0x60 => Some(LD(LT::Byte(LBT::H, LBS::B))),
            0x61 => Some(LD(LT::Byte(LBT::H, LBS::C))),
            0x62 => Some(LD(LT::Byte(LBT::H, LBS::D))),
            0x63 => Some(LD(LT::Byte(LBT::H, LBS::E))),
            0x64 => Some(LD(LT::Byte(LBT::H, LBS::H))),
            0x65 => Some(LD(LT::Byte(LBT::H, LBS::L))),
            0x66 => Some(LD(LT::Byte(LBT::H, LBS::HLI))),
            0x67 => Some(LD(LT::Byte(LBT::H, LBS::A))),

            0x68 => Some(LD(LT::Byte(LBT::L, LBS::B))),
            0x69 => Some(LD(LT::Byte(LBT::L, LBS::C))),
            0x6A => Some(LD(LT::Byte(LBT::L, LBS::D))),
            0x6B => Some(LD(LT::Byte(LBT::L, LBS::E))),
            0x6C => Some(LD(LT::Byte(LBT::L, LBS::H))),
            0x6D => Some(LD(LT::Byte(LBT::L, LBS::L))),
            0x6E => Some(LD(LT::Byte(LBT::L, LBS::HLI))),
            0x6F => Some(LD(LT::Byte(LBT::L, LBS::A))),

            0x70 => Some(LD(LT::Byte(LBT::HLI, LBS::B))),
            0x71 => Some(LD(LT::Byte(LBT::HLI, LBS::C))),
            0x72 => Some(LD(LT::Byte(LBT::HLI, LBS::D))),
            0x73 => Some(LD(LT::Byte(LBT::HLI, LBS::E))),
            0x74 => Some(LD(LT::Byte(LBT::HLI, LBS::H))),
            0x75 => Some(LD(LT::Byte(LBT::HLI, LBS::L))),
            0x76 => Some(HALT),
            0x77 => Some(LD(LT::IndirectFromA(LI::HL))),

            0x78 => Some(LD(LT::Byte(LBT::A, LBS::B))),
            0x79 => Some(LD(LT::Byte(LBT::A, LBS::C))),
            0x7A => Some(LD(LT::Byte(LBT::A, LBS::D))),
            0x7B => Some(LD(LT::Byte(LBT::A, LBS::E))),
            0x7C => Some(LD(LT::Byte(LBT::A, LBS::H))),
            0x7D => Some(LD(LT::Byte(LBT::A, LBS::L))),
            0x7E => Some(LD(LT::Byte(LBT::A, LBS::HLI))),
            0x7F => Some(LD(LT::Byte(LBT::A, LBS::A))),

            0x80 => Some(ADD(AS8::B)),
            0x81 => Some(ADD(AS8::C)),
            0x82 => Some(ADD(AS8::D)),
            0x83 => Some(ADD(AS8::E)),
            0x84 => Some(ADD(AS8::H)),
            0x85 => Some(ADD(AS8::L)),
            0x86 => Some(ADD(AS8::HLI)),
            0x87 => Some(ADD(AS8::A)),

            0xA8 => Some(XOR(AS8::B)),
            0xA9 => Some(XOR(AS8::C)),
            0xAA => Some(XOR(AS8::D)),
            0xAB => Some(XOR(AS8::E)),
            0xAC => Some(XOR(AS8::H)),
            0xAD => Some(XOR(AS8::L)),
            0xAE => Some(XOR(AS8::HLI)),
            0xAF => Some(XOR(AS8::A)),

            0xB0 => Some(OR(AS8::B)),
            0xB1 => Some(OR(AS8::C)),
            0xB2 => Some(OR(AS8::D)),
            0xB3 => Some(OR(AS8::E)),
            0xB4 => Some(OR(AS8::H)),
            0xB5 => Some(OR(AS8::L)),
            0xB6 => Some(OR(AS8::HLI)),
            0xB7 => Some(OR(AS8::A)),

            0xB8 => Some(CP(CPSource::B)),
            0xB9 => Some(CP(CPSource::C)),
            0xBA => Some(CP(CPSource::D)),
            0xBB => Some(CP(CPSource::E)),
            0xBC => Some(CP(CPSource::H)),
            0xBD => Some(CP(CPSource::L)),
            0xBE => Some(CP(CPSource::HLI)),
            0xBF => Some(CP(CPSource::A)),

            0xC0 => Some(RET(JumpTest::NotZero)),
            0xC1 => Some(POP(StackTarget::BC)),
            0xC2 => Some(JP(JumpTest::NotZero)),
            0xC3 => Some(JP(JumpTest::Always)),
            0xC4 => Some(CALL(JumpTest::NotZero)),
            0xC5 => Some(PUSH(StackTarget::BC)),
            // 0xC6 => Some(ADD(ArithmeticTarget::D8)),
            0xC8 => Some(RET(JumpTest::Zero)),
            0xC9 => Some(RET(JumpTest::Always)),
            0xCA => Some(JP(JumpTest::Zero)),
            0xCC => Some(CALL(JumpTest::Zero)),
            0xCD => Some(CALL(JumpTest::Always)),

            0xD0 => Some(RET(JumpTest::NotCarry)),
            0xD1 => Some(POP(StackTarget::DE)),
            0xD2 => Some(JP(JumpTest::NotCarry)),
            0xD3 => None,
            0xD4 => Some(CALL(JumpTest::NotCarry)),
            0xD5 => Some(PUSH(StackTarget::DE)),

            0xD8 => Some(RET(JumpTest::Carry)),
            0xDA => Some(JP(JumpTest::Carry)),
            0xDB => None,
            0xDC => Some(CALL(JumpTest::Carry)),
            0xDD => None,

            0xE0 => Some(LD(LT::IndirectFromA(LI::D8))),
            0xE1 => Some(POP(StackTarget::HL)),
            0xE2 => Some(LD(LT::IndirectFromA(LI::C))),
            0xE3 => None,
            0xE4 => None,
            0xE5 => Some(PUSH(StackTarget::HL)),

            0xEB => None,
            0xEC => None,
            0xED => None,
            0xEE => Some(XOR(AS8::D8)),
            // 0xE8 => Some(ADDSP(ArithmeticTarget::D8)),
            0xE9 => Some(JPHL),
            0xEA => Some(LD(LT::IndirectFromA(LI::D16))),

            0xF1 => Some(POP(StackTarget::AF)),
            0xF4 => None,
            0xF5 => Some(PUSH(StackTarget::AF)),
            0xF6 => Some(OR(AS8::D8)),

            0xF9 => Some(LD(LT::Word(LWT::SP, LWS::HL))),
            0xFC => None,
            0xFD => None,

            _ => None,
        }
    }

    pub fn from_byte_prefixed(_byte: u8) -> Option<Self> {
        None
    }
}
