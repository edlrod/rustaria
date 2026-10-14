use crate::position::{ChunkPosition, Position};
use crate::terrain::CHUNK_SIZE;
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

    pub fn set(&mut self, dt: f32) {
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
            let follow_speed = 6.0;
            let t = 1.0 - (-follow_speed * dt).exp();
            self.position.x += (target.0 - self.position.x) * t;
            self.position.y += (target.1 - self.position.y) * t;
        }
    }

    /// All chunk coordinates that contain at least one visible tile.
    pub fn visible_chunks(&self) -> impl Iterator<Item = ChunkPosition> {
        let (start_x, start_y, end_x, end_y) = self.visible_bounds();

        let start_cx = start_x.div_euclid(CHUNK_SIZE);
        let end_cx = (end_x - 1).div_euclid(CHUNK_SIZE);
        let start_cy = start_y.div_euclid(CHUNK_SIZE);
        let end_cy = (end_y - 1).div_euclid(CHUNK_SIZE);

        (start_cy..=end_cy).flat_map(move |cy| (start_cx..=end_cx).map(move |cx| (cx, cy)))
    }

    /// Visible tile bounds as `(start_x, start_y, end_x, end_y)`, ends
    /// exclusive, padded by one tile in each direction.
    fn visible_bounds(&self) -> (i32, i32, i32, i32) {
        let half_width_tiles = (screen_width() / self.ppu / 2.0).ceil() as i32;
        let half_height_tiles = (screen_height() / self.ppu / 2.0).ceil() as i32;

        let center_x = self.position.x as i32;
        let center_y = self.position.y as i32;

        (
            center_x - half_width_tiles - 1,
            center_y - half_height_tiles - 1,
            center_x + half_width_tiles + 1,
            center_y + half_height_tiles + 1,
        )
    }

    pub fn screen_to_world(&self, mouse_x: f32, mouse_y: f32) -> Position {
        let center_mouse_x = mouse_x - screen_width() / 2.0;
        let center_mouse_y = mouse_y - screen_height() / 2.0;

        (
            self.position.x + center_mouse_x / self.ppu,
            self.position.y + center_mouse_y / self.ppu,
        )
    }
}
