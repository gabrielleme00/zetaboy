pub enum Instruction {
    // ADC,
    ADD(ArithmeticTarget),
    // ADDHL(ArithmeticTarget),
    // ADDSP(ArithmeticTarget),
    // AND,
    // BIT,
    CALL(JumpTest),
    // CCF,
    // CP,
    // CPL,
    // DEC,
    // DI,
    HALT,
    // INC(ArithmeticTarget),
    JP(JumpTest),
    JPHL,
    LD(LoadType),
    NOP,
    // OR,
    POP(StackTarget),
    PUSH(StackTarget),
    RET(JumpTest),
    // RETI,
    // RST,
    // RL,
    // RLA,
    // RLC,
    // RR,
    // RRA,
    // RRC,
    // RRCA,
    // RRLA,
    // SBC,
    // SCF,
    // SET,
    // SLA,
    // SRA,
    // SRL,
    // STOP,
    // SUB,
    // SWAP,
    XOR(ArithmeticTarget),
}

pub enum JumpTest {
    Always,
    Zero,
    NotZero,
    Carry,
    NotCarry,
}

pub enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    // BC,
    // DE,
    HLI,
    // SP,
    D8,
}

pub enum LoadType {
    Byte(LoadByteTarget, LoadByteSource),
    Word(LoadWordTarget, LoadWordSource),
    // AFromIndirect(_),
    IndirectFromA(LoadIndirectTarget),
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
    D16I
}

pub enum LoadWordSource {
    HL,
    SP,
    D16,
}

pub enum LoadIndirectTarget {
    BC,
    DE,
    HLP,
    HLM,
}

pub enum StackTarget {
    AF,
    BC,
    DE,
    HL,
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
        use LoadType as LT;
        use LoadByteTarget as LBT;
        use LoadByteSource as LBS;
        use LoadWordTarget as LWT;
        use LoadWordSource as LWS;
        use LoadIndirectTarget as LIT;

        match byte {
            0x00 => Some(NOP),
            0x01 => Some(LD(LT::Word(LWT::BC, LWS::D16))),
            0x02 => Some(LD(LT::IndirectFromA(LIT::BC))),
            // 0x03 => Some(INC(ArithmeticTarget::BC)),
            // 0x04 => Some(INC(ArithmeticTarget::B)),
            0x06 => Some(LD(LT::Byte(LBT::B, LBS::D8))),
            0x08 => Some(LD(LT::Word(LWT::D16I, LWS::SP))),
            // 0x09 => Some(ADDHL(ArithmeticTarget::BC)),

            // 0x0C => Some(INC(ArithmeticTarget::C)),
            0x0E => Some(LD(LT::Byte(LBT::C, LBS::D8))),

            0x11 => Some(LD(LT::Word(LWT::DE, LWS::D16))),
            0x12 => Some(LD(LT::IndirectFromA(LIT::DE))),
            // 0x13 => Some(INC(ArithmeticTarget::DE)),
            // 0x14 => Some(INC(ArithmeticTarget::D)),
            0x16 => Some(LD(LT::Byte(LBT::D, LBS::D8))),
            // 0x19 => Some(ADDHL(ArithmeticTarget::DE)),

            // 0x1C => Some(INC(ArithmeticTarget::E)),
            0x1E => Some(LD(LT::Byte(LBT::E, LBS::D8))),

            0x21 => Some(LD(LT::Word(LWT::HL, LWS::D16))),
            0x22 => Some(LD(LT::IndirectFromA(LIT::HLP))),
            // 0x23 => Some(INC(ArithmeticTarget::HL)),
            // 0x24 => Some(INC(ArithmeticTarget::H)),
            0x26 => Some(LD(LT::Byte(LBT::H, LBS::D8))),
            // 0x29 => Some(ADDHL(ArithmeticTarget::HL)),

            // 0x2C => Some(INC(ArithmeticTarget::L)),
            0x2E => Some(LD(LT::Byte(LBT::L, LBS::D8))),

            0x31 => Some(LD(LT::Word(LWT::SP, LWS::D16))),
            0x32 => Some(LD(LT::IndirectFromA(LIT::HLM))),
            // 0x33 => Some(INC(ArithmeticTarget::SP)),
            // 0x34 => Some(INC(ArithmeticTarget::HL)),
            0x36 => Some(LD(LT::Byte(LBT::HLI, LBS::D8))),
            // 0x39 => Some(ADDHL(ArithmeticTarget::SP)),

            // 0x3C => Some(INC(ArithmeticTarget::A)),
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
            0x77 => Some(LD(LT::Byte(LBT::HLI, LBS::A))),

            0x78 => Some(LD(LT::Byte(LBT::A, LBS::B))),
            0x79 => Some(LD(LT::Byte(LBT::A, LBS::C))),
            0x7A => Some(LD(LT::Byte(LBT::A, LBS::D))),
            0x7B => Some(LD(LT::Byte(LBT::A, LBS::E))),
            0x7C => Some(LD(LT::Byte(LBT::A, LBS::H))),
            0x7D => Some(LD(LT::Byte(LBT::A, LBS::L))),
            0x7E => Some(LD(LT::Byte(LBT::A, LBS::HLI))),
            0x7F => Some(LD(LT::Byte(LBT::A, LBS::A))),

            0x80 => Some(ADD(ArithmeticTarget::B)),
            0x81 => Some(ADD(ArithmeticTarget::C)),
            0x82 => Some(ADD(ArithmeticTarget::D)),
            0x83 => Some(ADD(ArithmeticTarget::E)),
            0x84 => Some(ADD(ArithmeticTarget::H)),
            0x85 => Some(ADD(ArithmeticTarget::L)),
            0x86 => Some(ADD(ArithmeticTarget::HLI)),
            0x87 => Some(ADD(ArithmeticTarget::A)),

            0xA8 => Some(XOR(ArithmeticTarget::B)),
            0xA9 => Some(XOR(ArithmeticTarget::C)),
            0xAA => Some(XOR(ArithmeticTarget::D)),
            0xAB => Some(XOR(ArithmeticTarget::E)),
            0xAC => Some(XOR(ArithmeticTarget::H)),
            0xAD => Some(XOR(ArithmeticTarget::L)),
            0xAE => Some(XOR(ArithmeticTarget::HLI)),
            0xAF => Some(XOR(ArithmeticTarget::A)),

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

            0xE1 => Some(POP(StackTarget::HL)),
            0xE3 => None,
            0xE4 => None,
            0xE5 => Some(PUSH(StackTarget::HL)),

            0xEB => None,
            0xEC => None,
            0xED => None,
            0xEE => Some(XOR(ArithmeticTarget::D8)),
            // 0xE8 => Some(ADDSP(ArithmeticTarget::D8)),
            0xE9 => Some(JPHL),

            0xF1 => Some(POP(StackTarget::AF)),
            0xF4 => None,
            0xF5 => Some(PUSH(StackTarget::AF)),

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
