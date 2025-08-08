use super::*;

pub struct OpcodeInfo {
    pub instruction: Instruction,
    pub mnemonic: &'static str,
    pub size: u8,
    pub cycles: u8,
}

impl std::fmt::Display for OpcodeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Instruction: {:?}, Mnemonic: {}, Size: {}, Cycles: {}",
            self.instruction, self.mnemonic, self.size, self.cycles
        )
    }
}

impl OpcodeInfo {
    pub const fn new(
        instruction: Instruction,
        mnemonic: &'static str,
        size: u8,
        cycles: u8,
    ) -> Self {
        Self {
            instruction,
            mnemonic,
            size,
            cycles,
        }
    }

    pub fn from_byte(byte: u8, prefixed: bool) -> Option<&'static Self> {
        match prefixed {
            true => OPCODE_TABLE_PREFIXED.get(byte as usize).and_then(|entry| entry.as_ref().map(|info| info)),
            false => OPCODE_TABLE.get(byte as usize).and_then(|entry| entry.as_ref().map(|info| info)),
        }
    }
}