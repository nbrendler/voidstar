use na::Vector2;
use std::borrow::BorrowMut;

pub struct WorldBounds(pub Vector2<u32>);

impl WorldBounds {
    pub fn as_f32(&self) -> Vector2<f32> {
        Vector2::new(self.0.x as f32, self.0.y as f32)
    }
}

impl Default for WorldBounds {
    fn default() -> Self {
        WorldBounds(Vector2::new(100, 50))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WindowDimensions {
    pub aspect_ratio: f32,
    pub w: u32,
    pub h: u32,
}

impl Default for WindowDimensions {
    fn default() -> Self {
        WindowDimensions {
            w: 960,
            h: 540,
            aspect_ratio: 960_f32 / 540_f32,
        }
    }
}
