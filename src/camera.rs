use crate::position::{Position, TilePosition};
use macroquad::prelude::*;

pub struct Camera {
    position: Vec2,
    target: Option<Position>,
    pub(crate) ppu: f32,
}

impl Camera {
    pub fn new(ppu: f32) -> Self {
        Self {
            position: vec2(0.0, 0.0),
            target: None,
            ppu,
        }
    }

    pub fn follow(&mut self, target: Position) {
        self.target = Some(target);
    }

    pub fn set(&mut self) {
        set_camera(&Camera2D {
            target: self.position,
            offset: Vec2::ZERO,
            zoom: vec2(
                2.0 * self.ppu / screen_width(),
                2.0 * self.ppu / screen_height(),
            ),
            ..Default::default()
        });

        if let Some(target) = self.target {
            let dt = get_frame_time();
            let follow_speed = 6.0;
            let t = 1.0 - (-follow_speed * dt).exp();
            self.position.x += (target.0 - self.position.x) * t;
            self.position.y += (target.1 - self.position.y) * t;
        }
    }

    pub fn visible_tiles(&self) -> impl Iterator<Item = TilePosition> {
        let half_width_tiles = (screen_width() / self.ppu / 2.0).ceil() as i32;
        let half_height_tiles = (screen_height() / self.ppu / 2.0).ceil() as i32;

        let center_x = self.position.x as i32;
        let center_y = self.position.y as i32;

        let start_x = center_x - half_width_tiles - 1;
        let end_x = center_x + half_width_tiles + 1;
        let start_y = center_y - half_height_tiles - 1;
        let end_y = center_y + half_height_tiles + 1;

        (start_y..end_y).flat_map(move |y| (start_x..end_x).map(move |x| (x, y)))
    }
}
