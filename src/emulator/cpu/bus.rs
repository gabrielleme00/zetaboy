const MEMORY_SIZE: usize = 0xFFFF;

pub struct MemoryBus {
    memory: [u8; MEMORY_SIZE],
}

impl MemoryBus {
    pub fn new() -> Self {
        Self { memory: [0; MEMORY_SIZE] }
    }

    /// Reads a byte from the `address`.
    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    /// Reads 2 bytes from the `address`.
    pub fn read_word(&self, address: u16) -> u16 {
        let a = (self.read_byte(address) << 8) as u16;
        let b = self.read_byte(address + 1) as u16;
        (a << 8) | b
    }

    /// Writes a byte of `value` to the `address`.
    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }
}
