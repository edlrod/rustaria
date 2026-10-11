use macroquad::prelude::get_frame_time;

// Largest simulated time step, in seconds.
pub const MAX_DELTA: f32 = 0.1;

/// The current frame's delta, clamped to [`MAX_DELTA`].
pub fn frame_delta() -> f32 {
    get_frame_time().min(MAX_DELTA)
}
