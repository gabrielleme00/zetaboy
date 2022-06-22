const MEMORY_SIZE: usize = 0xFFFF;

pub struct MemoryBus {
    memory: [u8; MEMORY_SIZE],
}

impl MemoryBus {
    pub fn new(cart_data: &Vec<u8>) -> Self {
        let mut memory = [0; MEMORY_SIZE];
        for i in 0..cart_data.len() {
            memory[i] = cart_data[i];
        }
        Self { memory }
    }

    /// Reads a byte from the `address`.
    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    /// Reads 2 bytes from the `address`.
    pub fn read_word(&self, address: u16) -> u16 {
        let a = (self.read_byte(address) as u16) << 8;
        let b = self.read_byte(address + 1) as u16;
        (a << 8) | b
    }

    /// Writes a byte of `value` to the `address`.
    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }
}
