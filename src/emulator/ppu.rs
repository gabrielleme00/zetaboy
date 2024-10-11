const VRAM_SIZE: usize = 0x2000;

const COLS_PER_TILE_ROW: usize = 8;
const ROWS_PER_TILE: usize = 8;
const BYTES_PER_TILE_ROW: usize = 2;
const BYTES_PER_TILE: usize = ROWS_PER_TILE * BYTES_PER_TILE_ROW;
const TILE_SET_SIZE: usize = 384;

#[derive(Copy, Clone)]
enum TilePixelValue {
    Zero,
    One,
    Two,
    Three,
}

type Tile = [[TilePixelValue; 8]; 8];
fn empty_tile() -> Tile {
    [[TilePixelValue::Zero; 8]; 8]
}

pub struct PPU {
    vram: [u8; VRAM_SIZE],
    tile_set: [Tile; TILE_SET_SIZE],
}

impl PPU {
    pub fn new() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            tile_set: [empty_tile(); TILE_SET_SIZE],
        }
    }

    /// Returns a byte from the `index` (local address).
    pub fn read_vram(&self, index: usize) -> u8 {
        self.vram[index - 0x8000]
    }

    /// Writes a byte of `value` to the `index` (local address).
    pub fn write_vram(&mut self, index: usize, value: u8) {
        let index = index - 0x8000;
        self.vram[index] = value;

        if index >= 0x1800 {
            return;
        }

        let (byte1, byte2) = self.get_tile_row(index);
        let tile_index = index / BYTES_PER_TILE;
        let row_index = (index % BYTES_PER_TILE) / 2;

        for pixel_index in 0..COLS_PER_TILE_ROW {
            let mask = 1 << (7 - pixel_index);
            let lsb = byte1 & mask;
            let msb = byte2 & mask;

            let value = match (lsb != 0, msb != 0) {
                (false, false) => TilePixelValue::Zero,
                (true, false) => TilePixelValue::One,
                (false, true) => TilePixelValue::Two,
                (true, true) => TilePixelValue::Three,
            };

            self.tile_set[tile_index][row_index][pixel_index] = value;
        }
    }

    fn get_tile_row(&self, index: usize) -> (u8, u8) {
        let normalized_index = index & 0xFFFE;
        let byte1 = self.read_vram(normalized_index);
        let byte2 = self.read_vram(normalized_index + 1);
        (byte1, byte2)
    }
}
