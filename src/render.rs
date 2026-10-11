use crate::camera::Camera;
use crate::ecs::components::{ColorRenderer, Transform};
use crate::world::{CHUNK_SIZE, World};
use macroquad::prelude::*;

pub fn draw_world(world: &mut World, camera: &Camera) {
    // Copy the palette out first so the immutable tile-set borrow doesn't
    // conflict with the mutable chunk access below.
    let palette: Vec<Option<Color>> = world
        .tiles()
        .infos()
        .iter()
        .map(|info| info.color.map(to_color))
        .collect();

    for (cx, cy) in camera.visible_chunks() {
        let chunk = world.get_chunk(cx, cy);
        for (i, tile) in chunk.tiles().iter().enumerate() {
            let Some(color) = palette.get(*tile as usize).copied().flatten() else {
                continue;
            };
            let x = cx * CHUNK_SIZE + ((i as i32) % CHUNK_SIZE);
            let y = cy * CHUNK_SIZE + ((i as i32) / CHUNK_SIZE);
            draw_rectangle(x as f32, y as f32, 1.0, 1.0, color);
        }
    }
}

fn to_color(rgb: [u8; 3]) -> Color {
    Color::new(
        rgb[0] as f32 / 255.0,
        rgb[1] as f32 / 255.0,
        rgb[2] as f32 / 255.0,
        1.0,
    )
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
