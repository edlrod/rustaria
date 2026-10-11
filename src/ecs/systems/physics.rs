use crate::ecs::components::{Transform, Velocity};
use crate::world::World;

const GRAVITY: f32 = 40.0; // tiles per second squared
const MAX_FALL: f32 = 40.0; // terminal velocity, tiles per second
const EDGE_EPS: f32 = 1e-4;

pub fn physics(entities: &mut hecs::World, tiles: &mut World, dt: f32) {
    for (transform, velocity) in entities.query_mut::<(&mut Transform, &mut Velocity)>() {
        velocity.y = (velocity.y + GRAVITY * dt).min(MAX_FALL);

        transform.x += velocity.x * dt;
        resolve_x(transform, velocity, tiles);

        transform.y += velocity.y * dt;
        resolve_y(transform, velocity, tiles);
    }
}

fn resolve_x(transform: &mut Transform, velocity: &mut Velocity, tiles: &mut World) {
    if !tiles.overlaps_solid(transform.x, transform.y, transform.width, transform.height) {
        return;
    }
    if velocity.x > 0.0 {
        transform.x = (transform.x + transform.width - EDGE_EPS).floor() - transform.width;
    } else if velocity.x < 0.0 {
        transform.x = transform.x.floor() + 1.0;
    }
    velocity.x = 0.0;
}

fn resolve_y(transform: &mut Transform, velocity: &mut Velocity, tiles: &mut World) {
    if !tiles.overlaps_solid(transform.x, transform.y, transform.width, transform.height) {
        return;
    }
    if velocity.y > 0.0 {
        transform.y = (transform.y + transform.height - EDGE_EPS).floor() - transform.height;
    } else if velocity.y < 0.0 {
        transform.y = transform.y.floor() + 1.0;
    }
    velocity.y = 0.0;
}
