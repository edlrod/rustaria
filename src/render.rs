use crate::camera::Camera;
use crate::ecs::components::{ColorRenderer, Transform};
use crate::world::{CHUNK_SIZE, Tile, World};
use macroquad::prelude::*;

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

pub fn draw_world(world: &mut World, camera: &Camera) {
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
}

pub fn draw_entities(entities: &mut hecs::World) {
    for (transform, renderer) in &mut entities.query::<(&Transform, &ColorRenderer)>() {
        draw_rectangle(
            transform.x,
            transform.y,
            transform.width,
            transform.height,
            renderer.color,
        );
    }
}
