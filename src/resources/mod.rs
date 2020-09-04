use na::Vector2;
use rapier2d::geometry::{ContactEvent, ProximityEvent};
use rapier2d::pipeline::EventHandler;
use std::borrow::BorrowMut;

use crate::types::{ContactEventQueue, ProximityEventQueue};

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

#[derive(Clone, Default)]
pub struct PhysicsEventCollector {
    pub proximity_queue: ProximityEventQueue,
    pub contact_queue: ContactEventQueue,
}

impl EventHandler for PhysicsEventCollector {
    fn handle_contact_event(&self, e: ContactEvent) {
        self.contact_queue.push(e);
    }
    fn handle_proximity_event(&self, e: ProximityEvent) {
        self.proximity_queue.push(e);
    }
}
