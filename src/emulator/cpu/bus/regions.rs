// Memory Map (regions)
// 0x0000 - 0x3FFF : ROM Bank 0
// 0x4000 - 0x7FFF : ROM Bank 1 - Switchable
// 0x8000 - 0x97FF : CHR RAM
// 0x9800 - 0x9BFF : BG Map 1
// 0x9C00 - 0x9FFF : BG Map 2
// 0xA000 - 0xBFFF : Cartridge RAM
// 0xC000 - 0xCFFF : High RAM Bank 0
// 0xD000 - 0xDFFF : High RAM Bank 1-7 (switchable, Color only)
// 0xE000 - 0xFDFF : Echo RAM (Unusable)
// 0xFE00 - 0xFE9F : Object Attribute Memory
// 0xFEA0 - 0xFEFF : Reserved (Unusable)
// 0xFF00 - 0xFF7F : I/O Registers
// 0xFF80 - 0xFFFE : High RAM
// 0xFFFF : Interrupt Enable Register (IE)

pub enum MemoryRegion {
  ROM,
  VRAM,
  CRAM,
  WRAM,
  ECHO,
  OAM,
  RESERVED,
  IO,
  HRAM,
  IE,
}

impl MemoryRegion {
  pub fn from_address(address: usize) -> Self {
    match address {
      0x0000..=0x7FFF => Self::ROM,
      0x8000..=0x9FFF => Self::VRAM,
      0xA000..=0xBFFF => Self::CRAM,
      0xC000..=0xDFFF => Self::WRAM,
      0xE000..=0xFDFF => Self::ECHO,
      0xFE00..=0xFE9F => Self::OAM,
      0xFEA0..=0xFEFF => Self::RESERVED,
      0xFF00..=0xFF7F => Self::IO,
      0xFF80..=0xFFFE => Self::HRAM,
      0xFFFF => Self::IE,
      _ => panic!("Invalid memory address: {}", address),
    }
  }
}
