use std::collections::HashMap;

use legion::Entity;
use legion::Resources;
use rapier2d::dynamics::{IntegrationParameters, JointSet, RigidBodySet};
use rapier2d::geometry::{BroadPhase, ColliderSet, ContactEvent, NarrowPhase, ProximityEvent};
use rapier2d::na::Vector2;
use rapier2d::pipeline::{EventHandler, PhysicsPipeline};

pub use rapier2d::dynamics::{RigidBodyBuilder, RigidBodyHandle};
pub use rapier2d::geometry::{ColliderBuilder, Proximity};

use crate::event_queue::{Drain, SharedEventQueue};

pub struct Physics {
    pipeline: PhysicsPipeline,
    gravity: Vector2<f32>,
    integration_parameters: IntegrationParameters,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub joints: JointSet,
    event_handler: PhysicsEventCollector,
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
            event_handler: PhysicsEventCollector::default(),
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

    pub fn cleanup(&mut self, world: &mut legion::World) {
        let mut to_remove = vec![];
        for (h, e) in self.event_handler.entity_map.iter() {
            if world.entry(*e).is_none() {
                to_remove.push(*h);
            }
        }

        if cfg!(debug) && !to_remove.is_empty() {
            debug!("Physics cleanup. Removing {} rigid bodies", to_remove.len());
        }

        for h in to_remove.iter() {
            self.pipeline.remove_rigid_body(
                *h,
                &mut self.broad_phase,
                &mut self.narrow_phase,
                &mut self.bodies,
                &mut self.colliders,
                &mut self.joints,
            );
        }
    }

    pub fn create(
        &mut self,
        entity: Entity,
        rbb: RigidBodyBuilder,
        collider_builders: Vec<ColliderBuilder>,
    ) -> RigidBodyHandle {
        let h = self.bodies.insert(rbb.build());
        for cb in collider_builders {
            self.colliders.insert(cb.build(), h, &mut self.bodies);
        }
        self.event_handler.entity_map.insert(h, entity);
        h
    }

    // TODO: figure out how to do this without allocating a Vec
    pub fn proximity_events(&mut self) -> Vec<EntityProximityEvent> {
        self.event_handler
            .proximity_queue
            .get_mut()
            .drain()
            .collect()
    }

    pub fn contact_events(&mut self) -> Vec<EntityContactEvent> {
        self.event_handler.contact_queue.get_mut().drain().collect()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum EntityContactEvent {
    Started(Entity, Entity),
    Stopped(Entity, Entity),
}

#[derive(Debug, Copy, Clone)]
pub struct EntityProximityEvent {
    pub e1: Entity,
    pub e2: Entity,
    pub prev_status: Proximity,
    pub new_status: Proximity,
}

#[derive(Clone, Default)]
struct PhysicsEventCollector {
    entity_map: HashMap<RigidBodyHandle, Entity>,
    contact_queue: SharedEventQueue<EntityContactEvent>,
    proximity_queue: SharedEventQueue<EntityProximityEvent>,
}

impl EventHandler for PhysicsEventCollector {
    fn handle_contact_event(&self, e: ContactEvent) {
        match e {
            ContactEvent::Started(h1, h2) => {
                let contact_entity_1 = self.entity_map.get(&h1).unwrap();
                let contact_entity_2 = self.entity_map.get(&h2).unwrap();

                self.contact_queue.push(EntityContactEvent::Started(
                    *contact_entity_1,
                    *contact_entity_2,
                ));
            }
            ContactEvent::Stopped(h1, h2) => {
                let contact_entity_1 = self.entity_map.get(&h1).unwrap();
                let contact_entity_2 = self.entity_map.get(&h2).unwrap();

                self.contact_queue.push(EntityContactEvent::Stopped(
                    *contact_entity_1,
                    *contact_entity_2,
                ));
            }
        }
    }
    fn handle_proximity_event(&self, e: ProximityEvent) {
        let collider_1 = self.entity_map.get(&e.collider1).unwrap();
        let collider_2 = self.entity_map.get(&e.collider2).unwrap();

        self.proximity_queue.push(EntityProximityEvent {
            e1: *collider_1,
            e2: *collider_2,
            prev_status: e.prev_status,
            new_status: e.new_status,
        });
    }
}
