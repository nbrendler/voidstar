use legion::storage::IntoComponentSource;
use legion::World;
use na::Vector3;
use rapier2d::dynamics::{RigidBodyBuilder, RigidBodyHandle};
use rapier2d::geometry::ColliderBuilder;

use crate::components::{Player, Projectile, Sprite, Transform};
use crate::physics::Physics;
use crate::resources::WorldBounds;

pub fn create_bullet(
    physics: &mut Physics,
    mut start: Transform,
) -> (Transform, Sprite, Projectile, RigidBodyHandle) {
    let mut bullet_vec = start.isometry.rotation * Vector3::y();
    start.isometry.translation.vector += bullet_vec;
    bullet_vec *= 30.0;

    let rbb = RigidBodyBuilder::new_dynamic()
        .position(start.as_2d())
        .linvel(bullet_vec.x, bullet_vec.y)
        .can_sleep(false);
    let collider = ColliderBuilder::cuboid(0.5, 0.5).sensor(true);

    (
        start,
        Sprite {
            index: 2,
            color: [1., 0., 0.],
        },
        Projectile,
        physics.create(rbb, vec![collider]),
    )
}

pub fn create_player(
    physics: &mut Physics,
    bounds: &WorldBounds,
) -> (Transform, Sprite, Player, RigidBodyHandle) {
    let center = bounds.as_f32() / 2.0;
    let rbb = RigidBodyBuilder::new_dynamic()
        .translation(center.x, center.y)
        .can_sleep(false);
    let collider = ColliderBuilder::cuboid(0.5, 0.5);

    (
        Transform::default().with_translation(Vector3::new(center.x, center.y, 1.)),
        Sprite {
            index: 1,
            color: [1., 1., 1.],
        },
        Player,
        physics.create(rbb, vec![collider]),
    )
}
