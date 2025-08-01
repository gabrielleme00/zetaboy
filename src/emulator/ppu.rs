use crate::emulator::cpu::memory_bus::io_registers::IORegisters;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

const VRAM_SIZE: usize = 0x9FFF - 0x8000 + 1;
const OAM_SIZE: usize = 160;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PPUMode {
    HBlank,
    VBlank,
    OAMSearch,
    PixelTransfer,
}

pub struct PPU {
    pub buffer: Vec<u32>,
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
    bg_palette: [u8; 32],
    obj_palette: [u8; 32],
    pub mode: PPUMode,
    line: u8,
    dots: u16,
    pub interrupt: u8,
}

impl PPU {
    pub fn new() -> Self {
        let mut new_ppu = PPU {
            buffer: vec![0; WIDTH * HEIGHT],
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            bg_palette: [0; 32],
            obj_palette: [0; 32],
            mode: PPUMode::OAMSearch,
            line: 0,
            dots: 0,
            interrupt: 0,
        };
        // Fill the first tile with a simple pattern for testing
        for i in 0..8 {
            new_ppu.vram[i] = 0xAA; // Alternating bits
            new_ppu.vram[i + 8] = 0x55; // Alternating bits
        }
        new_ppu
    }

    pub fn read_vram(&self, address: u16) -> u8 {
        self.vram[(address - 0x8000) as usize]
    }

    pub fn write_vram(&mut self, address: u16, value: u8) -> Result<(), String> {
        if address >= 0x8000 && address < 0xA000 {
            self.vram[(address - 0x8000) as usize] = value;
            Ok(())
        } else {
            Err(format!("Warning: VRAM write out of bounds: {:#04X} = {:#02X}", address, value))
        }
    }

    pub fn read_oam(&self, address: u16) -> u8 {
        self.oam[address as usize]
    }

    pub fn write_oam(&mut self, address: u16, value: u8) -> Result<(), String> {
        if address < self.oam.len() as u16 {
            self.oam[address as usize] = value;
            Ok(())
        } else {
            Err(format!("Warning: OAM write out of bounds: {:#04X} = {:#02X}", address, value))
        }
    }

    pub fn read_bg_palette_ram(&self, address: u16) -> u8 {
        self.bg_palette[address as usize]
    }

    pub fn write_bg_palette_ram(&mut self, address: u16, value: u8) -> Result<(), String> {
        if address < self.bg_palette.len() as u16 {
            self.bg_palette[address as usize] = value;
            Ok(())
        } else {
            Err(format!("Warning: BG Palette RAM write out of bounds: {:#04X} = {:#02X}", address, value))
        }
    }

    pub fn read_obj_palette_ram(&self, address: u16) -> u8 {
        self.obj_palette[address as usize]
    }

    pub fn write_obj_palette_ram(&mut self, address: u16, value: u8) -> Result<(), String> {
        if address < self.obj_palette.len() as u16 {
            self.obj_palette[address as usize] = value;
            Ok(())
        } else {
            Err(format!("Warning: OBJ Palette RAM write out of bounds: {:#04X} = {:#02X}", address, value))
        }
    }

    pub fn step(&mut self, cycles: u8, bgp_value: u8, io_registers: &mut IORegisters) {
        self.dots += cycles as u16;
        io_registers.ly = self.line;
        io_registers.stat = (io_registers.stat & 0xFC) | (self.mode as u8);

        match self.mode {
            PPUMode::OAMSearch => {
                if self.dots >= 80 {
                    self.dots -= 80; // Carry over remaining dots to next mode
                    self.mode = PPUMode::PixelTransfer;
                }
            }
            PPUMode::PixelTransfer => {
                if self.dots >= 172 { // PixelTransfer takes 172 dots
                    self.dots -= 172; // Carry over remaining dots to next mode
                    self.mode = PPUMode::HBlank;
                    self.render_scanline(io_registers.lcdc, bgp_value);
                }
            }
            PPUMode::HBlank => {
                if self.dots >= 204 { // HBlank takes 204 dots (456 - 80 - 172)
                    self.dots = 0; // Reset dots for the next line
                    self.line += 1;

                    if self.line == 144 { // Line 144 is the first VBlank line
                        self.mode = PPUMode::VBlank;
                        self.interrupt |= 1; // VBlank interrupt
                    } else {
                        self.mode = PPUMode::OAMSearch;
                    }
                }
            }
            PPUMode::VBlank => {
                if self.dots >= 456 { // End of VBlank line
                    self.dots = 0; // Reset dots for the next line
                    self.line += 1;

                    if self.line >= 154 { // After line 153, go back to line 0 and OAMSearch
                        self.line = 0;
                        self.mode = PPUMode::OAMSearch;
                    }
                }
            }
        }
    }

    fn render_scanline(&mut self, lcdc: u8, bgp_value: u8) {
        // BG tile map selection
        let tile_map_addr = if (lcdc & 0x08) != 0 { 0x9C00 } else { 0x9800 };
        // BG tile data selection
        let tile_data_addr = if (lcdc & 0x10) != 0 { 0x8000 } else { 0x8800 };

        // SCY/SCX registers (scroll Y/X), for now hardcoded to 0
        let scy = 0u8;
        let scx = 0u8;

        let line_y = self.line.wrapping_add(scy);

        for x in 0..WIDTH {
            let pixel_x = x.wrapping_add(scx as usize);

            // Find which tile this pixel is in
            let tile_map_x = (pixel_x / 8) % 32;
            let tile_map_y = (line_y as usize / 8) % 32;

            let tile_map_index = tile_map_y * 32 + tile_map_x;
            let tile_index = self.read_vram(tile_map_addr + tile_map_index as u16);

            let tile_addr = if tile_data_addr == 0x8000 {
                tile_data_addr + (tile_index as u16) * 16
            } else {
                // tile_index is signed
                let idx = tile_index as i8 as i16;
                (tile_data_addr as i16 + (idx + 128) * 16) as u16
            };

            let tile_x = pixel_x % 8;
            let tile_y = line_y as usize % 8;

            let byte1 = self.read_vram(tile_addr + (tile_y as u16) * 2);
            let byte2 = self.read_vram(tile_addr + (tile_y as u16) * 2 + 1);

            let bit1 = (byte1 >> (7 - tile_x)) & 1;
            let bit2 = (byte2 >> (7 - tile_x)) & 1;

            let color_index = (bit2 << 1) | bit1;

            // Use BG palette (DMG: 4 shades of gray)
            let color_value = (bgp_value >> (color_index * 2)) & 0x03;

            let color = match color_value {
                0 => 0xFFFFFFFF, // White
                1 => 0xFFAAAAAA, // Light gray
                2 => 0xFF555555, // Dark gray
                3 => 0xFF000000, // Black
                _ => unreachable!(),
            };

            self.buffer[self.line as usize * WIDTH + x] = color;
        }
    }
}