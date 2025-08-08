use super::*;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    // Control (misc)
    NOP,
    STOP,
    HALT,
    DI,
    EI,

    // Control (branch)
    JP(Option<FlagCondition>),
    JPHL,
    JR,
    JRIF(FlagCondition),
    CALL(Option<FlagCondition>),
    RET(Option<FlagCondition>),
    RETI,
    RST(u16),

    // ALU (8-bit)
    ADD(ArithmeticSource8),
    ADC(ArithmeticSource8),
    SUB(ArithmeticSource8),
    SBC(ArithmeticSource8),
    AND(ArithmeticSource8),
    XOR(ArithmeticSource8),
    OR(ArithmeticSource8),
    CP(ArithmeticSource8),
    DAA,
    // SCF,
    CPL,
    // CCF,

    // ALU (16-bit)
    ADDHL(ArithmeticSource16),
    ADDSP,

    // LSM (8-bit)

    // LSM (16-bit)
    POP(StackOperand),
    PUSH(StackOperand),

    // RSB (8-bit)
    RLCA,
    RLA,
    RRCA,
    RRA,
    BIT(u8, ArithmeticSource8),
    RLC(ArithmeticSource8),
    RRC(ArithmeticSource8),
    RL(ArithmeticSource8),
    RR(ArithmeticSource8),
    // SET,
    SLA(ArithmeticSource8),
    SRA(ArithmeticSource8),
    SRL(ArithmeticSource8),
    SWAP(ArithmeticSource8),
    RES(u8, ArithmeticSource8),

    // TODO: change param types
    INC(IncDecSource),
    DEC(IncDecSource),
    LD(LoadType),
}