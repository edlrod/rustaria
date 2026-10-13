mod generation;
mod lighting;

use crate::position::{ChunkPosition, TilePosition};
use crate::tile::{Tile, TileRegistry};
use std::collections::HashMap;

use generation::TerrainGen;
use lighting::Lighting;

// Side length of a chunk, in tiles.
pub const CHUNK_SIZE: i32 = 32;
pub(crate) const CHUNK_AREA: usize = (CHUNK_SIZE * CHUNK_SIZE) as usize;

pub struct Chunk {
    tiles: Box<[Tile; CHUNK_AREA]>,
    sky: Box<[f32; CHUNK_AREA]>,
    block: Box<[f32; CHUNK_AREA]>,
    // Set when tiles changed and the light arrays are stale.
    light_dirty: bool,
}

impl Chunk {
    pub fn tiles(&self) -> &[Tile; CHUNK_AREA] {
        &self.tiles
    }
    pub fn sky(&self) -> &[f32; CHUNK_AREA] {
        &self.sky
    }
    pub fn block(&self) -> &[f32; CHUNK_AREA] {
        &self.block
    }
}

/// The tile world: chunk storage plus the tile-edit and collision API.
///
/// Terrain *shape* comes from [`generation::TerrainGen`], light values from
/// [`lighting::Lighting`]; this type owns the shared chunk storage they both
/// work on.
pub struct Terrain {
    chunks: HashMap<ChunkPosition, Chunk>,
    tile_registry: TileRegistry,
    generator: TerrainGen,
    lighting: Lighting,
}

impl Terrain {
    pub fn new(seed: u32, tile_registry: TileRegistry) -> Self {
        Self {
            chunks: HashMap::new(),
            tile_registry,
            generator: TerrainGen::new(seed),
            lighting: Lighting::new(),
        }
    }

    // The tile definitions this terrain was created with.
    pub fn tile_registry(&self) -> &TileRegistry {
        &self.tile_registry
    }

    fn ensure_generated(&mut self, cx: i32, cy: i32) {
        if !self.chunks.contains_key(&(cx, cy)) {
            let chunk = self.generator.generate_chunk(cx, cy);
            self.chunks.insert((cx, cy), chunk);
        }
    }

    pub fn get_chunk(&mut self, cx: i32, cy: i32) -> &Chunk {
        self.ensure_generated(cx, cy);
        let dirty = self.chunks.get(&(cx, cy)).unwrap().light_dirty;
        if dirty {
            self.relight_chunk(cx, cy);
        }
        self.chunks.get(&(cx, cy)).unwrap()
    }

    fn get_chunk_mut(&mut self, cx: i32, cy: i32) -> &mut Chunk {
        self.ensure_generated(cx, cy);
        self.chunks.get_mut(&(cx, cy)).unwrap()
    }

    pub fn get_tile(&mut self, pos: TilePosition) -> Tile {
        let (x, y) = pos;
        let (cx, cy) = (x.div_euclid(CHUNK_SIZE), y.div_euclid(CHUNK_SIZE));
        self.ensure_generated(cx, cy);
        let chunk = self.chunks.get(&(cx, cy)).unwrap();
        let (lx, ly) = (x - cx * CHUNK_SIZE, y - cy * CHUNK_SIZE);
        chunk.tiles()[(ly * CHUNK_SIZE + lx) as usize]
    }

    pub fn set_tile(&mut self, pos: TilePosition, tile: Tile) {
        let (x, y) = pos;
        let (cx, cy) = (x.div_euclid(CHUNK_SIZE), y.div_euclid(CHUNK_SIZE));
        let chunk = self.get_chunk_mut(cx, cy);
        let (lx, ly) = (x - cx * CHUNK_SIZE, y - cy * CHUNK_SIZE);
        chunk.tiles[(ly * CHUNK_SIZE + lx) as usize] = tile;

        // Invalidate this column's sky entry; if the floor moved, light
        // changes for the whole column below, not just nearby.
        let floor_moved = self.invalidate_sky_column(x);

        // Light travels at most LIGHT_RADIUS (32) <= CHUNK_SIZE (32).
        for dcx in -1..=1 {
            for dcy in -1..=1 {
                if let Some(chunk) = self.chunks.get_mut(&(cx + dcx, cy + dcy)) {
                    chunk.light_dirty = true;
                }
            }
        }

        // All three columns must re-light below the edit.
        if floor_moved {
            for (&(kx, _), chunk) in self.chunks.iter_mut() {
                if (kx - cx).abs() <= 1 {
                    chunk.light_dirty = true;
                }
            }
        }
    }

    pub fn get_light(&mut self, pos: TilePosition) -> (f32, f32) {
        let (x, y) = pos;
        let (cx, cy) = (x.div_euclid(CHUNK_SIZE), y.div_euclid(CHUNK_SIZE));
        let chunk = self.get_chunk(cx, cy);
        let (lx, ly) = (x - cx * CHUNK_SIZE, y - cy * CHUNK_SIZE);
        let i = (ly * CHUNK_SIZE + lx) as usize;
        (chunk.sky()[i], chunk.block()[i])
    }

    pub fn overlaps_solid(&mut self, x: f32, y: f32, w: f32, h: f32) -> bool {
        const EPS: f32 = 1e-4;
        let x0 = x.floor() as i32;
        let x1 = (x + w - EPS).floor() as i32;
        let y0 = y.floor() as i32;
        let y1 = (y + h - EPS).floor() as i32;

        for ty in y0..=y1 {
            for tx in x0..=x1 {
                let tile = self.get_tile((tx, ty));
                if self.tile_registry.is_solid(tile) {
                    return true;
                }
            }
        }
        false
    }
}