use crate::tile::{Tile, TileSet};
use std::collections::VecDeque;

pub const LIGHT_MAX: u8 = 15;

// Compute light for a rectangle of tiles ("window").
//
// * `tiles`        – `w * h` window contents, row-major (same layout as `Chunk`).
// * `column_open`  – one bool per column: does this column see open sky above the window's top edge?
// * returns        – `(sky, block)` light arrays, same row-major layout.
pub fn compute_light(
    tiles: &[Tile],
    column_open: &[bool],
    infos: &TileSet,
    w: i32,
    h: i32,
) -> (Vec<u8>, Vec<u8>) {
    let n = (w * h) as usize;
    // Contracts are cheap to check and turn misuse into an obvious crash instead of a silent wrong answer.
    assert_eq!(tiles.len(), n, "window must be exactly w * h tiles");
    assert_eq!(
        column_open.len(),
        w as usize,
        "one open-sky flag per column"
    );

    let mut sky = vec![0u8; n]; // vec![value; count] — n copies of 0
    let mut block = vec![0u8; n];

    // Blocker tables first: the sky seeding below needs to know what stops sunlight.
    let mut sky_blocks = vec![false; n];
    let mut block_blocks = vec![false; n];
    for i in 0..n {
        let info = infos.info(tiles[i]);
        sky_blocks[i] = info.opaque || info.blocks_sky; // walls block sun
        block_blocks[i] = info.opaque;
    }

    // Sky seeds: walk each open column from the top, lighting tiles until something blocks the sun.
    for x in 0..w {
        if !column_open[x as usize] {
            continue;
        }
        for y in 0..h {
            let i = (y * w + x) as usize;
            if sky_blocks[i] {
                break;
            }
            sky[i] = LIGHT_MAX;
        }
    }

    // Block seeds: every emitter glows, even opaque ones.
    for i in 0..n {
        let emission = infos.info(tiles[i]).light;
        if emission > 0 {
            block[i] = emission;
        }
    }

    propagate(&mut sky, &sky_blocks, w, h);
    propagate(&mut block, &block_blocks, w, h);

    (sky, block)
}

// Spread light outward from every lit tile, losing one level per step, stopped by `blockers`.
// Label-correcting BFS: values only ever increase, every push strictly raised one cell, and cells cap at [`LIGHT_MAX`].
fn propagate(values: &mut [u8], blockers: &[bool], w: i32, h: i32) {
    // Seed the queue with every lit tile.
    let mut queue: VecDeque<usize> = (0..values.len()).filter(|&i| values[i] > 0).collect();

    while let Some(i) = queue.pop_front() {
        let level = values[i];

        if level <= 1 {
            continue;
        }
        let next = level - 1;

        // Reverse the row-major index back into coordinates.
        let x = (i as i32) % w;
        let y = (i as i32) / w;

        // Iterate the four neighbors as an array of tuples, destructured directly by the loop pattern.
        for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if nx < 0 || nx >= w || ny < 0 || ny >= h {
                continue;
            }
            let n = (ny * w + nx) as usize;
            if blockers[n] {
                continue; // light cannot enter this tile
            }
            // Cells can be pushed more than once, but when popped we read the CURRENT value, so stale entries are redundant.
            if next > values[n] {
                values[n] = next;
                queue.push_back(n);
            }
        }
    }
}
