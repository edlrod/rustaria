use crate::ecs::components::{Transform, Velocity};
use crate::terrain::Terrain;

const GRAVITY: f32 = 40.0; // tiles per second squared
const MAX_FALL: f32 = 40.0; // terminal velocity, tiles per second
const EDGE_EPS: f32 = 1e-4;

/// Largest distance an entity may move between collision checks.
const MAX_STEP: f32 = 0.5;

pub fn physics(entities: &mut hecs::World, terrain: &mut Terrain, dt: f32) {
    for (transform, velocity) in entities.query_mut::<(&mut Transform, &mut Velocity)>() {
        // Split movement into sub-steps so a fast fall can't skip over tiles.
        let steps = substeps(velocity, dt);
        let step_dt = dt / steps as f32;

        for _ in 0..steps {
            velocity.y = (velocity.y + GRAVITY * step_dt).min(MAX_FALL);

            transform.x += velocity.x * step_dt;
            resolve_x(transform, velocity, terrain);

            transform.y += velocity.y * step_dt;
            resolve_y(transform, velocity, terrain);
        }
    }
}

/// Number of sub-steps needed to keep movement below `MAX_STEP` per check.
fn substeps(velocity: &Velocity, dt: f32) -> i32 {
    // Gravity can reach MAX_FALL within the frame, so treat it as the ceiling.
    let speed = velocity.x.abs().max(velocity.y.abs()).max(MAX_FALL);
    (speed * dt / MAX_STEP).ceil().max(1.0) as i32
}

fn resolve_x(transform: &mut Transform, velocity: &mut Velocity, terrain: &mut Terrain) {
    if !terrain.overlaps_solid(transform.x, transform.y, transform.width, transform.height) {
        return;
    }
    if velocity.x > 0.0 {
        transform.x = (transform.x + transform.width - EDGE_EPS).floor() - transform.width;
    } else if velocity.x < 0.0 {
        transform.x = transform.x.floor() + 1.0;
    }
    velocity.x = 0.0;
}

fn resolve_y(transform: &mut Transform, velocity: &mut Velocity, terrain: &mut Terrain) {
    if !terrain.overlaps_solid(transform.x, transform.y, transform.width, transform.height) {
        return;
    }
    if velocity.y > 0.0 {
        transform.y = (transform.y + transform.height - EDGE_EPS).floor() - transform.height;
    } else if velocity.y < 0.0 {
        transform.y = transform.y.floor() + 1.0;
    }
    velocity.y = 0.0;
}
