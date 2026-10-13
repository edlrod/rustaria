use crate::tile::{Tile, TileRegistry};
use std::collections::VecDeque;

// Fraction of light that survives one tile of air.
pub const AIR_DECAY: f32 = 0.91;

// Fraction that survives one tile of solid ground.
pub const SOLID_DECAY: f32 = 0.66;

// Light dimmer than this is considered dead and never propagates.
pub const LIGHT_EPSILON: f32 = 0.05;

// Compute light for a rectangle of tiles ("window").
pub fn compute_light(
    tiles: &[Tile],
    column_open: &[bool],
    infos: &TileRegistry,
    w: i32,
    h: i32,
) -> (Vec<f32>, Vec<f32>) {
    let n = (w * h) as usize;
    assert_eq!(tiles.len(), n, "window must be exactly w * h tiles");
    assert_eq!(
        column_open.len(),
        w as usize,
        "one open-sky flag per column"
    );

    let mut sky = vec![0.0f32; n];
    let mut block = vec![0.0f32; n];

    // Decay tables: the fraction of light that survives ENTERING each tile per channel.
    let mut sky_decay = vec![AIR_DECAY; n];
    let mut block_decay = vec![AIR_DECAY; n];
    for i in 0..n {
        let info = infos.info(tiles[i]);
        if info.opaque {
            sky_decay[i] = SOLID_DECAY;
            block_decay[i] = SOLID_DECAY;
        } else if info.blocks_sky {
            sky_decay[i] = 0.0;
        }
    }

    // Sky seeds: the sun is a vertical column scan, from the open sky down to the first tile that blocks it.
    for x in 0..w {
        if !column_open[x as usize] {
            continue;
        }
        for y in 0..h {
            let i = (y * w + x) as usize;
            if sky_decay[i] < AIR_DECAY {
                break; // solids dim the sun's passage, walls end it
            }
            sky[i] = 1.0;
        }
    }

    // Block seeds: every emitter glows, even opaque ones.
    for i in 0..n {
        let emission = infos.info(tiles[i]).light;
        if emission > 0.0 {
            block[i] = emission;
        }
    }

    propagate(&mut sky, &sky_decay, w, h);
    propagate(&mut block, &block_decay, w, h);

    (sky, block)
}

fn propagate(values: &mut [f32], decays: &[f32], w: i32, h: i32) {
    let mut queue: VecDeque<usize> = (0..values.len())
        .filter(|&i| values[i] > LIGHT_EPSILON)
        .collect();

    while let Some(i) = queue.pop_front() {
        let level = values[i];

        // Too dim to matter. Multiplicative decay fades forever, so this threshold is what bounds the light's reach.
        if level <= LIGHT_EPSILON {
            continue;
        }

        let x = (i as i32) % w;
        let y = (i as i32) / w;

        for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if nx < 0 || nx >= w || ny < 0 || ny >= h {
                continue;
            }
            let n = (ny * w + nx) as usize;
            let next = level * decays[n];
            if next > values[n] {
                values[n] = next;
                queue.push_back(n);
            }
        }
    }
}
