use crate::camera::Camera;
use crate::ecs::components::{ColorRenderer, Cursor, Transform};
use crate::terrain::{CHUNK_SIZE, Terrain};
use macroquad::prelude::*;

/// Brightness curve. Multiplicative decay already produces a natural falloff.
const LIGHT_GAMMA: f32 = 1.0;

/// Nothing renders darker than this.
const AMBIENT: f32 = 0.12;

pub fn draw_terrain(terrain: &mut Terrain, camera: &Camera) {
    let palette: Vec<Option<Color>> = terrain
        .tile_registry()
        .infos()
        .iter()
        .map(|info| info.color.map(to_color))
        .collect();

    for (cx, cy) in camera.visible_chunks() {
        let chunk = terrain.get_chunk((cx, cy));
        let sky = chunk.sky();
        let block = chunk.block();

        for (i, tile) in chunk.tiles().iter().enumerate() {
            let x = (cx * CHUNK_SIZE + ((i as i32) % CHUNK_SIZE)) as f32;
            let y = (cy * CHUNK_SIZE + ((i as i32) / CHUNK_SIZE)) as f32;

            // Draw the tile color.
            if let Some(color) = palette.get(*tile as usize).copied().flatten() {
                draw_rectangle(x, y, 1.0, 1.0, color);
            }

            // Draw darkness.
            let level = sky[i].max(block[i]);
            let b = brightness(level);
            if b < 1.0 {
                draw_rectangle(x, y, 1.0, 1.0, Color::new(0.0, 0.0, 0.0, 1.0 - b));
            }
        }
    }
}

/// One light interpretation for tiles and entities.
fn brightness(level: f32) -> f32 {
    level.max(AMBIENT).powf(LIGHT_GAMMA)
}

fn shaded(color: Color, brightness: f32) -> Color {
    Color::new(
        color.r * brightness,
        color.g * brightness,
        color.b * brightness,
        color.a,
    )
}

fn to_color(rgb: [u8; 3]) -> Color {
    Color::new(
        rgb[0] as f32 / 255.0,
        rgb[1] as f32 / 255.0,
        rgb[2] as f32 / 255.0,
        1.0,
    )
}

pub fn draw_entities(entities: &mut hecs::World, terrain: &mut Terrain) {
    // World objects: tinted by the light at their center.
    for (transform, renderer) in &mut entities
        .query::<(&Transform, &ColorRenderer)>()
        .without::<&Cursor>()
    {
        let (sky, block) = terrain.get_light((
            (transform.x + transform.width * 0.5).floor() as i32,
            (transform.y + transform.height * 0.5).floor() as i32,
        ));
        let b = brightness(sky.max(block));
        draw_rectangle(
            transform.x,
            transform.y,
            transform.width,
            transform.height,
            shaded(renderer.color, b),
        );
    }

    for (transform, renderer) in &mut entities
        .query::<(&Transform, &ColorRenderer)>()
        .with::<&Cursor>()
    {
        draw_rectangle(
            transform.x,
            transform.y,
            transform.width,
            transform.height,
            renderer.color,
        );
    }
}
