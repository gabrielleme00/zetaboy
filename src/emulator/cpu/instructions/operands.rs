pub enum FlagCondition {
    Zero,
    NotZero,
    Carry,
    NotCarry,
}

pub enum JumpCondition {
    Always,
    Flag(FlagCondition),
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
