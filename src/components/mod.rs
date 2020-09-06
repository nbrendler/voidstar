mod sprite;
mod transform;

pub use sprite::*;
pub use transform::*;

bitflags! {
    #[rustfmt::ignore]
    pub struct EntityTag: u32 {
        const PLAYER     = 0b00000001;
        const ENEMY      = 0b00000010;
        const ASTEROID   = 0b00000100;
        const PROJECTILE = 0b00001000;

        const ENEMY_OR_ASTEROID  = 0b00000110;
        const PLAYER_OR_ASTEROID = 0b00000101;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Player;
#[derive(Copy, Clone, Debug)]
pub struct Projectile {
    pub can_hit: EntityTag,
}

/// should be culled when it goes offscreen
#[derive(Copy, Clone, Debug)]
pub struct Cull;
