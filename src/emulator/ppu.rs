use crate::emulator::cpu::memory_bus::io_registers::IORegisters;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

const VRAM_SIZE: usize = 0x9FFF - 0x8000 + 1;
const OAM_SIZE: usize = 160;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PPUMode {
    HBlank = 0,
    VBlank = 1,
    OAMSearch = 2,
    PixelTransfer = 3,
}

pub struct PPU {
    pub buffer: Vec<u32>,
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
    bg_palette: [u8; 32],
    obj_palette: [u8; 32],
    pub mode: PPUMode,
    line: u8, // LY
    dot_counter: u16,
    pub int: u8,
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
            dot_counter: 0,
            int: 0,
        };
        // Fill the first tile with a simple pattern for testing
        for i in 0..8 {
            new_ppu.vram[i] = 0xAA; // Alternating bits
            new_ppu.vram[i + 8] = 0x55; // Alternating bits
        }
        new_ppu
    }

    pub fn is_vblank(&self) -> bool {
        self.mode == PPUMode::VBlank
    }

    pub fn read_vram(&self, address: u16) -> u8 {
        self.vram[(address - 0x8000) as usize]
    }

    pub fn write_vram(&mut self, address: u16, value: u8) {
        if address >= 0x8000 && address < 0xA000 {
            self.vram[(address - 0x8000) as usize] = value;
        } else {
            panic!(
                "Warning: VRAM write out of bounds: {:#04X} = {:#02X}",
                address, value
            );
        }
    }

    pub fn read_oam(&self, address: u16) -> u8 {
        self.oam[address as usize]
    }

    pub fn write_oam(&mut self, address: u16, value: u8) {
        if address < self.oam.len() as u16 {
            self.oam[address as usize] = value;
        } else {
            panic!(
                "Warning: OAM write out of bounds: {:#04X} = {:#02X}",
                address, value
            );
        }
    }

    pub fn read_bg_palette_ram(&self, address: u16) -> u8 {
        self.bg_palette[address as usize]
    }

    pub fn write_bg_palette_ram(&mut self, address: u16, value: u8) {
        if address < self.bg_palette.len() as u16 {
            self.bg_palette[address as usize] = value;
        } else {
            self.bg_palette[(address - 0xFF68) as usize] = value;
        }
    }

    pub fn read_obj_palette_ram(&self, address: u16) -> u8 {
        self.obj_palette[address as usize]
    }

    pub fn write_obj_palette_ram(&mut self, address: u16, value: u8) {
        if address < self.obj_palette.len() as u16 {
            self.obj_palette[address as usize] = value;
        } else {
            panic!(
                "Warning: OBJ Palette RAM write out of bounds: {:#04X} = {:#02X}",
                address, value
            );
        }
    }

    pub fn step(&mut self, cpu_cycles: u8, bgp_value: u8, io_registers: &mut IORegisters) {
        let ppu_dots = (cpu_cycles as u16) * 4;

        let previous_mode = self.mode;
        let previous_line = self.line;
        let previous_ly_eq_lyc = io_registers.ly == io_registers.lyc;

        // --- Core PPU Clock & State Machine Logic ---
        self.dot_counter += ppu_dots;

        // Handle end-of-line
        if self.dot_counter >= 456 {
            self.dot_counter -= 456;
            self.line += 1;
            if self.line >= 154 {
                self.line = 0;
            }
        }

        // Update LY register here, right before we need it
        io_registers.ly = self.line;

        // --- Mode Determination and Interrupt Request ---

        // Determine the new mode based on the current line and dot counter
        let new_mode = if self.line >= 144 {
            // V-Blank interrupt is requested ONCE, when line transitions to 144
            if previous_line == 143 {
                self.int |= 0b1;
            }
            PPUMode::VBlank
        } else {
            // H-Blank Mode
            if self.dot_counter >= 252 {
                // The scanline is rendered when we *enter* H-Blank
                if previous_mode != PPUMode::HBlank {
                    self.render_scanline(io_registers, bgp_value);
                }
                // 80 (Mode 2) + 172 (Mode 3)
                PPUMode::HBlank
            // Pixel Transfer Mode
            } else if self.dot_counter >= 80 {
                PPUMode::PixelTransfer
            // OAM Search Mode
            } else {
                PPUMode::OAMSearch
            }
        };

        // Update the PPU's internal mode state
        self.mode = new_mode;

        // --- STAT Interrupt and Register Updates ---

        // Check for STAT interrupts (now edge-triggered)
        self.check_stat_interrupts(io_registers, previous_mode);

        // Update STAT register with the new mode and LYC=LY flag
        let ly_eq_lyc_flag = if io_registers.ly == io_registers.lyc {
            0b00000100
        } else {
            0
        };
        let stat_with_flags = (io_registers.stat & 0b11111000) | (new_mode as u8) | ly_eq_lyc_flag;
        io_registers.stat = stat_with_flags;

        // Check for LYC=LY interrupt (this is also an edge-triggered condition)
        let new_ly_eq_lyc = io_registers.ly == io_registers.lyc;
        if !previous_ly_eq_lyc && new_ly_eq_lyc {
            // The condition just became true, check if interrupt is enabled
            if (io_registers.stat & 0b01000000) != 0 {
                self.int |= 0b10;
            }
        }
    }

    fn check_stat_interrupts(&mut self, io_registers: &mut IORegisters, previous_mode: PPUMode) {
        let stat = io_registers.stat;
        let current_mode = self.mode;

        // This is the core of the edge-triggered logic.
        // We only request an interrupt when we've just entered a mode
        // and that mode's interrupt is enabled.

        if current_mode != previous_mode {
            // H-Blank Interrupt
            if current_mode == PPUMode::HBlank && (stat & 0b00001000) != 0 {
                self.int |= 0b10;
            }
            // V-Blank Interrupt
            else if current_mode == PPUMode::VBlank && (stat & 0b00010000) != 0 {
                self.int |= 0b10;
            }
            // OAM Search Interrupt
            else if current_mode == PPUMode::OAMSearch && (stat & 0b00100000) != 0 {
                self.int |= 0b10;
            }
        }
    }

    fn render_scanline(&mut self, io_registers: &IORegisters, bgp_value: u8) {
        let lcdc = io_registers.lcdc;

        // BG tile map selection
        let tile_map_addr = if (lcdc & 0x08) != 0 { 0x9C00 } else { 0x9800 };

        // SCY/SCX registers (scroll Y/X)
        let scy = io_registers.scy;
        let scx = io_registers.scx;

        // Calculate the effective line Y position
        let line_y = self.line.wrapping_add(scy);

        // --- Render Background ---
        for x in 0..WIDTH {
            let pixel_x = x.wrapping_add(scx as usize);

            // Find which tile this pixel is in
            let tile_map_x = (pixel_x / 8) % 32;
            let tile_map_y = (line_y as usize / 8) % 32;

            let tile_map_index = tile_map_y * 32 + tile_map_x;
            let tile_id = self.read_vram(tile_map_addr + tile_map_index as u16);

            let tile_addr = self.get_tile_address(tile_id, lcdc);

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

        // --- Render Sprites (OBJ) ---
        if (lcdc & 0x02) != 0 {
            self.render_sprites(lcdc);
        }
    }

    /// Renders sprites for the current scanline, respecting priority, palette, and flipping.
    fn render_sprites(&mut self, lcdc: u8) {
        let sprite_height = if (lcdc & 0x04) != 0 { 16 } else { 8 };
        let mut sprites_on_line = Vec::new();

        // OAM: 4 bytes per sprite, 40 sprites max
        for i in 0..40 {
            let oam_base = i * 4;
            let y = self.oam[oam_base] as i16 - 16;
            let x = self.oam[oam_base + 1] as i16 - 8;
            let tile = self.oam[oam_base + 2];
            let attr = self.oam[oam_base + 3];

            // Is this sprite visible on the current line?
            if (self.line as i16) >= y && (self.line as i16) < y + sprite_height {
                sprites_on_line.push((x, y, tile, attr));
                if sprites_on_line.len() == 10 {
                    break; // Hardware limit: max 10 sprites per line
                }
            }
        }

        // Lower X coordinate has priority (hardware behavior)
        sprites_on_line.sort_by_key(|&(x, _, _, _)| x);

        for &(x, y, tile, attr) in &sprites_on_line {
            let palette = if (attr & 0x10) != 0 { self.obj_palette[1] } else { self.obj_palette[0] };
            let x_flip = (attr & 0x20) != 0;
            let y_flip = (attr & 0x40) != 0;
            let priority = (attr & 0x80) != 0;

            let line_in_sprite = if y_flip {
                sprite_height - 1 - (self.line as i16 - y)
            } else {
                self.line as i16 - y
            } as u8;

            // For 8x16 sprites, lower bit of tile ignored (hardware behavior)
            let tile_num = if sprite_height == 16 { tile & 0xFE } else { tile };
            let tile_addr = self.get_tile_address(tile_num, lcdc) + (line_in_sprite as u16) * 2;
            let byte1 = self.read_vram(tile_addr);
            let byte2 = self.read_vram(tile_addr + 1);

            for px in 0..8 {
                let bit = if x_flip { px } else { 7 - px };
                let bit1 = (byte1 >> bit) & 1;
                let bit2 = (byte2 >> bit) & 1;
                let color_index = (bit2 << 1) | bit1;

                // Color index 0 is transparent for OBJ
                if color_index == 0 {
                    continue;
                }

                let color_value = (palette >> (color_index * 2)) & 0x03;
                let color = match color_value {
                    0 => 0xFFFFFFFF, // White
                    1 => 0xFFAAAAAA, // Light gray
                    2 => 0xFF555555, // Dark gray
                    3 => 0xFF000000, // Black
                    _ => unreachable!(),
                };

                let screen_x = x + px as i16;
                let screen_y = self.line as i16;
                if screen_x < 0 || screen_x >= WIDTH as i16 || screen_y < 0 || screen_y >= HEIGHT as i16 {
                    continue;
                }

                let idx = screen_y as usize * WIDTH + screen_x as usize;

                // Priority: if OBJ has priority, only draw if BG color is 0
                if priority {
                    let bg_color = self.buffer[idx];
                    if bg_color == 0xFFFFFFFF {
                        self.buffer[idx] = color;
                    }
                } else {
                    self.buffer[idx] = color;
                }
            }
        }
    }

    fn get_tile_address(&self, tile_id: u8, lcdc: u8) -> u16 {
        if (lcdc >> 4) & 0b1 == 1 {
            let base_addr: u16 = 0x8000;
            // Unsigned addressing (0x8000-0x8FFF)
            base_addr + (tile_id as u16 * 16)
        } else {
            let base_addr: u16 = 0x9000;
            // Signed addressing (0x8800-0x97FF)
            let signed_id = tile_id as i8;
            let offset: i16 = signed_id as i16 * 16;
            (base_addr as i16 + offset) as u16
        }
    }
}
