mod lcdc;
mod sprite;

use crate::{emulator::cpu::memory_bus::InterruptBit, utils::bits::*};
use lcdc::LcdcData;
use serde::{Deserialize, Serialize};
use sprite::{OAMSprite, RenderSprite, SPRITE_WIDTH};

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

const VRAM_SIZE: usize = 0x9FFF - 0x8000 + 1;
const OAM_SIZE: usize = 160;
const VRAM_BANKS: usize = 2; // CGB has 2 VRAM banks
const BG_PALETTE_SIZE: usize = 64; // 8 palettes * 4 colors * 2 bytes
const OBJ_PALETTE_SIZE: usize = 64; // 8 palettes * 4 colors * 2 bytes

const MAX_SPRITES_PER_SCANLINE: usize = 10;

const REG_LCDC: u16 = 0xFF40;
const REG_STAT: u16 = 0xFF41;
const REG_SCY: u16 = 0xFF42;
const REG_SCX: u16 = 0xFF43;
const REG_LY: u16 = 0xFF44;
const REG_LYC: u16 = 0xFF45;
const REG_BGP: u16 = 0xFF47;
const REG_OBP0: u16 = 0xFF48;
const REG_OBP1: u16 = 0xFF49;
const REG_WY: u16 = 0xFF4A;
const REG_WX: u16 = 0xFF4B;

#[derive(PartialEq, Clone, Copy, Debug, Deserialize, Serialize)]
pub enum PPUMode {
    HBlank = 0,
    VBlank = 1,
    OAMSearch = 2,
    PixelTransfer = 3,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PPU {
    #[serde(skip)]
    pub buffer: Vec<u32>,
    vram: Vec<Vec<u8>>, // 2 banks of VRAM for CGB
    #[serde(with = "serde_arrays")]
    oam: [u8; OAM_SIZE],
    pub mode: PPUMode,
    pub dot_counter: u16,
    window_line: u8,
    #[serde(skip)]
    bg_color_indices: Vec<u8>,
    // CGB-specific fields
    pub cgb_mode: bool,
    vram_bank: usize,
    bg_palette_data: Vec<u8>,
    bg_palette_index: u8,
    bg_palette_auto_increment: bool,
    obj_palette_data: Vec<u8>,
    obj_palette_index: u8,
    obj_palette_auto_increment: bool,
    // PPU registers
    lcdc: u8,  // 0xFF40
    stat: u8,  // 0xFF41
    scy: u8,   // 0xFF42
    scx: u8,   // 0xFF43
    ly: u8,    // 0xFF44
    lyc: u8,   // 0xFF45
    bgp: u8,   // 0xFF47
    obp0: u8,  // 0xFF48
    obp1: u8,  // 0xFF49
    wy: u8,    // 0xFF4A
    wx: u8,    // 0xFF4B
}

impl PPU {
    pub fn new() -> Self {
        Self {
            buffer: vec![0; WIDTH * HEIGHT],
            vram: vec![vec![0; VRAM_SIZE]; VRAM_BANKS],
            oam: [0; OAM_SIZE],
            mode: PPUMode::OAMSearch,
            dot_counter: 0,
            window_line: 0,
            bg_color_indices: vec![0; WIDTH * HEIGHT],
            cgb_mode: false,
            vram_bank: 0,
            bg_palette_data: vec![0; BG_PALETTE_SIZE],
            bg_palette_index: 0,
            bg_palette_auto_increment: false,
            obj_palette_data: vec![0; OBJ_PALETTE_SIZE],
            obj_palette_index: 0,
            obj_palette_auto_increment: false,
            // Initialize PPU registers with boot values
            lcdc: 0x91,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0xFC,
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,
        }
    }

    pub fn set_cgb_mode(&mut self, enabled: bool) {
        self.cgb_mode = enabled;
        
        // Initialize CGB palettes with DMG-equivalent colors if enabling CGB mode
        if enabled {
            self.init_default_cgb_palettes();
        }
    }

    /// Reinitialize transient buffers after deserialization (since they're skipped in save states)
    pub fn reinit_buffers(&mut self) {
        if self.buffer.is_empty() {
            self.buffer = vec![0; WIDTH * HEIGHT];
        }
        if self.bg_color_indices.is_empty() {
            self.bg_color_indices = vec![0; WIDTH * HEIGHT];
        }
    }
    
    // PPU register read/write methods
    pub fn read_register(&self, address: u16) -> u8 {
        match address {
            REG_LCDC => self.lcdc,
            REG_STAT => self.stat,
            REG_SCY => self.scy,
            REG_SCX => self.scx,
            REG_LY => self.ly,
            REG_LYC => self.lyc,
            REG_BGP => self.bgp,
            REG_OBP0 => self.obp0,
            REG_OBP1 => self.obp1,
            REG_WY => self.wy,
            REG_WX => self.wx,
            _ => 0xFF,
        }
    }
    
    pub fn write_register(&mut self, address: u16, value: u8) {
        match address {
            REG_LCDC => self.lcdc = value,
            REG_STAT => {
                // Bits 0-2 are read-only (mode and LYC=LY flag)
                self.stat = (self.stat & 0x87) | (value & 0x78);
            }
            REG_SCY => self.scy = value,
            REG_SCX => self.scx = value,
            REG_LY => {} // LY is read-only
            REG_LYC => self.lyc = value,
            REG_BGP => self.bgp = value,
            REG_OBP0 => self.obp0 = value,
            REG_OBP1 => self.obp1 = value,
            REG_WY => self.wy = value,
            REG_WX => self.wx = value,
            _ => {}
        }
    }
    
    pub fn force_write_register(&mut self, address: u16, value: u8) {
        match address {
            REG_LCDC => self.lcdc = value,
            REG_STAT => self.stat = value,
            REG_SCY => self.scy = value,
            REG_SCX => self.scx = value,
            REG_LY => self.ly = value,
            REG_LYC => self.lyc = value,
            REG_BGP => self.bgp = value,
            REG_OBP0 => self.obp0 = value,
            REG_OBP1 => self.obp1 = value,
            REG_WY => self.wy = value,
            REG_WX => self.wx = value,
            _ => {}
        }
    }
    
    fn init_default_cgb_palettes(&mut self) {
        // Initialize with a simple visible grayscale palette so we can see if anything is rendering
        // Games will overwrite these with their own palettes
        // Using more visible colors: white, light gray, dark gray, black
        let default_colors_rgb555: [u16; 4] = [
            0x7FFF, // Color 0: White (31, 31, 31)
            0x5294, // Color 1: Light gray (20, 20, 20)
            0x294A, // Color 2: Dark gray (10, 10, 10)
            0x0000, // Color 3: Black (0, 0, 0)
        ];
        
        // Initialize all 8 BG palettes
        for palette_num in 0..8 {
            for color_idx in 0..4 {
                let base_index = (palette_num * 8 + color_idx * 2) as usize;
                let rgb555 = default_colors_rgb555[color_idx];
                self.bg_palette_data[base_index] = (rgb555 & 0xFF) as u8;
                self.bg_palette_data[base_index + 1] = ((rgb555 >> 8) & 0xFF) as u8;
            }
        }
        
        // Initialize all 8 OBJ palettes
        for palette_num in 0..8 {
            for color_idx in 0..4 {
                let base_index = (palette_num * 8 + color_idx * 2) as usize;
                let rgb555 = default_colors_rgb555[color_idx];
                self.obj_palette_data[base_index] = (rgb555 & 0xFF) as u8;
                self.obj_palette_data[base_index + 1] = ((rgb555 >> 8) & 0xFF) as u8;
            }
        }
    }

    // VRAM Bank register (VBK - 0xFF4F)
    pub fn read_vram_bank(&self) -> u8 {
        if !self.cgb_mode {
            return 0xFF;
        }
        self.vram_bank as u8 | 0xFE // Bit 0 only is used
    }

    pub fn write_vram_bank(&mut self, value: u8) {
        if !self.cgb_mode {
            return;
        }
        self.vram_bank = (value & 0x01) as usize;
    }

    // BG Palette Spec register (BCPS/BGPI - 0xFF68)
    pub fn read_bg_palette_index(&self) -> u8 {
        if !self.cgb_mode {
            return 0xFF;
        }
        self.bg_palette_index | if self.bg_palette_auto_increment { 0x80 } else { 0x00 }
    }

    pub fn write_bg_palette_index(&mut self, value: u8) {
        if !self.cgb_mode {
            return;
        }
        self.bg_palette_index = value & 0x3F; // 6 bits for index (0-63)
        self.bg_palette_auto_increment = (value & 0x80) != 0;
    }

    // BG Palette Data register (BCPD/BGPD - 0xFF69)
    pub fn read_bg_palette_data(&self) -> u8 {
        if !self.cgb_mode {
            return 0xFF;
        }
        self.bg_palette_data[self.bg_palette_index as usize]
    }

    pub fn write_bg_palette_data(&mut self, value: u8) {
        if !self.cgb_mode {
            return;
        }
        self.bg_palette_data[self.bg_palette_index as usize] = value;
        if self.bg_palette_auto_increment {
            self.bg_palette_index = (self.bg_palette_index + 1) & 0x3F;
        }
    }

    // OBJ Palette Spec register (OCPS/OBPI - 0xFF6A)
    pub fn read_obj_palette_index(&self) -> u8 {
        if !self.cgb_mode {
            return 0xFF;
        }
        self.obj_palette_index | if self.obj_palette_auto_increment { 0x80 } else { 0x00 }
    }

    pub fn write_obj_palette_index(&mut self, value: u8) {
        if !self.cgb_mode {
            return;
        }
        self.obj_palette_index = value & 0x3F;
        self.obj_palette_auto_increment = (value & 0x80) != 0;
    }

    // OBJ Palette Data register (OCPD/OBPD - 0xFF6B)
    pub fn read_obj_palette_data(&self) -> u8 {
        if !self.cgb_mode {
            return 0xFF;
        }
        self.obj_palette_data[self.obj_palette_index as usize]
    }

    pub fn write_obj_palette_data(&mut self, value: u8) {
        if !self.cgb_mode {
            return;
        }
        self.obj_palette_data[self.obj_palette_index as usize] = value;
        if self.obj_palette_auto_increment {
            self.obj_palette_index = (self.obj_palette_index + 1) & 0x3F;
        }
    }

    pub fn read_vram(&self, address: u16) -> u8 {
        if self.mode == PPUMode::PixelTransfer {
            return 0xFF;
        }
        self.vram[self.vram_bank][vram_index(address)]
    }

    pub fn write_vram(&mut self, address: u16, value: u8) {
        self.vram[self.vram_bank][vram_index(address)] = value;
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
        (self.vram[self.vram_bank][index], self.vram[self.vram_bank][index + 1])
    }

    pub fn is_lcd_enabled(&self) -> bool {
        (self.lcdc & BIT_7) != 0
    }

    pub fn tick(&mut self, interrupt_flag: &mut u8) {
        let lcdc_data = LcdcData::from(self.lcdc);

        // If LCD is off, do nothing
        if !lcdc_data.lcd_enable {
            return;
        }

        let previous_mode = self.mode;
        let previous_line = self.ly;

        // --- Core PPU Clock & State Machine Logic ---
        self.dot_counter += 1;

        // Handle end-of-line
        let mut ly = previous_line;
        let new_scanline = self.dot_counter == 456;
        if new_scanline {
            self.dot_counter = 0;
            ly += 1;
            if ly == 154 {
                ly = 0;
            }
            self.ly = ly;
        }

        // --- Window internal line counter logic ---
        // Reset window_line at start of frame (LY==0)
        if ly == 0 {
            self.window_line = 0;
        }
        // Only increment window_line if window is enabled and visible on this scanline
        // The window_line should increment BEFORE rendering the scanline
        if new_scanline && ly < 144 {
            let window_enabled = lcdc_data.window_enable;
            let window_visible = ly >= self.wy && self.wx <= 166 && self.wy <= 143;
            if window_enabled && window_visible {
                // Window line counter only increments when window is actually rendered
                // Don't increment on the first line where window becomes visible
                if ly > self.wy {
                    self.window_line += 1;
                }
            }
        }

        // --- Mode Determination and Interrupt Request ---

        // Determine the new mode based on the current line and dot counter
        let new_mode = if ly >= 144 {
            // V-Blank interrupt is requested ONCE, when line transitions to 144
            if previous_line == 143 && ly == 144 {
                *interrupt_flag |= InterruptBit::VBlank as u8;
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
                    self.render_scanline();
                }
                PPUMode::HBlank
            }
        };

        // Update the PPU's internal mode state
        self.mode = new_mode;

        // --- STAT Interrupt and Register Updates ---

        // Update STAT register with the new mode and LYC=LY flag
        let ly_eq_lyc_flag = if ly == self.lyc { BIT_2 } else { 0 };

        let stat_with_flags = (self.stat & 0xF8) | (new_mode as u8) | ly_eq_lyc_flag;
        self.stat = stat_with_flags;

        if self.mode != previous_mode {
            self.check_stat_interrupts(interrupt_flag);
        }
    }

    pub fn check_lyc(&mut self) {
        let ly_eq_lyc = self.ly == self.lyc;
        let new_stat = if ly_eq_lyc {
            self.stat | BIT_2 // Set LYC=LY flag
        } else {
            self.stat & !BIT_2 // Clear LYC=LY flag
        };
        self.stat = new_stat;
    }

    pub fn check_stat_interrupts(&mut self, interrupt_flag: &mut u8) {
        if (self.mode == PPUMode::HBlank && (self.stat & BIT_3) != 0)
            || (self.mode == PPUMode::VBlank && (self.stat & BIT_4) != 0)
            || (self.mode == PPUMode::OAMSearch && (self.stat & BIT_5) != 0)
            || ((self.stat & BIT_6) != 0 && (self.stat & BIT_2) != 0)
        {
            *interrupt_flag |= InterruptBit::LCDStat as u8;
        }
    }

    fn render_scanline(&mut self) {
        let lcdc_data = LcdcData::from(self.lcdc);

        // IF LCD is off, don't render
        if !lcdc_data.lcd_enable {
            return;
        }

        // IF BG is disabled, fill with 0th color
        if !lcdc_data.bg_enable {
            for x in 0..WIDTH {
                self.buffer[self.ly as usize * WIDTH + x] = self.get_dmg_color(self.bgp, 0);
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

        // Render window if enabled and ly >= wy and WX in range
        let window_enabled = lcdc_data.window_enable;
        let window_visible = self.ly >= self.wy && self.wx <= 166 && self.wy <= 143;
        let mut window_x_counter = 0; // counts window pixels per line

        // For each pixel in the line
        for x in 0..WIDTH {
            let use_window = window_enabled && window_visible && (x as u8) >= self.wx.saturating_sub(7);
            let (tile_map_addr, x_coord, y_coord) = if use_window {
                // Window coordinates
                let x_coord = window_x_counter;
                let y_coord = self.window_line as usize;
                window_x_counter += 1;
                (win_tile_map_addr, x_coord, y_coord)
            } else {
                // Background coordinates
                let x_coord = x.wrapping_add(self.scx as usize);
                let y_coord = self.ly.wrapping_add(self.scy) as usize;
                (bg_tile_map_addr, x_coord, y_coord)
            };
            let tile_map_x = (x_coord / 8) % 32;
            let tile_map_y = (y_coord / 8) % 32;
            let tile_x = x_coord % 8;
            let tile_y = y_coord % 8;

            let tile_map_index = tile_map_y * 32 + tile_map_x;
            let tile_id_addr = tile_map_addr + tile_map_index as u16;
            let tile_id = self.vram[0][vram_index(tile_id_addr)];
            
            // CGB: Read tile attributes from VRAM bank 1
            let (palette_num, vram_bank, x_flip, y_flip, _bg_priority) = if self.cgb_mode {
                let attr = self.vram[1][vram_index(tile_id_addr)];
                let palette_num = attr & 0x07;
                let vram_bank = ((attr & 0x08) >> 3) as usize;
                let x_flip = (attr & 0x20) != 0;
                let y_flip = (attr & 0x40) != 0;
                let bg_priority = (attr & 0x80) != 0;
                (palette_num, vram_bank, x_flip, y_flip, bg_priority)
            } else {
                (0, 0, false, false, false)
            };
            
            // Apply Y flip
            let flipped_tile_y = if y_flip { 7 - tile_y } else { tile_y };
            
            // Get tile address (use correct VRAM bank for CGB)
            let tile_addr = get_tile_address(tile_id, lcdc_data.bg_window_tile_data);
            let tile_data_addr = tile_addr + (flipped_tile_y as u16) * 2;
            
            // Read tile data from the correct VRAM bank
            let (byte1, byte2) = if self.cgb_mode {
                let index = vram_index(tile_data_addr);
                (self.vram[vram_bank][index], self.vram[vram_bank][index + 1])
            } else {
                self.read_vram_pair(tile_data_addr)
            };
            
            // Apply X flip
            let flipped_tile_x = if x_flip { 7 - tile_x } else { tile_x };
            
            let bit1 = (byte1 >> (7 - flipped_tile_x)) & BIT_0;
            let bit2 = (byte2 >> (7 - flipped_tile_x)) & BIT_0;
            let color_index = (bit2 << 1) | bit1;

            // Get the final color
            let out_color = if self.cgb_mode {
                self.get_cgb_color(palette_num, color_index, false)
            } else {
                self.get_dmg_color(self.bgp, color_index)
            };

            let buffer_index = self.ly as usize * WIDTH + x;

            self.buffer[buffer_index] = out_color;
            self.bg_color_indices[buffer_index] = color_index;
        }

        // --- Render Sprites (OBJ) ---
        if lcdc_data.obj_enable {
            self.render_sprites();
        }
    }

    /// Returns a list of sprites that are visible on the current scanline.
    ///
    /// `ly` - The current scanline (0-143)
    ///
    /// `sprite_height` - The height of the sprite (8 or 16)
    fn get_visible_sprites(&self, ly: u8, sprite_height: u8) -> Vec<RenderSprite> {
        let mut visible_sprites = Vec::new();

        // Check all 40 sprites in OAM
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

        // Sort by X position (lower X first), then by OAM index (lower index first)
        visible_sprites.sort_by(|a, b| {
            if a.screen_x != b.screen_x {
                a.screen_x.cmp(&b.screen_x) // Lower X has higher priority
            } else {
                a.oam_index.cmp(&b.oam_index) // Lower OAM index has higher priority
            }
        });

        visible_sprites
    }

    /// Renders sprites for the current scanline.
    fn render_sprites(&mut self) {
        let lcdc_data = LcdcData::from(self.lcdc);

        let sprite_height: u8 = get_sprite_height(lcdc_data.obj_size);
        let visible_sprites = self.get_visible_sprites(self.ly, sprite_height);

        // Render sprites in reverse order so lower OAM index sprites draw on top
        for sprite in visible_sprites.iter().rev() {
            let line_in_sprite = sprite.get_line_in_sprite(self.ly, sprite_height);

            // For 8x16 sprites, lower bit of tile ignored (hardware behavior)
            let (tile_num, line_offset) = if sprite_height == 16 {
                let top_half = line_in_sprite < 8;
                let tile_num = (sprite.tile_index & 0xFE) + if top_half { 0 } else { 1 };
                let line_in_tile = if top_half {
                    line_in_sprite
                } else {
                    line_in_sprite - 8
                };
                (tile_num, line_in_tile)
            } else {
                (sprite.tile_index, line_in_sprite)
            };

            // CGB: Use sprite's VRAM bank
            let sprite_vram_bank = if self.cgb_mode {
                sprite.cgb_vram_bank as usize
            } else {
                0
            };

            let tile_addr = get_sprite_tile_address(tile_num) + (line_offset as u16) * 2;
            
            // Read from correct VRAM bank
            let (byte1, byte2) = if self.cgb_mode {
                let index = vram_index(tile_addr);
                (self.vram[sprite_vram_bank][index], self.vram[sprite_vram_bank][index + 1])
            } else {
                self.read_vram_pair(tile_addr)
            };

            for px in 0..SPRITE_WIDTH {
                let bit = if sprite.x_flip { px } else { 7 - px };
                let bit1 = (byte1 >> bit) & BIT_0;
                let bit2 = (byte2 >> bit) & BIT_0;
                let color_index = (bit2 << 1) | bit1;

                // Color index 0 is transparent for OBJ
                if color_index == 0 {
                    continue;
                }

                let (pixel_x, pixel_y) = (sprite.screen_x + px as i16, self.ly as i16);
                if is_pixel_out_of_bounds(pixel_x, pixel_y) {
                    continue;
                }

                let pixel_x_usize = pixel_x as usize;
                let buffer_index = self.ly as usize * WIDTH + pixel_x_usize;

                // Get sprite color
                let out_color = if self.cgb_mode {
                    self.get_cgb_color(sprite.cgb_palette, color_index, true)
                } else {
                    let obp = if sprite.palette_index { self.obp1 } else { self.obp0 };
                    self.get_dmg_color(obp, color_index)
                };

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

    fn get_dmg_color(&self, obp: u8, color_index: u8) -> u32 {
        let color = (obp >> (color_index * 2)) & 0b11;
        // let palette = [0xFFFFFFFF, 0xFFAAAAAA, 0xFF555555, 0xFF000000];
        let palette = [0xFF9A9E3F, 0xFF496B22, 0xFF0E450B, 0xFF1B2A09];
        palette[color as usize]
    }

    /// Get CGB color from palette data (RGB555 format)
    fn get_cgb_color(&self, palette_num: u8, color_index: u8, is_obj: bool) -> u32 {
        let palette_data = if is_obj {
            &self.obj_palette_data
        } else {
            &self.bg_palette_data
        };
        
        // Each palette is 8 bytes (4 colors * 2 bytes each)
        // Each color is 2 bytes in little-endian RGB555 format
        let base_index = (palette_num * 8 + color_index * 2) as usize;
        let low = palette_data[base_index] as u16;
        let high = palette_data[base_index + 1] as u16;
        let rgb555 = (high << 8) | low;
        
        // Convert RGB555 to RGB888 with proper scaling
        // Scale 5-bit values (0-31) to 8-bit values (0-255)
        let r5 = (rgb555 & 0x1F) as u8;
        let g5 = ((rgb555 >> 5) & 0x1F) as u8;
        let b5 = ((rgb555 >> 10) & 0x1F) as u8;
        
        // Proper scaling: (value * 255) / 31, or approximately (value << 3) | (value >> 2)
        let r = (r5 << 3) | (r5 >> 2);
        let g = (g5 << 3) | (g5 >> 2);
        let b = (b5 << 3) | (b5 >> 2);
        
        // Return as 0xAARRGGBB
        0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
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
fn get_tile_address(tile_id: u8, bg_window_tile_data: bool) -> u16 {
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
