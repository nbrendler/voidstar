use std::sync::Arc;

use legion::storage::IntoComponentSource;
use legion::systems::{CommandBuffer, WorldWritable};
use legion::{Entity, EntityStore, Resources, World};
use na::{Vector2, Vector3};

use crate::components::*;
use crate::physics::{ColliderBuilder, Physics, RigidBodyBuilder, RigidBodyHandle};
use crate::resources::WorldBounds;
use crate::types::*;

pub trait EntityBuilder {
    type Components: legion::storage::IntoComponentSource;

    fn components(&self) -> Self::Components;
    fn into_components(self) -> Self::Components
    where
        Self: Sized,
    {
        self.components()
    }
    fn create(&self, world: &mut World, physics: &mut Physics) {
        let entities = self.create_entities(world);
        let handles = self.create_physics(physics, &entities);
        self.update_world(world, &entities, &handles);
    }
    fn create_entities(&self, world: &mut World) -> Vec<Entity> {
        world.extend(self.components()).to_vec()
    }
    fn create_physics(&self, physics: &mut Physics, entities: &[Entity]) -> Vec<RigidBodyHandle>;
    fn update_world(&self, world: &mut World, entities: &[Entity], handles: &[RigidBodyHandle]) {
        for (e, h) in entities.iter().zip(handles.iter()) {
            world.entry(*e).map(|mut e| e.add_component(h.clone()));
        }
    }
}

#[derive(Debug, Default)]
pub struct AsteroidBuilder {
    positions: Vec<Transform>,
}

impl AsteroidBuilder {
    pub fn add_asteroid<T: Into<Transform>>(mut self, t: T) -> Self {
        self.positions.push(t.into());
        self
    }
}

impl EntityBuilder for AsteroidBuilder {
    type Components = Vec<(Transform, Sprite, EntityTag)>;

    fn components(&self) -> Self::Components {
        self.positions
            .iter()
            .map(|p| {
                (
                    p.clone(),
                    Sprite {
                        index: 3,
                        color: [1., 1., 1.],
                    },
                    EntityTag::ASTEROID,
                )
            })
            .collect::<Self::Components>()
    }

    fn create_physics(&self, physics: &mut Physics, entities: &[Entity]) -> Vec<RigidBodyHandle> {
        entities
            .iter()
            .zip(self.positions.iter())
            .map(|(e, t)| {
                let rbb = RigidBodyBuilder::new_dynamic().position(t.as_2d());
                let collider = ColliderBuilder::ball(0.2).density(20.0);
                physics.create(rbb, vec![collider])
            })
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct BulletBuilder {
    positions: Vec<(Transform, Vector3<f32>)>,
}

impl BulletBuilder {
    pub fn starting_from(t: Transform, speed: f32) -> Self {
        let bullet_vec = speed * (t.isometry.rotation * Vector3::y());
        BulletBuilder {
            positions: vec![(t, bullet_vec)],
        }
    }
}

impl EntityBuilder for BulletBuilder {
    type Components = Vec<(Transform, Sprite, EntityTag, Projectile, Cull)>;

    fn components(&self) -> Self::Components {
        self.positions
            .iter()
            .map(|(p, _)| {
                (
                    *p,
                    Sprite {
                        index: 2,
                        color: [1., 0., 0.],
                    },
                    EntityTag::PROJECTILE,
                    Projectile {
                        can_hit: EntityTag::ENEMY_OR_ASTEROID,
                    },
                    Cull,
                )
            })
            .collect::<Self::Components>()
    }

    fn create_physics(&self, physics: &mut Physics, entities: &[Entity]) -> Vec<RigidBodyHandle> {
        entities
            .iter()
            .zip(self.positions.iter())
            .map(|(_e, (t, bullet))| {
                let rbb = RigidBodyBuilder::new_dynamic()
                    .position(t.as_2d())
                    .linvel(bullet.x, bullet.y)
                    .can_sleep(false);
                let collider = ColliderBuilder::cuboid(0.1, 0.3).sensor(true);
                physics.create(rbb, vec![collider])
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct PlayerBuilder {
    positions: Vec<Transform>,
}

impl PlayerBuilder {
    pub fn starting_from(t: Transform) -> Self {
        PlayerBuilder { positions: vec![t] }
    }
}

impl EntityBuilder for PlayerBuilder {
    type Components = Vec<(Transform, Sprite, EntityTag, Player)>;

    fn components(&self) -> Self::Components {
        self.positions
            .iter()
            .map(|p| {
                (
                    *p,
                    Sprite {
                        index: 1,
                        color: [1., 1., 1.],
                    },
                    EntityTag::PROJECTILE,
                    Player,
                )
            })
            .collect::<Self::Components>()
    }

    fn create_physics(&self, physics: &mut Physics, entities: &[Entity]) -> Vec<RigidBodyHandle> {
        entities
            .iter()
            .zip(self.positions.iter())
            .map(|(e, t)| {
                let rbb = RigidBodyBuilder::new_dynamic()
                    .translation(
                        t.isometry.translation.vector.x,
                        t.isometry.translation.vector.y,
                    )
                    .can_sleep(false);
                let collider = ColliderBuilder::cuboid(0.5, 0.5);
                physics.create(rbb, vec![collider])
            })
            .collect()
    }
}
