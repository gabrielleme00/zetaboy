use std::convert;

// Flag register bytes positions
const ZERO_FLAG_POS: u8 = 7;
const SUB_FLAG_POS: u8 = 6;
const HALF_CARRY_FLAG_POS: u8 = 5;
const CARRY_FLAG_POS: u8 = 4;

pub struct Registers {
    pub a: u8, // Accumulator
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: FlagsRegister,
    pub h: u8,
    pub l: u8,
    pub sp: u16, // Stack Pointer
    pub pc: u16, // Program Counter
}

#[derive(Default)]
pub struct FlagsRegister {
    pub zero: bool,
    pub sub: bool,
    pub half_carry: bool,
    pub carry: bool,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: Default::default(),
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }

    fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | (self.c as u16)
    }

    fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }
}

impl convert::From<FlagsRegister> for u8 {
    fn from(flag: FlagsRegister) -> Self {
        (if flag.zero { 1 } else { 0 }) << ZERO_FLAG_POS
            | (if flag.sub { 1 } else { 0 }) << SUB_FLAG_POS
            | (if flag.half_carry { 1 } else { 0 }) << HALF_CARRY_FLAG_POS
            | (if flag.carry { 1 } else { 0 }) << CARRY_FLAG_POS
    }
}

impl std::convert::From<u8> for FlagsRegister {
    fn from(byte: u8) -> Self {
        let zero = ((byte >> ZERO_FLAG_POS) & 0b1) != 0;
        let sub = ((byte >> SUB_FLAG_POS) & 0b1) != 0;
        let half_carry = ((byte >> HALF_CARRY_FLAG_POS) & 0b1) != 0;
        let carry = ((byte >> CARRY_FLAG_POS) & 0b1) != 0;

        FlagsRegister {
            zero,
            sub,
            half_carry,
            carry
        }
    }
}
