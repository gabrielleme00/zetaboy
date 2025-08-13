use crate::PRINT_SERIAL;

const REG_P1: u16 = 0xFF00;
const REG_SB: u16 = 0xFF01;
const REG_SC: u16 = 0xFF02;
// const REG_DIV: u16 = 0xFF04;
// const REG_TIMA: u16 = 0xFF05;
// const REG_TMA: u16 = 0xFF06;
// const REG_TAC: u16 = 0xFF07;
const REG_IF: u16 = 0xFF0F;
const REG_NR10: u16 = 0xFF10;
const REG_NR11: u16 = 0xFF11;
const REG_NR12: u16 = 0xFF12;
const REG_NR13: u16 = 0xFF13;
const REG_NR14: u16 = 0xFF14;
const REG_NR21: u16 = 0xFF16;
const REG_NR22: u16 = 0xFF17;
const REG_NR23: u16 = 0xFF18;
const REG_NR24: u16 = 0xFF19;
const REG_NR30: u16 = 0xFF1A;
const REG_NR31: u16 = 0xFF1B;
const REG_NR32: u16 = 0xFF1C;
const REG_NR33: u16 = 0xFF1D;
const REG_NR34: u16 = 0xFF1E;
const REG_NR41: u16 = 0xFF20;
const REG_NR42: u16 = 0xFF21;
const REG_NR43: u16 = 0xFF22;
const REG_NR44: u16 = 0xFF23;
const REG_NR50: u16 = 0xFF24;
const REG_NR51: u16 = 0xFF25;
const REG_NR52: u16 = 0xFF26;
const REG_LCDC: u16 = 0xFF40;
const REG_STAT: u16 = 0xFF41;
const REG_SCY: u16 = 0xFF42;
const REG_SCX: u16 = 0xFF43;
const REG_LY: u16 = 0xFF44;
const REG_LYC: u16 = 0xFF45;
const REG_DMA: u16 = 0xFF46;
const REG_BGP: u16 = 0xFF47;
pub const REG_OBP0: u16 = 0xFF48;
pub const REG_OBP1: u16 = 0xFF49;
const REG_WY: u16 = 0xFF4A;
const REG_WX: u16 = 0xFF4B;
const REG_KEY1: u16 = 0xFF4D;
const REG_VBK: u16 = 0xFF4F;
const REG_HDMA1: u16 = 0xFF51;
const REG_HDMA2: u16 = 0xFF52;
const REG_HDMA3: u16 = 0xFF53;
const REG_HDMA4: u16 = 0xFF54;
const REG_HDMA5: u16 = 0xFF55;
const REG_SVBK: u16 = 0xFF70;
const REG_IE: u16 = 0xFFFF;

pub struct IORegisters {
    p1: u8,
    sb: u8,
    sc: u8,
    // div: moved to Timer struct
    // tima: u8,
    // tma: u8,
    // tac: u8,
    int_flag: u8,
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,
    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,
    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,
    nr50: u8,
    nr51: u8,
    nr52: u8,
    pub lcdc: u8,
    pub stat: u8,
    pub scy: u8,
    pub scx: u8,
    pub ly: u8,
    pub lyc: u8,
    dma: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,
    key1: u8,
    vbk: u8,
    hdma1: u8,
    hdma2: u8,
    hdma3: u8,
    hdma4: u8,
    hdma5: u8,
    svbk: u8,
    int_enable: u8,
}

impl IORegisters {
    pub fn new() -> Self {
        Self {
            p1: 0xCF,
            sb: 0x00,
            sc: 0x00,
            // div: 0xABCC,
            // tima: 0x00,
            // tma: 0x00,
            // tac: 0x00,
            int_flag: 0x00,
            nr10: 0x80,
            nr11: 0xBF,
            nr12: 0xF3,
            nr13: 0x00,
            nr14: 0xBF,
            nr21: 0x3F,
            nr22: 0x00,
            nr23: 0x00,
            nr24: 0xBF,
            nr30: 0x7F,
            nr31: 0xFF,
            nr32: 0x9F,
            nr33: 0x00,
            nr34: 0xBF,
            nr41: 0xFF,
            nr42: 0x00,
            nr43: 0x00,
            nr44: 0xBF,
            nr50: 0x77,
            nr51: 0xF3,
            nr52: 0b10001111,
            lcdc: 0x91,
            stat: 0x00,
            scy: 0x00,
            scx: 0x00,
            ly: 0x00,
            lyc: 0x00,
            dma: 0x00,
            bgp: 0xFC,
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0x00,
            wx: 0x00,
            key1: 0x00,
            vbk: 0x00,
            hdma1: 0xFF,
            hdma2: 0xFF,
            hdma3: 0x00,
            hdma4: 0x00,
            hdma5: 0xFF,
            svbk: 0x00,
            int_enable: 0x00,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            REG_P1 => (self.p1 & 0x30) | 0x0F,
            REG_SB => self.sb,
            REG_SC => self.sc,
            // REG_DIV => 0x00,
            // REG_TIMA => self.tima,
            // REG_TMA => self.tma,
            // REG_TAC => self.tac,
            REG_IF => self.int_flag,
            REG_NR10 => self.nr10,
            REG_NR11 => self.nr11,
            REG_NR12 => self.nr12,
            REG_NR13 => self.nr13,
            REG_NR14 => self.nr14,
            REG_NR21 => self.nr21,
            REG_NR22 => self.nr22,
            REG_NR23 => self.nr23,
            REG_NR24 => self.nr24,
            REG_NR30 => self.nr30,
            REG_NR31 => self.nr31,
            REG_NR32 => self.nr32,
            REG_NR33 => self.nr33,
            REG_NR34 => self.nr34,
            REG_NR41 => self.nr41,
            REG_NR42 => self.nr42,
            REG_NR43 => self.nr43,
            REG_NR44 => self.nr44,
            REG_NR50 => self.nr50,
            REG_NR51 => self.nr51,
            REG_NR52 => self.nr52,
            REG_LCDC => self.lcdc,
            REG_STAT => self.stat,
            REG_SCY => self.scy,
            REG_SCX => self.scx,
            REG_LY => self.ly,
            REG_LYC => self.lyc,
            REG_DMA => self.dma,
            REG_BGP => self.bgp,
            REG_OBP0 => self.obp0,
            REG_OBP1 => self.obp1,
            REG_WY => self.wy,
            REG_WX => self.wx,
            REG_KEY1 => self.key1,
            REG_VBK => self.vbk,
            REG_HDMA1 => self.hdma1,
            REG_HDMA2 => self.hdma2,
            REG_HDMA3 => self.hdma3,
            REG_HDMA4 => self.hdma4,
            REG_HDMA5 => self.hdma5,
            REG_SVBK => self.svbk,
            REG_IE => self.int_enable,
            _ => 0xFF, // For unused registers
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            REG_P1 => self.p1 = 0xC0 | (self.p1 & 0x0F) | (value & 0x30),
            REG_SB => self.sb = value,
            REG_SC => {
                self.sc = value;
                // Check if transfer is starting (bit 7 set and bit 0 set for internal clock)
                if value & 0x81 == 0x81 {
                    // Print the character from SB register and mark transfer as complete
                    if PRINT_SERIAL {
                        print!("{}", self.sb as char);
                    }
                    self.sc &= 0x7F; // Clear bit 7 to indicate transfer complete
                }
            }
            // REG_DIV => (),
            // REG_TIMA => self.tima = value,
            // REG_TMA => self.tma = value,
            // REG_TAC => self.tac = value & 0x07,
            REG_IF => self.int_flag = value & 0x1F, // Only lower 5 bits are writable
            REG_NR10 => self.nr10 = value,
            REG_NR11 => self.nr11 = value,
            REG_NR12 => self.nr12 = value,
            REG_NR13 => self.nr13 = value,
            REG_NR14 => self.nr14 = value,
            REG_NR21 => self.nr21 = value,
            REG_NR22 => self.nr22 = value,
            REG_NR23 => self.nr23 = value,
            REG_NR24 => self.nr24 = value,
            REG_NR30 => self.nr30 = value,
            REG_NR31 => self.nr31 = value,
            REG_NR32 => self.nr32 = value,
            REG_NR33 => self.nr33 = value,
            REG_NR34 => self.nr34 = value,
            REG_NR41 => self.nr41 = value,
            REG_NR42 => self.nr42 = value,
            REG_NR43 => self.nr43 = value,
            REG_NR44 => self.nr44 = value,
            REG_NR50 => self.nr50 = value,
            REG_NR51 => self.nr51 = value,
            REG_NR52 => self.nr52 = value & 0x80 | (self.nr52 & 0x7F), // Only bit 7 is writable
            REG_LCDC => self.lcdc = value,
            REG_STAT => self.stat = (self.stat & 0x83) | (value & 0x7C), // Bits 0-2 are read-only
            REG_SCY => self.scy = value,
            REG_SCX => self.scx = value,
            REG_LY => (), // LY is read-only
            REG_LYC => self.lyc = value,
            REG_DMA => self.dma = value,
            REG_BGP => self.bgp = value,
            REG_OBP0 => self.obp0 = value,
            REG_OBP1 => self.obp1 = value,
            REG_WY => self.wy = value,
            REG_WX => self.wx = value,
            REG_KEY1 => self.key1 = value,
            REG_VBK => self.vbk = value,
            REG_HDMA1 => self.hdma1 = value,
            REG_HDMA2 => self.hdma2 = value,
            REG_HDMA3 => self.hdma3 = value,
            REG_HDMA4 => self.hdma4 = value,
            REG_HDMA5 => self.hdma5 = value,
            REG_SVBK => self.svbk = value,
            REG_IE => self.int_enable = value & 0x1F, // Only lower 5 bits are writable
            _ => (),                                  // Ignore writes to unused registers
        };
    }
}
