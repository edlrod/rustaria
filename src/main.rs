mod camera;
mod position;
mod world;

use camera::Camera;
use macroquad::prelude::*;
use world::{CHUNK_SIZE, Tile, World};

use crate::position::Position;

// Flat colors for each tile type. `None` means "nothing to draw" (sky).
fn tile_color(tile: Tile) -> Option<Color> {
    let hex = match tile {
        Tile::Air => return None,
        Tile::Grass => 0x4ca64c,
        Tile::Dirt => 0x96613f,
        Tile::Stone => 0x7d8087,
        Tile::Wood => 0x8a5a36,
        Tile::Leaves => 0x2f7a3d,
        Tile::DirtWall => 0x4d3b2a,
        Tile::StoneWall => 0x39404c,
        Tile::CoalOre => 0x4a4e57,
        Tile::IronOre => 0xa87a58,
    };
    Some(Color::from_hex(hex))
}

fn seed_from_time() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() ^ (d.as_secs() as u32))
        .unwrap_or(42)
}

#[macroquad::main("Rusteria")]
async fn main() {
    let seed = seed_from_time();
    println!("world seed: {seed}");
    let mut world = World::new(seed);

    let mut target: Position = (0.0, world.surface_height(0) as f32 - 8.0);
    let mut camera = Camera::new(16.0);

    loop {
        clear_background(SKYBLUE);
        camera.set();

        let dt = get_frame_time();

        for (cx, cy) in camera.visible_chunks() {
            let chunk = world.get_chunk(cx, cy);

            for (i, tile) in chunk.tiles().iter().enumerate() {
                let Some(color) = tile_color(*tile) else {
                    continue;
                };

                let x = cx * CHUNK_SIZE + ((i as i32) % CHUNK_SIZE);
                let y = cy * CHUNK_SIZE + ((i as i32) / CHUNK_SIZE);

                draw_rectangle(x as f32, y as f32, 1.0, 1.0, color);
            }
        }

        let camera_speed = 150.0;

        if is_key_down(KeyCode::A) {
            target.0 -= (camera_speed / camera.ppu) * dt;
        }
        if is_key_down(KeyCode::D) {
            target.0 += (camera_speed / camera.ppu) * dt;
        }
        if is_key_down(KeyCode::W) {
            target.1 -= (camera_speed / camera.ppu) * dt;
        }
        if is_key_down(KeyCode::S) {
            target.1 += (camera_speed / camera.ppu) * dt;
        }
        if is_key_down(KeyCode::Minus) {
            camera.ppu = (camera.ppu * (1.0 - 2.0 * dt)).max(4.0);
        }
        if is_key_down(KeyCode::Equal) {
            camera.ppu *= 1.0 + 2.0 * dt;
        }

        camera.follow(target);

        next_frame().await
    }
}
