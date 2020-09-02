use rapier2d::dynamics::{
    IntegrationParameters, JointSet, RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
};
use rapier2d::geometry::{BroadPhase, ColliderBuilder, ColliderSet, NarrowPhase};
use rapier2d::na::Vector2;
use rapier2d::pipeline::PhysicsPipeline;

pub struct Physics {
    pipeline: PhysicsPipeline,
    gravity: Vector2<f32>,
    integration_parameters: IntegrationParameters,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub joints: JointSet,
    event_handler: (),
}

impl Default for Physics {
    fn default() -> Self {
        Physics {
            pipeline: PhysicsPipeline::new(),
            gravity: Vector2::new(0., 0.),
            integration_parameters: IntegrationParameters::default(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            joints: JointSet::new(),
            event_handler: (),
        }
    }
}

impl Physics {
    pub fn step(&mut self) {
        self.pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joints,
            &self.event_handler,
        )
    }

    pub fn create(
        &mut self,
        rbb: RigidBodyBuilder,
        collider_builders: Vec<ColliderBuilder>,
    ) -> RigidBodyHandle {
        let h = self.bodies.insert(rbb.build());
        for cb in collider_builders {
            self.colliders.insert(cb.build(), h, &mut self.bodies);
        }
        h
    }
}
