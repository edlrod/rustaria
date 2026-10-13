use macroquad::input::{MouseButton, is_mouse_button_down, mouse_position};

use crate::camera::Camera;
use crate::ecs::components::{Cursor, Transform};
use crate::tile::Tile;
use crate::terrain::Terrain;

pub fn cursor_controller(entities: &mut hecs::World, terrain: &mut Terrain, camera: &Camera) {
    for transform in entities.query_mut::<&mut Transform>().with::<&Cursor>() {
        let mouse_position = mouse_position();

        let world_position = camera.screen_to_world(mouse_position.0, mouse_position.1);

        transform.x = world_position.0.floor();
        transform.y = world_position.1.floor();

        if is_mouse_button_down(MouseButton::Left) {
            terrain.set_tile((transform.x as i32, transform.y as i32), Tile::Dirt);
        }
        if is_mouse_button_down(MouseButton::Right) {
            terrain.set_tile((transform.x as i32, transform.y as i32), Tile::Air);
        }
    }
}
