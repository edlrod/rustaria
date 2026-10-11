use crate::ecs::components::{Player, Transform};
use macroquad::prelude::*;

pub fn player_controller(world: &mut hecs::World, dt: f32) {
    const SPEED: f32 = 8.0;

    let mut direction = Vec2::ZERO;
    if is_key_down(KeyCode::A) {
        direction.x -= 1.0;
    }
    if is_key_down(KeyCode::D) {
        direction.x += 1.0;
    }
    if is_key_down(KeyCode::W) {
        direction.y -= 1.0;
    }
    if is_key_down(KeyCode::S) {
        direction.y += 1.0;
    }
    if direction.length_squared() > 1.0 {
        direction = direction.normalize(); // so diagonals aren't faster
    }

    for transform in world.query_mut::<&mut Transform>().with::<&Player>() {
        transform.x += direction.x * SPEED * dt;
        transform.y += direction.y * SPEED * dt;
    }
}
