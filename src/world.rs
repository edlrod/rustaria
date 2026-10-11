use crate::position::TilePosition;
use std::collections::HashMap;

pub struct World {
    tiles: HashMap<TilePosition, u32>,
}

impl World {
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
        }
    }

    pub fn make_tile(&mut self, pos: TilePosition) -> u32 {
        let tile_id = if (pos.0 % 2 + pos.1 % 2) == 0 { 1 } else { 0 };
        self.tiles.insert(pos, tile_id);
        tile_id
    }

    pub fn get_tile(&mut self, pos: TilePosition) -> u32 {
        if let Some(&tile_id) = self.tiles.get(&pos) {
            tile_id
        } else {
            self.make_tile(pos)
        }
    }

    // pub fn set_tile(&mut self, pos: TilePosition, tile_id: u32) {
    //     self.tiles.insert(pos, tile_id);
    // }
}
