pub struct WorldBounds {
    pub width: u32,
    pub height: u32,
}

impl Default for WorldBounds {
    fn default() -> Self {
        WorldBounds {
            width: 100,
            height: 50,
        }
    }
}
