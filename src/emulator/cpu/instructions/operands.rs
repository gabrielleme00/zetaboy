#[derive(Debug, Clone, Copy)]
pub enum FlagCondition {
    Zero,
    NotZero,
    Carry,
    NotCarry,
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum ArithmeticSource16 {
    BC,
    DE,
    HL,
    SP,
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum LoadType {
    Byte(LoadByteTarget, LoadByteSource),
    Word(LoadWordTarget, LoadWordSource),
    AFromIndirect(LoadIndirect),
    IndirectFromA(LoadIndirect),
    // AFromByteAddress(_),
    // ByteAddressFromA(_),
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum LoadWordTarget {
    BC,
    DE,
    HL,
    SP,
    A16,
}

#[derive(Debug, Clone, Copy)]
pub enum LoadWordSource {
    HL,
    SP,
    D16,
}

#[derive(Debug, Clone, Copy)]
pub enum LoadIndirect {
    BC,
    DE,
    HL,
    HLinc,
    HLdec,
    A8,
    A16,
    C,
}

#[derive(Debug, Clone, Copy)]
pub enum StackOperand {
    AF,
    BC,
    DE,
    HL,
}
