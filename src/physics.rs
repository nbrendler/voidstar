use rapier2d::dynamics::{
    IntegrationParameters, JointSet, RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
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

    pub fn create_dynamic<F: FnOnce(RigidBodyBuilder) -> RigidBodyBuilder>(
        &mut self,
        func: F,
    ) -> RigidBodyHandle {
        let builder = func(RigidBodyBuilder::new_dynamic());
        let collider = ColliderBuilder::cuboid(1., 1.).build();
        let h = self.bodies.insert(builder.build());
        self.colliders.insert(collider, h, &mut self.bodies);
        h
    }

    pub fn create_static<F: FnOnce(RigidBodyBuilder) -> RigidBodyBuilder>(
        &mut self,
        func: F,
    ) -> RigidBodyHandle {
        let builder = func(RigidBodyBuilder::new_static());
        let collider = ColliderBuilder::cuboid(1., 1.).build();
        let h = self.bodies.insert(builder.build());
        self.colliders.insert(collider, h, &mut self.bodies);
        h
    }
}
