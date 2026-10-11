mod camera;
mod position;
mod world;

use camera::Camera;
use macroquad::prelude::*;
use world::World;

use crate::position::Position;

#[macroquad::main("Rusteria")]
async fn main() {
    let mut target: Position = (0.0, 0.0);

    let mut camera = Camera::new(16.0);
    let mut world = World::new();

    loop {
        clear_background(SKYBLUE);
        camera.set();

        let dt = get_frame_time();

        for (x, y) in camera.visible_tiles() {
            let tile_id = world.get_tile((x, y));

            if tile_id == 1 {
                draw_rectangle(x as f32, y as f32, 1.0, 1.0, Color::from_hex(0xff0000));
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
