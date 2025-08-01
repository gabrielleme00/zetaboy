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
    pub pc: u16, // Program Counter
    pub sp: u16, // Stack Pointer
}

#[derive(Default, Copy, Clone)]
pub struct FlagsRegister {
    pub z: bool, // Zero
    pub n: bool, // Subtract
    pub h: bool, // Half carry
    pub c: bool, // Carry
}

impl std::fmt::Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A: ${:02X} B: ${:02X} C: ${:02X} D: ${:02X} E: ${:02X} F: [{}] H: ${:02X} L: ${:02X} PC: ${:04X} SP: ${:04X}",
            self.a, self.b, self.c, self.d, self.e, self.f, self.h, self.l, self.pc, self.sp
        )
    }
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

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            f: FlagsRegister::from(0xB0),
            h: 0x01,
            l: 0x4D,
            pc: 0x0100,
            sp: 0xFFFE,
        }
    }

    pub fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | (u8::from(self.f) as u16)
    }

    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | (self.c as u16)
    }

    pub fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | (self.e as u16)
    }

    pub fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | (self.l as u16)
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = FlagsRegister::from((value & 0xFF) as u8);
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xFF) as u8;
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
