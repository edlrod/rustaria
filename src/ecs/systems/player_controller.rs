use crate::ecs::components::{Player, Transform, Velocity};
use crate::world::World;
use macroquad::prelude::*;

const SPEED: f32 = 8.0;
const JUMP_SPEED: f32 = -16.0;

pub fn player_controller(entities: &mut hecs::World, tiles: &mut World) {
    let mut direction = 0.0;
    if is_key_down(KeyCode::A) {
        direction -= 1.0;
    }
    if is_key_down(KeyCode::D) {
        direction += 1.0;
    }

    for (transform, velocity) in entities
        .query_mut::<(&Transform, &mut Velocity)>()
        .with::<&Player>()
    {
        velocity.x = direction * SPEED;

        if (is_key_down(KeyCode::Space) || is_key_down(KeyCode::W)) && grounded(tiles, transform) {
            velocity.y = JUMP_SPEED;
        }
    }
}

fn grounded(tiles: &mut World, transform: &Transform) -> bool {
    const INSET: f32 = 0.05; // ignore walls touching the player's sides
    const PROBE: f32 = 0.1; // how far below the feet to look
    tiles.overlaps_solid(
        transform.x + INSET,
        transform.y + transform.height,
        transform.width - 2.0 * INSET,
        PROBE,
    )
}
