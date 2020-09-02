use na::Vector2;

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
