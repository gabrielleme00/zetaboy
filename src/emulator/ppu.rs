use crate::{emulator::cpu::memory_bus::io_registers::*, utils::bits::*};

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
    // bg_palette: [u8; 32], // CGB
    // obj_palette: [u8; 32], // CGB
    pub mode: PPUMode,
    dot_counter: u16,
    window_line: u8,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            buffer: vec![0; WIDTH * HEIGHT],
            vram: [0; VRAM_SIZE],
            oam: [0xFF; OAM_SIZE],
            // bg_palette: [0; 32], // CGB
            // obj_palette: [0; 32], // CGB
            mode: PPUMode::OAMSearch,
            dot_counter: 0,
            window_line: 0,
        }
    }

    pub fn is_vblank(&self) -> bool {
        self.mode == PPUMode::VBlank
    }

    pub fn read_vram(&self, address: u16) -> u8 {
        if self.mode == PPUMode::PixelTransfer {
            return 0xFF;
        }
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
        if self.mode == PPUMode::PixelTransfer || self.mode == PPUMode::OAMSearch {
            return 0xFF;
        }

        if address < self.oam.len() as u16 {
            self.oam[address as usize]
        } else {
            self.oam[(address - 0xFE00) as usize]
        }
    }

    pub fn write_oam(&mut self, address: u16, value: u8) {
        if address < self.oam.len() as u16 {
            self.oam[address as usize] = value;
        } else {
            self.oam[(address - 0xFE00) as usize] = value;
        }
    }

    // pub fn read_bg_palette_ram(&self, address: u16) -> u8 {
    //     self.bg_palette[address as usize]
    // }

    // pub fn write_bg_palette_ram(&mut self, address: u16, value: u8) {
    //     if address < self.bg_palette.len() as u16 {
    //         self.bg_palette[address as usize] = value;
    //     } else {
    //         self.bg_palette[(address - 0xFF68) as usize] = value;
    //     }
    // }

    // pub fn read_obj_palette_ram(&self, address: u16) -> u8 {
    //     self.obj_palette[address as usize]
    // }

    // pub fn write_obj_palette_ram(&mut self, address: u16, value: u8) {
    //     if address < self.obj_palette.len() as u16 {
    //         self.obj_palette[address as usize] = value;
    //     } else {
    //         self.obj_palette[(address - 0xFF48) as usize] = value;
    //     }
    // }

    pub fn tick(&mut self, io_registers: &mut IORegisters) {
        let previous_mode = self.mode;
        let previous_line = io_registers.read(REG_LY); // Read current LY before updating
        let previous_ly_eq_lyc = previous_line == io_registers.read(REG_LYC);

        // --- Core PPU Clock & State Machine Logic ---
        self.dot_counter += 1;

        // Handle end-of-line
        let mut ly = previous_line;
        let mut new_scanline = false;
        if self.dot_counter >= 456 {
            self.dot_counter = 0;
            ly += 1;
            if ly >= 154 {
                ly = 0;
            }
            new_scanline = true;
        }

        io_registers.force_write(REG_LY, ly);

        // --- Window internal line counter logic ---
        // Reset window_line at start of frame (LY==0)
        if ly == 0 {
            self.window_line = 0;
        }
        // Only increment window_line if window is enabled and visible on this scanline
        // The window_line should increment BEFORE rendering the scanline
        if new_scanline && ly < 144 {
            let lcdc = io_registers.read(REG_LCDC);
            let wx = io_registers.read(REG_WX);
            let wy = io_registers.read(REG_WY);
            let window_enabled = (lcdc & BIT_5) != 0;
            let window_visible = ly >= wy && wx <= 166 && wy <= 143;
            if window_enabled && window_visible {
                // Window line counter only increments when window is actually rendered
                // Don't increment on the first line where window becomes visible
                if ly > wy {
                    self.window_line = self.window_line.wrapping_add(1);
                }
            } else if ly < wy {
                // Reset window line counter when we're above the window Y position
                self.window_line = 0;
            }
        }

        // --- Mode Determination and Interrupt Request ---

        // Determine the new mode based on the current line and dot counter
        let new_mode = if ly >= 144 {
            // V-Blank interrupt is requested ONCE, when line transitions to 144
            if previous_line < 144 && ly >= 144 {
                io_registers.request_interrupt(InterruptBit::VBlank);
            }
            PPUMode::VBlank // VBlank for ALL dots during lines 144-153
        } else {
            // Visible scanlines (0-143) - normal mode progression
            if self.dot_counter < 80 {
                PPUMode::OAMSearch
            } else if self.dot_counter < 252 {
                PPUMode::PixelTransfer
            } else {
                // The scanline is rendered when we *enter* H-Blank
                if previous_mode != PPUMode::HBlank {
                    let bgp = io_registers.read(0xFF47);
                    self.render_scanline(io_registers, bgp);
                }
                PPUMode::HBlank
            }
        };

        // Update the PPU's internal mode state
        self.mode = new_mode;

        // --- STAT Interrupt and Register Updates ---

        // Check for STAT interrupts (now edge-triggered)
        self.check_stat_interrupts(io_registers, previous_mode);

        // Update STAT register with the new mode and LYC=LY flag
        let ly_eq_lyc_flag = if io_registers.read(REG_LY) == io_registers.read(REG_LYC) {
            0b00000100
        } else {
            0
        };
        let stat_with_flags =
            (io_registers.read(REG_STAT) & 0b11111000) | (new_mode as u8) | ly_eq_lyc_flag;
        io_registers.force_write(REG_STAT, stat_with_flags);

        // Check for LYC=LY interrupt (this is also an edge-triggered condition)
        let new_ly_eq_lyc = io_registers.read(REG_LY) == io_registers.read(REG_LYC);
        if !previous_ly_eq_lyc && new_ly_eq_lyc {
            // The condition just became true, check if interrupt is enabled
            if (io_registers.read(REG_STAT) & 0b01000000) != 0 {
                io_registers.request_interrupt(InterruptBit::LCDStat);
            }
        }
    }

    fn check_stat_interrupts(&mut self, io_registers: &mut IORegisters, previous_mode: PPUMode) {
        let stat = io_registers.read(REG_STAT);

        if self.mode != previous_mode {
            if (self.mode == PPUMode::HBlank && (stat & BIT_3) != 0)
                || (self.mode == PPUMode::VBlank && (stat & BIT_4) != 0)
                || (self.mode == PPUMode::OAMSearch && (stat & BIT_5) != 0)
                || (stat & BIT_6) & (stat & BIT_2) != 0
            {
                io_registers.request_interrupt(InterruptBit::LCDStat);
            }
        }
    }

    fn render_scanline(&mut self, io_registers: &IORegisters, bgp_value: u8) {
        let ly = io_registers.read(REG_LY);
        let lcdc = io_registers.read(REG_LCDC);

        // IF LCD is off, don't render
        if (lcdc & BIT_7) == 0 {
            return;
        }

        // IF BG is disabled, fill with 0th color
        if (lcdc & BIT_0) == 0 {
            for x in 0..WIDTH {
                self.buffer[ly as usize * WIDTH + x] = self.get_final_color(0);
            }
            return;
        }

        // BG tile map selection (LCDC.3)
        let bg_tile_map_addr = if (lcdc & BIT_3) != 0 { 0x9C00 } else { 0x9800 };

        // Window tile map selection (LCDC.6)
        let win_tile_map_addr = if (lcdc & BIT_6) != 0 { 0x9C00 } else { 0x9800 };

        // SCY/SCX registers (scroll Y/X)
        let scy = io_registers.read(REG_SCY);
        let scx = io_registers.read(REG_SCX);

        // WX/WY registers
        let wx = io_registers.read(REG_WX);
        let wy = io_registers.read(REG_WY);

        // Calculate the effective line Y position
        let line_y = ly.wrapping_add(scy);

        // Window enabled if LCDC.5 is set and ly >= wy and WX in range
        let window_enabled = (lcdc & BIT_5) != 0;
        let window_visible = ly >= wy && wx <= 166 && wy <= 143;
        let mut window_x_counter = 0; // counts window pixels per line

        for x in 0..WIDTH {
            let use_window = window_enabled && window_visible && (x as u8) >= wx.wrapping_sub(7);
            let (tile_map_addr, tile_map_x, tile_map_y, tile_x, tile_y) = if use_window {
                // Window coordinates
                let win_x = window_x_counter;
                let win_y = self.window_line as usize;
                let tile_map_x = (win_x / 8) % 32;
                let tile_map_y = (win_y / 8) % 32;
                let tile_x = win_x % 8;
                let tile_y = win_y % 8;
                window_x_counter += 1;
                (win_tile_map_addr, tile_map_x, tile_map_y, tile_x, tile_y)
            } else {
                // Background coordinates
                let pixel_x = x.wrapping_add(scx as usize);
                let tile_map_x = (pixel_x / 8) % 32;
                let tile_map_y = (line_y as usize / 8) % 32;
                let tile_x = pixel_x % 8;
                let tile_y = line_y as usize % 8;
                (bg_tile_map_addr, tile_map_x, tile_map_y, tile_x, tile_y)
            };

            let tile_map_index = tile_map_y * 32 + tile_map_x;
            let tile_id = self.vram[(tile_map_addr - 0x8000 + tile_map_index as u16) as usize];
            let tile_addr = get_window_tile_address(tile_id, lcdc);
            let byte1 = self.vram[(tile_addr - 0x8000 + (tile_y as u16) * 2) as usize];
            let byte2 = self.vram[(tile_addr - 0x8000 + (tile_y as u16) * 2 + 1) as usize];
            let bit1 = (byte1 >> (7 - tile_x)) & 1;
            let bit2 = (byte2 >> (7 - tile_x)) & 1;
            let color_index = (bit2 << 1) | bit1;
            // Use BG palette (DMG: 4 shades of gray)
            let color_value = (bgp_value >> (color_index * 2)) & 0x03;
            let color = self.get_final_color(color_value);
            self.buffer[ly as usize * WIDTH + x] = color;
        }

        // --- Render Sprites (OBJ) ---
        if (lcdc & BIT_1) != 0 {
            self.render_sprites(lcdc, io_registers);
        }
    }

    /// Renders sprites for the current scanline, respecting priority, palette, and flipping.
    fn render_sprites(&mut self, lcdc: u8, io_registers: &IORegisters) {
        let ly = io_registers.read(REG_LY);
        let sprite_height = if (lcdc & BIT_2) != 0 { 16 } else { 8 };
        let mut sprites_on_line = Vec::new();

        // OAM: 4 bytes per sprite, 40 sprites max
        for i in 0..40 {
            let oam_base = i * 4;
            let y = self.oam[oam_base] as i16 - 16;
            let x = self.oam[oam_base + 1] as i16 - 8;
            let tile = self.oam[oam_base + 2];
            let attr = self.oam[oam_base + 3];

            // Is this sprite visible on the current line?
            if (ly as i16) >= y && (ly as i16) < y + sprite_height {
                sprites_on_line.push((x, y, tile, attr, i)); // Include OAM index
                if sprites_on_line.len() == 10 {
                    break; // Hardware limit: max 10 sprites per line
                }
            }
        }

        // Sort by X coordinate first, then by OAM index (lower index = higher priority)
        sprites_on_line.sort_by_key(|&(x, _, _, _, oam_index)| (x, oam_index));

        // Track which pixels have been drawn by sprites (for priority)
        let mut sprite_pixels = vec![false; WIDTH];

        // Render sprites in order (lower OAM index has priority)
        for &(x, y, tile, attr, _oam_index) in &sprites_on_line {
            let dmg_palette = (attr & 0x10) != 0;
            let x_flip = (attr & 0x20) != 0;
            let y_flip = (attr & 0x40) != 0;
            let priority = (attr & 0x80) != 0;

            // Get OBJ palette
            let obp_value = io_registers.read(if dmg_palette { REG_OBP1 } else { REG_OBP0 });

            // Sprite Y position is relative to the top of the screen
            let line_in_sprite = if y_flip {
                sprite_height - 1 - (ly as i16 - y)
            } else {
                ly as i16 - y
            } as u8;

            // For 8x16 sprites, lower bit of tile ignored (hardware behavior)
            let tile_num = if sprite_height == 16 {
                tile & 0xFE
            } else {
                tile
            };
            let tile_addr = get_window_tile_address(tile_num, lcdc) + (line_in_sprite as u16) * 2;
            let byte1 = self.vram[tile_addr as usize - 0x8000];
            let byte2 = self.vram[tile_addr as usize - 0x8000 + 1];

            for px in 0..8 {
                let bit = if x_flip { px } else { 7 - px };
                let bit1 = (byte1 >> bit) & 1;
                let bit2 = (byte2 >> bit) & 1;
                let color_index = (bit2 << 1) | bit1;

                let (screen_x, screen_y) = (x + px as i16, ly as i16);
                if is_pixel_out_of_bounds(screen_x, screen_y) {
                    continue;
                }

                let screen_x_usize = screen_x as usize;

                // Check if this pixel position has already been drawn by a higher priority sprite
                if sprite_pixels[screen_x_usize] {
                    continue;
                }

                // Color index 0 is transparent for OBJ, but still blocks lower priority sprites
                if color_index == 0 {
                    sprite_pixels[screen_x_usize] = true; // Mark as drawn (blocks lower priority)
                    continue;
                }

                let color_value = (obp_value >> (color_index * 2)) & 0x03;
                let color = self.get_final_color(color_value);
                let idx = screen_y as usize * WIDTH + screen_x_usize;

                // Priority: if OBJ has priority bit set (0x80), only draw over BG color 0
                // If priority bit is clear, OBJ always draws over BG
                if priority {
                    // Priority set: only draw over BG color 0 (white)
                    let bg_color = self.buffer[idx];
                    if bg_color == self.get_final_color(0) {
                        self.buffer[idx] = color;
                    }
                } else {
                    // Priority clear: OBJ always draws over BG
                    self.buffer[idx] = color;
                }

                // Mark this pixel as drawn by a sprite
                sprite_pixels[screen_x_usize] = true;
            }
        }
    }

    /// Returns the final color value (for framebuffer) for a given color value (0 - 3).
    fn get_final_color(&self, color_value: u8) -> u32 {
        // let palette = [0xFFFFFFFF, 0xFFAAAAAA, 0xFF555555, 0xFF000000];
        let palette = [0xFF9A9E3F, 0xFF496B22, 0xFF0E450B, 0xFF1B2A09];
        palette[color_value as usize]
    }
}

/// Returns the tile address in VRAM for a given tile ID and LCDC register.
///
/// LCDC bit 4: BG & Window tile data area.
fn get_window_tile_address(tile_id: u8, lcdc: u8) -> u16 {
    if (lcdc >> 4) & 0b1 != 0 {
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

/// Checks if a pixel is out of the screen bounds.
fn is_pixel_out_of_bounds(x: i16, y: i16) -> bool {
    x < 0 || x >= WIDTH as i16 || y < 0 || y >= HEIGHT as i16
}
