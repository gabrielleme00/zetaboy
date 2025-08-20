use crate::{utils::bits::*, PRINT_SERIAL};

mod addresses {
    pub const REG_P1: u16 = 0xFF00;
    pub const REG_SB: u16 = 0xFF01;
    pub const REG_SC: u16 = 0xFF02;
    // pub const REG_DIV: u16 = 0xFF04;
    // pub const REG_TIMA: u16 = 0xFF05;
    // pub const REG_TMA: u16 = 0xFF06;
    // pub const REG_TAC: u16 = 0xFF07;
    pub const REG_IF: u16 = 0xFF0F;
    pub const REG_NR10: u16 = 0xFF10;
    pub const REG_NR11: u16 = 0xFF11;
    pub const REG_NR12: u16 = 0xFF12;
    // pub const REG_NR13: u16 = 0xFF13;
    pub const REG_NR14: u16 = 0xFF14;
    pub const REG_NR21: u16 = 0xFF16;
    // pub const REG_NR22: u16 = 0xFF17;
    // pub const REG_NR23: u16 = 0xFF18;
    pub const REG_NR24: u16 = 0xFF19;
    pub const REG_NR30: u16 = 0xFF1A;
    pub const REG_NR31: u16 = 0xFF1B;
    pub const REG_NR32: u16 = 0xFF1C;
    // pub const REG_NR33: u16 = 0xFF1D;
    pub const REG_NR34: u16 = 0xFF1E;
    pub const REG_NR41: u16 = 0xFF20;
    // pub const REG_NR42: u16 = 0xFF21;
    // pub const REG_NR43: u16 = 0xFF22;
    pub const REG_NR44: u16 = 0xFF23;
    pub const REG_NR50: u16 = 0xFF24;
    pub const REG_NR51: u16 = 0xFF25;
    pub const REG_NR52: u16 = 0xFF26;
    pub const REG_LCDC: u16 = 0xFF40;
    pub const REG_STAT: u16 = 0xFF41;
    pub const REG_SCY: u16 = 0xFF42;
    pub const REG_SCX: u16 = 0xFF43;
    pub const REG_LY: u16 = 0xFF44;
    pub const REG_LYC: u16 = 0xFF45;
    // pub const REG_DMA: u16 = 0xFF46;
    pub const REG_BGP: u16 = 0xFF47;
    pub const REG_OBP0: u16 = 0xFF48;
    pub const REG_OBP1: u16 = 0xFF49;
    // pub const REG_WY: u16 = 0xFF4A;
    // pub const REG_WX: u16 = 0xFF4B;
    // pub const REG_KEY1: u16 = 0xFF4D;
    // pub const REG_VBK: u16 = 0xFF4F;
    pub const REG_HDMA1: u16 = 0xFF51;
    pub const REG_HDMA2: u16 = 0xFF52;
    // pub const REG_HDMA3: u16 = 0xFF53;
    // pub const REG_HDMA4: u16 = 0xFF54;
    pub const REG_HDMA5: u16 = 0xFF55;
    // pub const REG_SVBK: u16 = 0xFF70;
    pub const REG_IE: u16 = 0xFFFF;
}

pub use addresses::*;

const REG_MEM_SIZE: u16 = 0xFFFF - 0xFF00 + 1;

pub enum InterruptBit {
    VBlank = BIT_0 as isize,
    LCDStat = BIT_1 as isize,
    Timer = BIT_2 as isize,
    // Serial = BIT_3 as isize,
    // Joypad = BIT_4 as isize,
}

#[derive(Debug)]
pub enum JoypadButton {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

impl JoypadButton {
    pub fn as_bit_index(&self) -> usize {
        match self {
            JoypadButton::Right => 0,
            JoypadButton::Left => 1,
            JoypadButton::Up => 2,
            JoypadButton::Down => 3,
            JoypadButton::A => 4,
            JoypadButton::B => 5,
            JoypadButton::Select => 6,
            JoypadButton::Start => 7,
        }
    }
}

pub struct IORegisters {
    joypad_state: u8,
    mem: [u8; REG_MEM_SIZE as usize],
}

impl IORegisters {
    pub fn new() -> Self {
        use get_local_address as addr;

        let mut mem = [0; REG_MEM_SIZE as usize];

        mem[addr(REG_P1)] = 0xFF;
        mem[addr(REG_NR10)] = 0x80;
        mem[addr(REG_NR11)] = 0xBF;
        mem[addr(REG_NR12)] = 0xF3;
        mem[addr(REG_NR14)] = 0xBF;
        mem[addr(REG_NR21)] = 0x3F;
        mem[addr(REG_NR24)] = 0xBF;
        mem[addr(REG_NR30)] = 0x7F;
        mem[addr(REG_NR31)] = 0xFF;
        mem[addr(REG_NR32)] = 0x9F;
        mem[addr(REG_NR34)] = 0xBF;
        mem[addr(REG_NR41)] = 0xFF;
        mem[addr(REG_NR44)] = 0xBF;
        mem[addr(REG_NR50)] = 0x77;
        mem[addr(REG_NR51)] = 0xF3;
        mem[addr(REG_NR52)] = 0b10001111;
        mem[addr(REG_LCDC)] = 0x91;
        mem[addr(REG_BGP)] = 0xFC;
        mem[addr(REG_OBP0)] = 0xFF;
        mem[addr(REG_OBP1)] = 0xFF;
        mem[addr(REG_HDMA1)] = 0xFF;
        mem[addr(REG_HDMA2)] = 0xFF;
        mem[addr(REG_HDMA5)] = 0xFF;

        Self {
            joypad_state: 0xFF, // All buttons unpressed
            mem,
        }
    }

    pub fn request_interrupt(&mut self, interrupt: InterruptBit) {
        self.write(REG_IF, self.read(REG_IF) | interrupt as u8);
    }

    pub fn set_button_state(&mut self, button: JoypadButton, pressed: bool) {
        let bit_index = button.as_bit_index();

        let current_state = (self.joypad_state >> bit_index) & 1;
        let new_state = if pressed { 0 } else { 1 }; // 0 for pressed, 1 for unpressed

        if current_state == 1 && new_state == 0 {
            // Button transitioned from unpressed to pressed, request interrupt
            self.write(REG_IF, 0x10); // Set Joypad interrupt bit (bit 4)
        }

        if pressed {
            self.joypad_state &= !(1 << bit_index); // Set bit to 0 (pressed)
        } else {
            self.joypad_state |= 1 << bit_index; // Set bit to 1 (unpressed)
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        let local_addr = get_local_address(address);

        match address {
            REG_P1 => {
                let mut result = self.mem[local_addr];
                // If P14 (Direction keys) is selected (0)
                if result & 0x10 == 0 {
                    result = (result & 0xF0) | (self.joypad_state & 0x0F); // Use bits 0-3 for directions
                }
                // If P15 (Action keys) is selected (0)
                if result & 0x20 == 0 {
                    result = (result & 0xF0) | ((self.joypad_state >> 4) & 0x0F); // Use bits 4-7 for actions
                }
                result
            },
            _ => self.mem[local_addr],
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        let local_addr = get_local_address(address);

        match address {
            REG_P1 => self.mem[local_addr] = (value & 0x30) | 0xCF,
            REG_SC => {
                self.mem[local_addr] = value;
                // Check if transfer is starting (bit 7 set and bit 0 set for internal clock)
                if value & 0x81 == 0x81 {
                    // Print the character from SB register and mark transfer as complete
                    if PRINT_SERIAL {
                        print!("{}", self.mem[get_local_address(REG_SB) as usize] as char);
                    }
                    self.mem[local_addr] &= 0x7F; // Clear bit 7 to indicate transfer complete
                }
            }
            REG_IF => self.mem[local_addr] = value & 0x1F, // Only lower 5 bits are writable
            REG_NR52 => self.mem[local_addr] = value & 0x80 | (self.mem[local_addr] & 0x7F), // Only bit 7 is writable
            REG_STAT => self.mem[local_addr] = (self.mem[local_addr] & 0x83) | (value & 0x7C), // Bits 0-2 are read-only
            REG_IE => self.mem[local_addr] = value & 0x1F, // Only lower 5 bits are writable
            _ => self.mem[local_addr] = value,
        };
    }
}

fn get_local_address(address: u16) -> usize {
    (address - 0xFF00) as usize
}
