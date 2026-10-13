use super::{Terrain, CHUNK_SIZE};
use crate::light;
use crate::tile::Tile;
use std::collections::HashMap;

/// How far light can still matter in air: 0.91^32 < LIGHT_EPSILON (0.05), so light is dead within 32 tiles of its source.
pub(super) const LIGHT_RADIUS: i32 = 32;

/// A chunk plus one light radius on each side. Any source that could light the chunk's interior brighter than LIGHT_EPSILON is inside this window.
const WINDOW_SIZE: i32 = CHUNK_SIZE + 2 * LIGHT_RADIUS;

/// The relight snapshot below copies whole chunks; that only works while the window is exactly a 3x3 grid of chunks.
const _: () = assert!(
    WINDOW_SIZE == 3 * CHUNK_SIZE,
    "window must be a 3x3 chunk grid"
);

/// Generation puts nothing above this height (terrain tops out near y = -76), so sky scans start here and only meet air.
const SKY_CEILING: i32 = -128;

/// Safety bound for the sky scan; generation guarantees a sun-blocking tile long before this depth.
const SKY_SCAN_LIMIT: i32 = 1024;

/// Light driver state: the per-column sky cache used while (re)lighting chunks.
pub(super) struct Lighting {
    /// Per-column sky cache: <x, y> of the first tile that blocks sunlight.
    sky_floors: HashMap<i32, i32>,
}

impl Lighting {
    pub(super) fn new() -> Self {
        // Invariant: the window margin must cover light's full reach, or chunk seams appear. Checked, not assumed.
        debug_assert!(
            light::AIR_DECAY.powi(LIGHT_RADIUS) < light::LIGHT_EPSILON,
            "LIGHT_RADIUS no longer covers light's reach through air"
        );

        Self {
            sky_floors: HashMap::new(),
        }
    }
}

// Lighting methods on `Terrain`: they read/write the shared chunk storage,
// so they live next to it instead of on `Lighting` (which only holds the cache).
impl Terrain {
    /// Drop column `x`'s cached sky floor and rescan. Did the floor move?
    pub(super) fn invalidate_sky_column(&mut self, x: i32) -> bool {
        let old_floor = self.lighting.sky_floors.get(&x).copied();
        self.lighting.sky_floors.remove(&x);
        match old_floor {
            Some(old) => self.sky_floor(x) != old, // rescans and re-caches
            // Never scanned: no chunk's light has ever depended on it.
            None => false,
        }
    }

    /// Y of the first tile in column `x` that blocks sunlight.
    fn sky_floor(&mut self, x: i32) -> i32 {
        if let Some(&floor) = self.lighting.sky_floors.get(&x) {
            return floor;
        }
        let mut y = SKY_CEILING;
        while y < SKY_SCAN_LIMIT {
            let tile = self.get_tile((x, y)); // &mut self ends when this returns
            let info = self.tile_registry().info(tile); // fresh shared borrow
            if info.opaque || info.blocks_sky {
                break;
            }
            y += 1;
        }
        let floor = if y >= SKY_SCAN_LIMIT { i32::MAX } else { y };
        self.lighting.sky_floors.insert(x, floor);
        floor
    }

    /// Recompute the light arrays of chunk `(cx, cy)` from a 3x3-chunk window.
    pub(super) fn relight_chunk(&mut self, cx: i32, cy: i32) {
        let x0 = cx * CHUNK_SIZE - LIGHT_RADIUS;
        let y0 = cy * CHUNK_SIZE - LIGHT_RADIUS;

        // Snapshot the window's tiles.
        let mut window = vec![Tile::Air; (WINDOW_SIZE * WINDOW_SIZE) as usize];
        for gy in -1i32..=1 {
            for gx in -1i32..=1 {
                self.ensure_generated(cx + gx, cy + gy);
                let chunk = self.chunks.get(&(cx + gx, cy + gy)).unwrap();
                let wx = ((gx + 1) * CHUNK_SIZE) as usize;
                let wy = ((gy + 1) * CHUNK_SIZE) as usize;
                for row in 0..CHUNK_SIZE as usize {
                    let src =
                        &chunk.tiles()[row * CHUNK_SIZE as usize..(row + 1) * CHUNK_SIZE as usize];
                    let dst = (wy + row) * WINDOW_SIZE as usize + wx;
                    window[dst..dst + CHUNK_SIZE as usize].copy_from_slice(src);
                }
            }
        }

        // Does each window column start in open sky?
        let mut column_open = vec![false; WINDOW_SIZE as usize];
        for lx in 0..WINDOW_SIZE {
            column_open[lx as usize] = self.sky_floor(x0 + lx) > y0;
        }

        let (sky, block) =
            light::compute_light(&window, &column_open, &self.tile_registry, WINDOW_SIZE, WINDOW_SIZE);

        // Copy the window's center back into the chunk.
        let chunk = self
            .chunks
            .get_mut(&(cx, cy))
            .expect("relight only runs on generated chunks");
        for ly in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                let from = ((ly + LIGHT_RADIUS) * WINDOW_SIZE + lx + LIGHT_RADIUS) as usize;
                let to = (ly * CHUNK_SIZE + lx) as usize;
                chunk.sky[to] = sky[from];
                chunk.block[to] = block[from];
            }
        }
        chunk.light_dirty = false;
    }
}