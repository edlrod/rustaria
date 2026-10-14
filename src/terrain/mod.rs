mod generation;
mod lighting;

use crate::position::{ChunkPosition, TilePosition};
use crate::tile::{Tile, TileRegistry};
use std::collections::HashMap;

use generation::TerrainGen;
use lighting::Lighting;

/// Side length of a chunk, in tiles.
pub const CHUNK_SIZE: i32 = 32;
pub(crate) const CHUNK_AREA: usize = (CHUNK_SIZE * CHUNK_SIZE) as usize;

pub struct Chunk {
    tiles: Box<[Tile; CHUNK_AREA]>,
    sky: Box<[f32; CHUNK_AREA]>,
    block: Box<[f32; CHUNK_AREA]>,
    /// Set when tiles changed and the light arrays are stale.
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

/// Decompose a tile coordinate into its chunk coordinate plus the local
/// coordinate inside that chunk. `div_euclid` keeps this correct across the
/// negative-coordinate chunk boundary.
fn chunk_and_local(pos: TilePosition) -> (ChunkPosition, TilePosition) {
    let (x, y) = pos;
    let (cx, cy) = (x.div_euclid(CHUNK_SIZE), y.div_euclid(CHUNK_SIZE));
    let local = (x - cx * CHUNK_SIZE, y - cy * CHUNK_SIZE);
    ((cx, cy), local)
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

    /// The tile definitions this terrain was created with.
    pub fn tile_registry(&self) -> &TileRegistry {
        &self.tile_registry
    }

    fn ensure_generated(&mut self, chunk_pos: ChunkPosition) {
        if !self.chunks.contains_key(&chunk_pos) {
            let chunk = self.generator.generate_chunk(chunk_pos);
            self.chunks.insert(chunk_pos, chunk);
        }
    }

    pub fn get_chunk(&mut self, chunk_pos: ChunkPosition) -> &Chunk {
        self.ensure_generated(chunk_pos);
        let dirty = self.chunks.get(&chunk_pos).unwrap().light_dirty;
        if dirty {
            self.relight_chunk(chunk_pos);
        }
        self.chunks.get(&chunk_pos).unwrap()
    }

    fn get_chunk_mut(&mut self, chunk_pos: ChunkPosition) -> &mut Chunk {
        self.ensure_generated(chunk_pos);
        self.chunks.get_mut(&chunk_pos).unwrap()
    }

    pub fn get_tile(&mut self, pos: TilePosition) -> Tile {
        let (chunk_pos, (lx, ly)) = chunk_and_local(pos);
        self.ensure_generated(chunk_pos);
        let chunk = self.chunks.get(&chunk_pos).unwrap();
        chunk.tiles()[(ly * CHUNK_SIZE + lx) as usize]
    }

    pub fn set_tile(&mut self, pos: TilePosition, tile: Tile) {
        let (chunk_pos, (lx, ly)) = chunk_and_local(pos);
        let chunk = self.get_chunk_mut(chunk_pos);
        chunk.tiles[(ly * CHUNK_SIZE + lx) as usize] = tile;

        // Invalidate this column's sky entry; if the floor moved, light
        // changes for the whole column below, not just nearby.
        let floor_moved = self.invalidate_sky_column(pos.0);

        // Light travels at most LIGHT_RADIUS (32) <= CHUNK_SIZE (32).
        let (cx, cy) = chunk_pos;
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
        let (chunk_pos, (lx, ly)) = chunk_and_local(pos);
        let chunk = self.get_chunk(chunk_pos);
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