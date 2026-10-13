mod camera;
mod ecs;
mod light;
mod position;
mod render;
mod terrain;
mod tile;
mod time;

use camera::Camera;
use ecs::components::Transform;
use macroquad::prelude::*;
use terrain::Terrain;

use crate::ecs::{
    components::{ColorRenderer, Cursor, Player, Velocity},
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
    let tile_registry = tile::load(tile::DEFAULT_PATH).expect("failed to load tile config");
    let mut terrain = Terrain::new(seed, tile_registry);

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

    entities.spawn((
        Transform {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        ColorRenderer { color: WHITE },
        Cursor,
    ));

    let mut camera = Camera::new(16.0);

    loop {
        // Update
        let dt = time::frame_delta();
        systems::player_controller::player_controller(&mut entities, &mut terrain);
        systems::cursor_controller::cursor_controller(&mut entities, &mut terrain, &camera);
        systems::physics::physics(&mut entities, &mut terrain, dt);

        if let Ok(transform) = entities.get::<&Transform>(player) {
            camera.follow((transform.x, transform.y));
        }

        // Draw
        clear_background(SKYBLUE);
        camera.set(dt);

        render::draw_terrain(&mut terrain, &camera);
        render::draw_entities(&mut entities, &mut terrain);

        next_frame().await
    }
}
