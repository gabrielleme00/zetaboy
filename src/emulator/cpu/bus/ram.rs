const WRAM_SIZE: usize = 0x2000;
const WRAM_OFFSET: usize = 0xC000;
const HRAM_SIZE: usize = 0x80;
const HRAM_OFFSET: usize = 0xFF80;

pub struct RAM {
  wram: [u8; WRAM_SIZE],
  hram: [u8; HRAM_SIZE],
}

impl RAM {
  pub fn new() -> Self {
      Self { wram: [0; WRAM_SIZE], hram: [0; HRAM_SIZE] }
  }

  pub fn read_wram(&self, address: usize) -> u8 {
    self.wram[address - WRAM_OFFSET]
  }

  pub fn write_wram(&mut self, address: usize, value: u8) {
    self.wram[address - WRAM_OFFSET] = value;
  }

  pub fn read_hram(&self, address: usize) -> u8 {
    self.hram[address - HRAM_OFFSET]
  }

  pub fn write_hram(&mut self, address: usize, value: u8) {
    self.hram[address - HRAM_OFFSET] = value;
  }
}
