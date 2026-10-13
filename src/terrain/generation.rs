use super::{CHUNK_AREA, CHUNK_SIZE, Chunk};
use crate::tile::Tile;
use noise::{Fbm, NoiseFn, Perlin};

// How many tiles outside a chunk we scan for trees that overlap into it.
// Must cover the canopy radius.
const TREE_MARGIN: i32 = 3;

// Minimum distance between two trees, in tiles.
const TREE_SPACING: i32 = 3;

// Chance for a column to want a tree (before spacing is applied).
const TREE_CHANCE: f64 = 0.16;

// Iron only spawns this far below the surface, in tiles.
const IRON_DEPTH: i32 = 60;

// Procedural terrain shape: the noise fields and the rules that turn them into tiles.
pub(super) struct TerrainGen {
    seed: u32,

    // Noise fields, each seeded independently so they don't correlate.
    hills: Fbm<Perlin>,     // broad surface elevation
    bumps: Fbm<Perlin>,     // small surface detail + dirt depth
    tunnels: Fbm<Perlin>,   // winding cave tunnels (zero-contour bands)
    tunnels_b: Fbm<Perlin>, // slow regional on/off gate for tunnel systems
    caverns: Fbm<Perlin>,   // large blobby caverns
    ores: Fbm<Perlin>,      // ore veins
}

impl TerrainGen {
    pub(super) fn new(seed: u32) -> Self {
        let fbm = |salt: u32, octaves: usize| {
            let seed = seed.wrapping_add(salt.wrapping_mul(7919));
            let mut noise = Fbm::<Perlin>::new(seed);
            noise.octaves = octaves;
            noise
        };

        Self {
            seed,
            hills: fbm(1, 4),
            bumps: fbm(2, 4),
            tunnels: fbm(3, 4),
            tunnels_b: fbm(4, 4),
            caverns: fbm(5, 3),
            ores: fbm(6, 3),
        }
    }

    // Height of the first solid tile in column `x`.
    pub fn surface_height(&self, x: i32) -> i32 {
        let fx = x as f64;
        let hills = self.hills.get([fx * 0.003, 0.0]) * 55.0;
        let bumps = self.bumps.get([fx * 0.02, 0.0]) * 10.0;
        (hills + bumps).round() as i32
    }

    pub(super) fn generate_chunk(&self, cx: i32, cy: i32) -> Chunk {
        let x0 = cx * CHUNK_SIZE;
        let y0 = cy * CHUNK_SIZE;

        let mut tiles = [Tile::Air; CHUNK_AREA];

        for lx in 0..CHUNK_SIZE {
            let x = x0 + lx;
            let surface = self.surface_height(x);
            let dirt_bottom = surface + self.dirt_thickness(x);
            for ly in 0..CHUNK_SIZE {
                let y = y0 + ly;
                tiles[(ly * CHUNK_SIZE + lx) as usize] =
                    self.terrain_tile(x, y, surface, dirt_bottom);
            }
        }

        // Trees rooted slightly outside this chunk can still reach into it.
        for root_x in x0 - TREE_MARGIN..x0 + CHUNK_SIZE + TREE_MARGIN {
            if self.has_tree(root_x) {
                self.stamp_tree(&mut tiles, x0, y0, root_x);
            }
        }

        Chunk {
            tiles: Box::new(tiles),
            sky: Box::new([0.0f32; CHUNK_AREA]),
            block: Box::new([0.0f32; CHUNK_AREA]),
            light_dirty: true,
        }
    }

    fn dirt_thickness(&self, x: i32) -> i32 {
        let t = self.bumps.get([x as f64 * 0.01, 512.0]);
        (11.0 + t * 5.0).round() as i32
    }

    // Terrain for a single tile without tree overlap.
    fn terrain_tile(&self, x: i32, y: i32, surface: i32, dirt_bottom: i32) -> Tile {
        if y < surface {
            return Tile::Air;
        }
        if y == surface {
            return Tile::Grass;
        }

        if let Some(carved) = self.carve_tile(x, y, surface, dirt_bottom) {
            return carved;
        }

        if y < dirt_bottom {
            return Tile::Dirt;
        }

        // Ore veins in stone.
        let fx = x as f64;
        let fy = y as f64;
        if y > surface + IRON_DEPTH && self.ores.get([fx * 0.1 + 512.0, fy * 0.1]) > 0.62 {
            return Tile::IronOre;
        }
        if self.ores.get([fx * 0.09, fy * 0.09]) > 0.58 {
            return Tile::CoalOre;
        }
        Tile::Stone
    }

    fn carve_tile(&self, x: i32, y: i32, surface: i32, dirt_bottom: i32) -> Option<Tile> {
        let depth = (y - surface) as f64;
        let fx = x as f64;
        let fy = y as f64;

        // Winding tunnels.
        let width = 0.02 + 0.03 * ((depth - 1.0) / 90.0).min(1.0);
        let tunnel = self.tunnels.get([fx * 0.012, fy * 0.024]).abs() < width
            && self.tunnels_b.get([fx * 0.004, fy * 0.006]) > 0.0;

        // Large caverns that only appear deeper underground and keep growing with depth.
        let cavern = if depth > 20.0 {
            let threshold = 0.44 - 0.20 * ((depth - 20.0) / 280.0).min(1.0);
            self.caverns.get([fx * 0.012, fy * 0.02]) > threshold
        } else {
            false
        };

        if !(tunnel || cavern) {
            return None;
        }

        if y < surface + 8 {
            Some(Tile::Air)
        } else if y < dirt_bottom + 4 {
            Some(Tile::DirtWall)
        } else {
            Some(Tile::StoneWall)
        }
    }

    // Does the tree rooted in column `x` exist?
    fn has_tree(&self, x: i32) -> bool {
        self.hash01(x, 0x51ED) < TREE_CHANCE
            && (1..=TREE_SPACING).all(|d| self.hash01(x - d, 0x51ED) >= TREE_CHANCE)
    }

    // Trunk height of the tree in column `x`, in tiles.
    fn tree_height(&self, x: i32) -> i32 {
        4 + (self.hash01(x, 0xA11CE) * 7.0) as i32
    }

    // Writes the tiles of the tree rooted in column `root_x` into `tiles`.
    fn stamp_tree(&self, tiles: &mut [Tile], x0: i32, y0: i32, root_x: i32) {
        let surface = self.surface_height(root_x);
        let top = surface - self.tree_height(root_x);

        let mut put = |x: i32, y: i32, tile: Tile| {
            let (lx, ly) = (x - x0, y - y0);
            if lx < 0 || ly < 0 || lx >= CHUNK_SIZE || ly >= CHUNK_SIZE {
                return;
            }
            let idx = (ly * CHUNK_SIZE + lx) as usize;
            // Trees only grow into open air, never into solid ground.
            if tiles[idx] == Tile::Air {
                tiles[idx] = tile;
            }
        };

        // Tree trunk.
        for y in top..surface {
            put(root_x, y, Tile::Wood);
        }

        // Tree canopy.
        for dy in -3i32..=1 {
            for dx in -2i32..=2 {
                if dx.abs() + (dy + 1).abs() > 2 {
                    continue;
                }
                put(root_x + dx, top - 1 + dy, Tile::Leaves);
            }
        }
    }

    // Cheap deterministic per-integer hash in `[0, 1)`.
    fn hash01(&self, x: i32, salt: u32) -> f64 {
        let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ salt ^ self.seed;
        h ^= h >> 15;
        h = h.wrapping_mul(0x85EB_CA6B);
        h ^= h >> 13;
        h = h.wrapping_mul(0xC2B2_AE35);
        h ^= h >> 16;
        (h >> 8) as f64 / 16_777_216.0
    }
}
