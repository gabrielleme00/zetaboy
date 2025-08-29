mod lcdc;
mod sprite;

use crate::{emulator::cpu::memory_bus::io_registers::*, utils::bits::*};
use lcdc::LcdcData;
use sprite::{OAMSprite, RenderSprite, SPRITE_WIDTH};

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

const VRAM_SIZE: usize = 0x9FFF - 0x8000 + 1;
const OAM_SIZE: usize = 160;

const MAX_SPRITES_PER_SCANLINE: usize = 10;

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
    pub mode: PPUMode,
    pub dot_counter: u16,
    window_line: u8,
    bg_color_indices: Vec<u8>,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            buffer: vec![0; WIDTH * HEIGHT],
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            mode: PPUMode::OAMSearch,
            dot_counter: 0,
            window_line: 0,
            bg_color_indices: vec![0; WIDTH * HEIGHT],
        }
    }

    pub fn is_vblank(&self) -> bool {
        self.mode == PPUMode::VBlank
    }

    pub fn read_vram(&self, address: u16) -> u8 {
        if self.mode == PPUMode::PixelTransfer {
            return 0xFF;
        }
        self.vram[vram_index(address)]
    }

    pub fn write_vram(&mut self, address: u16, value: u8) {
        self.vram[vram_index(address)] = value;
    }

    pub fn can_use_oam(&self) -> bool {
        self.mode != PPUMode::PixelTransfer && self.mode != PPUMode::OAMSearch
    }

    pub fn read_oam(&self, address: u16) -> u8 {
        self.oam[(address - 0xFE00) as usize]
    }

    pub fn write_oam(&mut self, address: u16, value: u8) {
        self.oam[(address - 0xFE00) as usize] = value;
    }

    /// Reads two consecutive bytes from VRAM
    fn read_vram_pair(&self, addr: u16) -> (u8, u8) {
        let index = vram_index(addr);
        (self.vram[index], self.vram[index + 1])
    }

    pub fn tick(&mut self, io_registers: &mut IORegisters) {
        let previous_mode = self.mode;
        let previous_line = io_registers.read(REG_LY); // Read current LY before updating
        let lcdc = io_registers.read(REG_LCDC);

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
                    self.render_scanline(io_registers);
                }
                PPUMode::HBlank
            }
        };

        // Update the PPU's internal mode state
        self.mode = new_mode;

        // --- STAT Interrupt and Register Updates ---

        // Check for STAT interrupts on mode change
        if self.mode != previous_mode {
            self.check_stat_interrupts(io_registers);
        }

        // Update STAT register with the new mode and LYC=LY flag
        let ly_eq_lyc_flag = if io_registers.read(REG_LY) == io_registers.read(REG_LYC) {
            BIT_2
        } else {
            0
        };
        let stat_with_flags =
            (io_registers.read(REG_STAT) & 0b11111000) | (new_mode as u8) | ly_eq_lyc_flag;
        io_registers.force_write(REG_STAT, stat_with_flags);

        // Check for LYC=LY interrupt (this is also an edge-triggered condition)
        self.check_lyc(io_registers);
        self.check_stat_interrupts(io_registers);
    }

    pub fn check_lyc(&mut self, io_registers: &mut IORegisters) {
        let ly = io_registers.read(REG_LY);
        let lyc = io_registers.read(REG_LYC);
        let stat = io_registers.read(REG_STAT);

        let ly_eq_lyc = ly == lyc;
        let new_stat = if ly_eq_lyc {
            stat | BIT_2 // Set LYC=LY flag
        } else {
            stat & !BIT_2 // Clear LYC=LY flag
        };
        io_registers.force_write(REG_STAT, new_stat);
    }

    pub fn check_stat_interrupts(&mut self, io_registers: &mut IORegisters) {
        let stat = io_registers.read(REG_STAT);

        if (self.mode == PPUMode::HBlank && (stat & BIT_3) != 0)
            || (self.mode == PPUMode::VBlank && (stat & BIT_4) != 0)
            || (self.mode == PPUMode::OAMSearch && (stat & BIT_5) != 0)
            || (stat & BIT_6) & (stat & BIT_2) != 0
        {
            io_registers.request_interrupt(InterruptBit::LCDStat);
        }
    }

    fn render_scanline(&mut self, io_registers: &IORegisters) {
        let lcdc = io_registers.read(REG_LCDC);
        let scy = io_registers.read(REG_SCY);
        let scx = io_registers.read(REG_SCX);
        let ly = io_registers.read(REG_LY);
        let bgp = io_registers.read(REG_BGP);
        let wy = io_registers.read(REG_WY);
        let wx = io_registers.read(REG_WX);

        let lcdc_data = LcdcData::from(lcdc);

        // IF LCD is off, don't render
        if !lcdc_data.lcd_enable {
            return;
        }

        // IF BG is disabled, fill with 0th color
        if !lcdc_data.bg_enable {
            for x in 0..WIDTH {
                self.buffer[ly as usize * WIDTH + x] = self.get_output_color(0);
            }
            return;
        }

        let bg_tile_map_addr = if lcdc_data.bg_tile_map {
            0x9C00
        } else {
            0x9800
        };
        let win_tile_map_addr = if lcdc_data.window_tile_map {
            0x9C00
        } else {
            0x9800
        };

        // Calculate the effective line Y position
        let line_y = ly.wrapping_add(scy);

        // Render window if enabled and ly >= wy and WX in range
        let window_enabled = lcdc_data.window_enable;
        let window_visible = ly >= wy && wx <= 166 && wy <= 143;
        let mut window_x_counter = 0; // counts window pixels per line

        // For each pixel in the line
        for x in 0..WIDTH {
            let use_window = window_enabled && window_visible && (x as u8) >= wx.wrapping_sub(7);
            let (tile_map_addr, x_coord, y_coord) = if use_window {
                // Window coordinates
                let x_coord = window_x_counter;
                let y_coord = self.window_line as usize;
                window_x_counter += 1;
                (win_tile_map_addr, x_coord, y_coord)
            } else {
                // Background coordinates
                let x_coord = x.wrapping_add(scx as usize);
                let y_coord = line_y as usize;
                (bg_tile_map_addr, x_coord, y_coord)
            };
            let tile_map_x = (x_coord / 8) % 32;
            let tile_map_y = (y_coord / 8) % 32;
            let tile_x = x_coord % 8;
            let tile_y = y_coord % 8;

            let tile_map_index = tile_map_y * 32 + tile_map_x;
            let tile_id = self.vram[vram_index(tile_map_addr) + tile_map_index];
            let tile_addr = get_window_tile_address(tile_id, lcdc_data.bg_window_tile_data);

            let (byte1, byte2) = self.read_vram_pair(tile_addr + (tile_y as u16) * 2);
            let bit1 = (byte1 >> (7 - tile_x)) & BIT_0;
            let bit2 = (byte2 >> (7 - tile_x)) & BIT_0;
            let color_index = (bit2 << 1) | bit1;

            let dmg_color = self.get_dmg_color(bgp, color_index);
            let out_color = self.get_output_color(dmg_color);

            let buffer_index = ly as usize * WIDTH + x;

            self.buffer[buffer_index] = out_color;
            self.bg_color_indices[buffer_index] = color_index;
        }

        // --- Render Sprites (OBJ) ---
        if lcdc_data.obj_enable {
            self.render_sprites(io_registers);
        }
    }

    /// Returns a list of sprites that are visible on the current scanline.
    ///
    /// `ly` - The current scanline (0-143)
    ///
    /// `sprite_height` - The height of the sprite (8 or 16)
    fn get_visible_sprites(&self, ly: u8, sprite_height: u8) -> Vec<RenderSprite> {
        let mut visible_sprites = Vec::new();

        for i in 0..40 {
            let oam_sprite = OAMSprite::at_oam(&self.oam, i);
            let render_sprite = RenderSprite::from(oam_sprite);

            if render_sprite.is_visible_on_line(ly, sprite_height) {
                visible_sprites.push(render_sprite);
                if visible_sprites.len() >= MAX_SPRITES_PER_SCANLINE {
                    break;
                }
            }
        }

        visible_sprites
    }

    /// Renders sprites for the current scanline.
    fn render_sprites(&mut self, io_registers: &IORegisters) {
        let lcdc_data = LcdcData::from(io_registers.read(REG_LCDC));
        let ly = io_registers.read(REG_LY);

        let sprite_height: u8 = get_sprite_height(lcdc_data.obj_size);
        let visible_sprites = self.get_visible_sprites(ly, sprite_height);

        // Render sprites in order (lower OAM index has priority)
        for sprite in visible_sprites {
            let obp = io_registers.read(sprite.get_obp_address());
            let line_in_sprite = sprite.get_line_in_sprite(ly, sprite_height);

            // For 8x16 sprites, lower bit of tile ignored (hardware behavior)
            let (tile_num, line_in_tile) = if sprite_height == 16 {
                let top_half = line_in_sprite < 8;
                let tile_num = if top_half {
                    sprite.tile_index & 0xFE
                } else {
                    (sprite.tile_index & 0xFE).wrapping_add(1)
                };
                let line_in_tile = if top_half {
                    line_in_sprite
                } else {
                    line_in_sprite - 8
                };
                (tile_num, line_in_tile)
            } else {
                (sprite.tile_index, line_in_sprite)
            };

            let tile_addr = get_sprite_tile_address(tile_num) + (line_in_tile as u16) * 2;
            let (byte1, byte2) = self.read_vram_pair(tile_addr);

            for px in 0..SPRITE_WIDTH {
                let bit = if sprite.x_flip { px } else { 7 - px };
                let bit1 = (byte1 >> bit) & BIT_0;
                let bit2 = (byte2 >> bit) & BIT_0;
                let color_index = (bit2 << 1) | bit1;

                // Color index 0 is transparent for OBJ
                if color_index == 0 {
                    continue;
                }

                let (pixel_x, pixel_y) = (sprite.screen_x + px as i16, ly as i16);
                if is_pixel_out_of_bounds(pixel_x, pixel_y) {
                    continue;
                }

                let pixel_x_usize = pixel_x as usize;
                let pixel_y_usize = pixel_y as usize;

                // Get OBJ palette
                let dmg_color = self.get_dmg_color(obp, color_index);
                let out_color = self.get_output_color(dmg_color);
                let buffer_index = pixel_y_usize * WIDTH + pixel_x_usize;

                if sprite.bg_priority {
                    // Only draw over BG color 0
                    if self.bg_color_indices[buffer_index] == 0 {
                        self.buffer[buffer_index] = out_color;
                    }
                } else {
                    // Priority clear: OBJ always draws over BG
                    self.buffer[buffer_index] = out_color;
                }
            }
        }
    }

    fn get_dmg_color(&self, palette: u8, color_index: u8) -> u8 {
        (palette >> (color_index * 2)) & 0b11
    }

    /// Returns the output color value (for framebuffer) for a given color value (0 - 3).
    fn get_output_color(&self, dmg_color: u8) -> u32 {
        // let palette = [0xFFFFFFFF, 0xFFAAAAAA, 0xFF555555, 0xFF000000];
        let palette = [0xFF9A9E3F, 0xFF496B22, 0xFF0E450B, 0xFF1B2A09];
        palette[dmg_color as usize]
    }
}

/// Converts a VRAM address to an array index
fn vram_index(addr: u16) -> usize {
    (addr - 0x8000) as usize
}

/// Returns the height of a sprite based on the LCDC register.
/// 
/// LCDC bit 2: 0 = 8x8 sprites, 1 = 8x16 sprites
fn get_sprite_height(lcdc_obj_size: bool) -> u8 {
    if lcdc_obj_size {
        16
    } else {
        8
    }
}

/// Returns a tile's address in VRAM for a given ID and addressing mode.
///
/// LCDC bit 4: BG & Window tile data area.
fn get_window_tile_address(tile_id: u8, bg_window_tile_data: bool) -> u16 {
    if bg_window_tile_data {
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

/// Returns the tile address in VRAM for a given sprite tile ID.
fn get_sprite_tile_address(tile_num: u8) -> u16 {
    0x8000 + (tile_num as u16) * 16
}

/// Checks if a pixel is out of the screen bounds.
fn is_pixel_out_of_bounds(x: i16, y: i16) -> bool {
    x < 0 || x >= WIDTH as i16 || y < 0 || y >= HEIGHT as i16
}
