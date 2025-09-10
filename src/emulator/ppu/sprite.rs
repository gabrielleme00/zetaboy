use crate::utils::bits::*;

pub const SPRITE_SIZE_BYTES: u8 = 4;
pub const SPRITE_OFFSET_Y: u8 = 16;
pub const SPRITE_OFFSET_X: u8 = 8;
pub const SPRITE_WIDTH: u8 = 8;

/// Raw sprite data as stored in OAM
#[derive(Debug, Clone, Copy)]
pub struct OAMSprite {
    pub y: u8,
    pub x: u8,
    pub tile_index: u8,
    pub attributes: u8,
    pub oam_index: usize,
}

impl OAMSprite {
    /// Creates a new OAMSprite from OAM data by an index (0-39).
    pub fn at_oam(oam: &[u8], oam_index: usize) -> Self {
        let base = oam_index * SPRITE_SIZE_BYTES as usize;
        Self {
            y: oam[base],
            x: oam[base + 1],
            tile_index: oam[base + 2],
            attributes: oam[base + 3],
            oam_index,
        }
    }
}

/// Sprite prepared for rendering with parsed attributes
#[derive(Debug, Clone, Copy)]
pub struct RenderSprite {
    pub screen_x: i16,
    pub screen_y: i16,
    pub tile_index: u8,
    pub palette_index: bool, // false = OBP0, true = OBP1
    pub x_flip: bool,
    pub y_flip: bool,
    pub bg_priority: bool, // true = behind BG colors 1-3
    pub oam_index: usize,
}

impl From<OAMSprite> for RenderSprite {
    fn from(oam_sprite: OAMSprite) -> Self {
        let attr = oam_sprite.attributes;
        Self {
            screen_x: oam_sprite.x as i16 - SPRITE_OFFSET_X as i16,
            screen_y: oam_sprite.y as i16 - SPRITE_OFFSET_Y as i16,
            tile_index: oam_sprite.tile_index,
            palette_index: (attr & BIT_4) != 0,
            x_flip: (attr & BIT_5) != 0,
            y_flip: (attr & BIT_6) != 0,
            bg_priority: (attr & BIT_7) != 0,
            oam_index: oam_sprite.oam_index,
        }
    }
}

impl RenderSprite {
    /// Checks if the sprite is visible on the given scanline.
    pub fn is_visible_on_line(&self, ly: u8, sprite_height: u8) -> bool {
        let scanline = ly as i16;
        scanline >= self.screen_y && scanline < self.screen_y + sprite_height as i16
    }

    /// Returns the line within the sprite for the given scanline.
    pub fn get_line_in_sprite(&self, ly: u8, sprite_height: u8) -> u8 {
        (if self.y_flip {
            sprite_height as i16 - 1 - (ly as i16 - self.screen_y)
        } else {
            ly as i16 - self.screen_y
        }) as u8
    }

    /// Returns the address of the sprite's palette.
    pub fn get_obp_address(&self) -> u16 {
        if self.palette_index {
            0xFF49 // OBP1
        } else {
            0xFF48 // OBP0
        }
    }
}
