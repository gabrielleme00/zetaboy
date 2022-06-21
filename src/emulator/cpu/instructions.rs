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
    // XOR,
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
    HL,
    // SP,
    D8,
}

pub enum LoadType {
    Byte(LoadByteTarget, LoadByteSource),
    // Word(LoadByteTarget, LoadByteSource),
    // AFromIndirect(_),
    // IndirectFromA(_),
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
        use LoadType as LT;
        use LoadByteTarget as LBT;
        use LoadByteSource as LBS;

        match byte {
            0x00 => Some(Self::NOP),
            // 0x03 => Some(Self::INC(ArithmeticTarget::BC)),
            // 0x04 => Some(Self::INC(ArithmeticTarget::B)),
            0x06 => Some(Self::LD(LT::Byte(LBT::B, LBS::D8))),
            // 0x09 => Some(Self::ADDHL(ArithmeticTarget::BC)),

            // 0x0C => Some(Self::INC(ArithmeticTarget::C)),
            0x0E => Some(Self::LD(LT::Byte(LBT::C, LBS::D8))),

            // 0x13 => Some(Self::INC(ArithmeticTarget::DE)),
            // 0x14 => Some(Self::INC(ArithmeticTarget::D)),
            0x16 => Some(Self::LD(LT::Byte(LBT::D, LBS::D8))),
            // 0x19 => Some(Self::ADDHL(ArithmeticTarget::DE)),

            // 0x1C => Some(Self::INC(ArithmeticTarget::E)),
            0x1E => Some(Self::LD(LT::Byte(LBT::E, LBS::D8))),

            // 0x23 => Some(Self::INC(ArithmeticTarget::HL)),
            // 0x24 => Some(Self::INC(ArithmeticTarget::H)),
            0x26 => Some(Self::LD(LT::Byte(LBT::H, LBS::D8))),
            // 0x29 => Some(Self::ADDHL(ArithmeticTarget::HL)),

            // 0x2C => Some(Self::INC(ArithmeticTarget::L)),
            0x2E => Some(Self::LD(LT::Byte(LBT::L, LBS::D8))),

            // 0x33 => Some(Self::INC(ArithmeticTarget::SP)),
            // 0x34 => Some(Self::INC(ArithmeticTarget::HL)),
            0x36 => Some(Self::LD(LT::Byte(LBT::HLI, LBS::D8))),
            // 0x39 => Some(Self::ADDHL(ArithmeticTarget::SP)),

            // 0x3C => Some(Self::INC(ArithmeticTarget::A)),
            0x3E => Some(Self::LD(LT::Byte(LBT::A, LBS::D8))),

            0x40 => Some(Self::LD(LT::Byte(LBT::B, LBS::B))),
            0x41 => Some(Self::LD(LT::Byte(LBT::B, LBS::C))),
            0x42 => Some(Self::LD(LT::Byte(LBT::B, LBS::D))),
            0x43 => Some(Self::LD(LT::Byte(LBT::B, LBS::E))),
            0x44 => Some(Self::LD(LT::Byte(LBT::B, LBS::H))),
            0x45 => Some(Self::LD(LT::Byte(LBT::B, LBS::L))),
            0x46 => Some(Self::LD(LT::Byte(LBT::B, LBS::HLI))),
            0x47 => Some(Self::LD(LT::Byte(LBT::B, LBS::A))),

            0x48 => Some(Self::LD(LT::Byte(LBT::C, LBS::B))),
            0x49 => Some(Self::LD(LT::Byte(LBT::C, LBS::C))),
            0x4A => Some(Self::LD(LT::Byte(LBT::C, LBS::D))),
            0x4B => Some(Self::LD(LT::Byte(LBT::C, LBS::E))),
            0x4C => Some(Self::LD(LT::Byte(LBT::C, LBS::H))),
            0x4D => Some(Self::LD(LT::Byte(LBT::C, LBS::L))),
            0x4E => Some(Self::LD(LT::Byte(LBT::C, LBS::HLI))),
            0x4F => Some(Self::LD(LT::Byte(LBT::C, LBS::A))),

            0x50 => Some(Self::LD(LT::Byte(LBT::D, LBS::B))),
            0x51 => Some(Self::LD(LT::Byte(LBT::D, LBS::C))),
            0x52 => Some(Self::LD(LT::Byte(LBT::D, LBS::D))),
            0x53 => Some(Self::LD(LT::Byte(LBT::D, LBS::E))),
            0x54 => Some(Self::LD(LT::Byte(LBT::D, LBS::H))),
            0x55 => Some(Self::LD(LT::Byte(LBT::D, LBS::L))),
            0x56 => Some(Self::LD(LT::Byte(LBT::D, LBS::HLI))),
            0x57 => Some(Self::LD(LT::Byte(LBT::D, LBS::A))),

            0x58 => Some(Self::LD(LT::Byte(LBT::E, LBS::B))),
            0x59 => Some(Self::LD(LT::Byte(LBT::E, LBS::C))),
            0x5A => Some(Self::LD(LT::Byte(LBT::E, LBS::D))),
            0x5B => Some(Self::LD(LT::Byte(LBT::E, LBS::E))),
            0x5C => Some(Self::LD(LT::Byte(LBT::E, LBS::H))),
            0x5D => Some(Self::LD(LT::Byte(LBT::E, LBS::L))),
            0x5E => Some(Self::LD(LT::Byte(LBT::E, LBS::HLI))),
            0x5F => Some(Self::LD(LT::Byte(LBT::E, LBS::A))),

            0x60 => Some(Self::LD(LT::Byte(LBT::H, LBS::B))),
            0x61 => Some(Self::LD(LT::Byte(LBT::H, LBS::C))),
            0x62 => Some(Self::LD(LT::Byte(LBT::H, LBS::D))),
            0x63 => Some(Self::LD(LT::Byte(LBT::H, LBS::E))),
            0x64 => Some(Self::LD(LT::Byte(LBT::H, LBS::H))),
            0x65 => Some(Self::LD(LT::Byte(LBT::H, LBS::L))),
            0x66 => Some(Self::LD(LT::Byte(LBT::H, LBS::HLI))),
            0x67 => Some(Self::LD(LT::Byte(LBT::H, LBS::A))),

            0x68 => Some(Self::LD(LT::Byte(LBT::L, LBS::B))),
            0x69 => Some(Self::LD(LT::Byte(LBT::L, LBS::C))),
            0x6A => Some(Self::LD(LT::Byte(LBT::L, LBS::D))),
            0x6B => Some(Self::LD(LT::Byte(LBT::L, LBS::E))),
            0x6C => Some(Self::LD(LT::Byte(LBT::L, LBS::H))),
            0x6D => Some(Self::LD(LT::Byte(LBT::L, LBS::L))),
            0x6E => Some(Self::LD(LT::Byte(LBT::L, LBS::HLI))),
            0x6F => Some(Self::LD(LT::Byte(LBT::L, LBS::A))),

            0x70 => Some(Self::LD(LT::Byte(LBT::HLI, LBS::B))),
            0x71 => Some(Self::LD(LT::Byte(LBT::HLI, LBS::C))),
            0x72 => Some(Self::LD(LT::Byte(LBT::HLI, LBS::D))),
            0x73 => Some(Self::LD(LT::Byte(LBT::HLI, LBS::E))),
            0x74 => Some(Self::LD(LT::Byte(LBT::HLI, LBS::H))),
            0x75 => Some(Self::LD(LT::Byte(LBT::HLI, LBS::L))),
            0x76 => Some(Self::HALT),
            0x77 => Some(Self::LD(LT::Byte(LBT::HLI, LBS::A))),

            0x78 => Some(Self::LD(LT::Byte(LBT::A, LBS::B))),
            0x79 => Some(Self::LD(LT::Byte(LBT::A, LBS::C))),
            0x7A => Some(Self::LD(LT::Byte(LBT::A, LBS::D))),
            0x7B => Some(Self::LD(LT::Byte(LBT::A, LBS::E))),
            0x7C => Some(Self::LD(LT::Byte(LBT::A, LBS::H))),
            0x7D => Some(Self::LD(LT::Byte(LBT::A, LBS::L))),
            0x7E => Some(Self::LD(LT::Byte(LBT::A, LBS::HLI))),
            0x7F => Some(Self::LD(LT::Byte(LBT::A, LBS::A))),

            0x80 => Some(Self::ADD(ArithmeticTarget::B)),
            0x81 => Some(Self::ADD(ArithmeticTarget::C)),
            0x82 => Some(Self::ADD(ArithmeticTarget::D)),
            0x83 => Some(Self::ADD(ArithmeticTarget::E)),
            0x84 => Some(Self::ADD(ArithmeticTarget::H)),
            0x85 => Some(Self::ADD(ArithmeticTarget::L)),
            0x86 => Some(Self::ADD(ArithmeticTarget::HL)),
            0x87 => Some(Self::ADD(ArithmeticTarget::A)),

            0xC0 => Some(Self::RET(JumpTest::NotZero)),
            0xC1 => Some(Self::POP(StackTarget::BC)),
            0xC2 => Some(Self::JP(JumpTest::NotZero)),
            0xC3 => Some(Self::JP(JumpTest::Always)),
            0xC4 => Some(Self::CALL(JumpTest::NotZero)),
            0xC5 => Some(Self::PUSH(StackTarget::BC)),
            // 0xC6 => Some(Self::ADD(ArithmeticTarget::D8)),

            0xC8 => Some(Self::RET(JumpTest::Zero)),
            0xC9 => Some(Self::RET(JumpTest::Always)),
            0xCA => Some(Self::JP(JumpTest::Zero)),
            0xCC => Some(Self::CALL(JumpTest::Zero)),
            0xCD => Some(Self::CALL(JumpTest::Always)),

            0xD0 => Some(Self::RET(JumpTest::NotCarry)),
            0xD1 => Some(Self::POP(StackTarget::DE)),
            0xD2 => Some(Self::JP(JumpTest::NotCarry)),
            0xD3 => None,
            0xD4 => Some(Self::CALL(JumpTest::NotCarry)),
            0xD5 => Some(Self::PUSH(StackTarget::DE)),

            0xD8 => Some(Self::RET(JumpTest::Carry)),
            0xDA => Some(Self::JP(JumpTest::Carry)),
            0xDB => None,
            0xDC => Some(Self::CALL(JumpTest::Carry)),
            0xDD => None,

            0xE1 => Some(Self::POP(StackTarget::HL)),
            0xE3 => None,
            0xE4 => None,
            0xE5 => Some(Self::PUSH(StackTarget::HL)),

            0xEB => None,
            0xEC => None,
            0xED => None,
            // 0xE8 => Some(Self::ADDSP(ArithmeticTarget::D8)),
            0xE9 => Some(Self::JPHL),

            0xF1 => Some(Self::POP(StackTarget::AF)),
            0xF4 => None,
            0xF5 => Some(Self::PUSH(StackTarget::AF)),

            0xFC => None,
            0xFD => None,

            _ => None,
        }
    }

    pub fn from_byte_prefixed(_byte: u8) -> Option<Self> {
        None
    }
}
