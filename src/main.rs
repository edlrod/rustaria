mod camera;
mod ecs;
mod position;
mod render;
mod world;

use camera::Camera;
use ecs::components::Transform;
use macroquad::prelude::*;
use world::World;

use crate::ecs::{
    components::{ColorRenderer, Player, Velocity},
    systems,
};

fn seed_from_time() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() ^ (d.as_secs() as u32))
        .unwrap_or(42)
}

#[macroquad::main("Rustaria")]
async fn main() {
    let seed = seed_from_time();
    println!("world seed: {seed}");
    let mut tiles = World::new(seed);

    let mut entities = hecs::World::new();
    let player: hecs::Entity = entities.spawn((
        Transform {
            x: 0.0,
            y: -40.0,
            width: 1.5,
            height: 2.5,
        },
        Velocity { x: 0.0, y: 0.0 },
        ColorRenderer { color: RED },
        Player,
    ));

    let mut camera = Camera::new(16.0);

    loop {
        // Update
        let dt = get_frame_time();
        systems::player_controller::player_controller(&mut entities, &mut tiles);
        systems::physics::physics(&mut entities, &mut tiles, dt);

        if let Ok(transform) = entities.get::<&Transform>(player) {
            camera.follow((transform.x, transform.y));
        }

        // Draw
        clear_background(SKYBLUE);
        camera.set();

        render::draw_world(&mut tiles, &camera);
        render::draw_entities(&mut entities);

        next_frame().await
    }
}
