use crate::utils::bits::*;

pub struct LcdcData {
    pub bg_enable: bool,
    pub obj_enable: bool,
    pub obj_size: bool,
    pub bg_tile_map: bool,
    pub bg_window_tile_data: bool,
    pub window_enable: bool,
    pub window_tile_map: bool,
    pub lcd_enable: bool,
}

impl From<u8> for LcdcData {
    fn from(byte: u8) -> Self {
        Self {
            bg_enable: (byte & BIT_0) != 0,
            obj_enable: (byte & BIT_1) != 0,
            obj_size: (byte & BIT_2) != 0,
            bg_tile_map: (byte & BIT_3) != 0,
            bg_window_tile_data: (byte & BIT_4) != 0,
            window_enable: (byte & BIT_5) != 0,
            window_tile_map: (byte & BIT_6) != 0,
            lcd_enable: (byte & BIT_7) != 0,
        }
    }
}
