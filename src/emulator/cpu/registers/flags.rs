// Flag register bytes positions
const ZERO_FLAG_POS: u8 = 7;
const SUB_FLAG_POS: u8 = 6;
const HALF_CARRY_FLAG_POS: u8 = 5;
const CARRY_FLAG_POS: u8 = 4;

use std::convert;

#[derive(Default, Copy, Clone)]
pub struct FlagsRegister {
    pub z: bool, // Zero
    pub n: bool, // Subtract
    pub h: bool, // Half carry
    pub c: bool, // Carry
}

impl std::fmt::Display for FlagsRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Z: {} N: {} H: {} C: {}",
            self.z as u8, self.n as u8, self.h as u8, self.c as u8
        )
    }
}

impl convert::From<FlagsRegister> for u8 {
    fn from(flag: FlagsRegister) -> Self {
        (if flag.z { 1 } else { 0 }) << ZERO_FLAG_POS
            | (if flag.n { 1 } else { 0 }) << SUB_FLAG_POS
            | (if flag.h { 1 } else { 0 }) << HALF_CARRY_FLAG_POS
            | (if flag.c { 1 } else { 0 }) << CARRY_FLAG_POS
    }
}

impl std::convert::From<u8> for FlagsRegister {
    fn from(byte: u8) -> Self {
        let zero = ((byte >> ZERO_FLAG_POS) & 0b1) != 0;
        let sub = ((byte >> SUB_FLAG_POS) & 0b1) != 0;
        let half_carry = ((byte >> HALF_CARRY_FLAG_POS) & 0b1) != 0;
        let carry = ((byte >> CARRY_FLAG_POS) & 0b1) != 0;

        FlagsRegister {
            z: zero,
            n: sub,
            h: half_carry,
            c: carry,
        }
    }
}