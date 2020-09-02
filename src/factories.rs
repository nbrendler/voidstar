use legion::World;
use na::Vector3;
use rapier2d::dynamics::RigidBodyBuilder;
use rapier2d::geometry::ColliderBuilder;

use crate::components::{Player, Sprite, Transform};
use crate::physics::Physics;
use crate::resources::WorldBounds;

pub fn create_bullet() {}

pub fn create_player(world: &mut World, physics: &mut Physics, bounds: &WorldBounds) {
    let center = bounds.as_f32() / 2.0;
    let rbb = RigidBodyBuilder::new_dynamic()
        .translation(center.x, center.y)
        .can_sleep(false);
    let collider = ColliderBuilder::cuboid(0.5, 0.5);

    world.push((
        Player,
        Transform::default().with_translation(Vector3::new(center.x, center.y, 1.)),
        physics.create(rbb, vec![collider]),
        Sprite {
            index: 1,
            color: [1., 1., 1.],
        },
    ));
}
