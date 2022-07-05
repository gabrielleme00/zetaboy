// Memory Map (regions)

// 0x0000 - 0x3FFF : ROM Bank 0
// 0x4000 - 0x7FFF : ROM Bank 1 - Switchable

// 0x8000 - 0x97FF : CHR RAM  | ----
// 0x9800 - 0x9BFF : BG Map 1 | VRAM
// 0x9C00 - 0x9FFF : BG Map 2 | ----

// 0xA000 - 0xBFFF : Cartridge RAM

// 0xC000 - 0xCFFF : High RAM Bank 0
// 0xD000 - 0xDFFF : High RAM Bank 1-7 (switchable, Color only)

// 0xE000 - 0xFDFF : Echo RAM (Unusable)

// 0xFE00 - 0xFE9F : Object Attribute Memory

// 0xFEA0 - 0xFEFF : Reserved (Unusable)

// 0xFF00 - 0xFF7F : I/O Registers

// 0xFF80 - 0xFFFE : High RAM

// 0xFFFF : Interrupt Enable Register (IE)

pub const ROM_BEGIN: usize = 0x0000;
pub const ROM_END: usize = 0x7FFF;

pub const VRAM_BEGIN: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;

pub const CRAM_BEGIN: usize = 0x0A000;
pub const CRAM_END: usize = 0xBFFF;

pub const WRAM_BEGIN: usize = 0xC000;
pub const WRAM_END: usize = 0xDFFF;

pub const ECHO_BEGIN: usize = 0xE000;
pub const ECHO_END: usize = 0xFDFF;

pub const OAM_BEGIN: usize = 0xFE00;
pub const OAM_END: usize = 0xFE9F;

pub const RESERVED_BEGIN: usize = 0xFEA0;
pub const RESERVED_END: usize = 0xFEFF;

pub const IO_BEGIN: usize = 0xFF00;
pub const IO_END: usize = 0xFF7F;

pub const HRAM_BEGIN: usize = 0xFF80;
pub const HRAM_END: usize = 0xFFFE;

pub const IER: usize = 0xFFFF;

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
  IER,
}

impl MemoryRegion {
  pub fn from_address(address: usize) -> Self {
    match address {
      ROM_BEGIN..=ROM_END => Self::ROM,
      VRAM_BEGIN..=VRAM_END => Self::VRAM,
      CRAM_BEGIN..=CRAM_END => Self::CRAM,
      WRAM_BEGIN..=WRAM_END => Self::WRAM,
      ECHO_BEGIN..=ECHO_END => Self::ECHO,
      OAM_BEGIN..=OAM_END => Self::OAM,
      RESERVED_BEGIN..=RESERVED_END => Self::RESERVED,
      IO_BEGIN..=IO_END => Self::IO,
      HRAM_BEGIN..=HRAM_END => Self::HRAM,
      IER => Self::IER,
      _ => panic!("Invalid memory address: {}", address),
    }
  }
}
