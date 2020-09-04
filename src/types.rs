use na::Matrix4;
use rapier2d::geometry::{ContactEvent, ProximityEvent};

use crate::event_queue::SharedEventQueue;
use crate::input::InputEvent;

pub type InputEventQueue = SharedEventQueue<InputEvent>;
pub type ContactEventQueue = SharedEventQueue<ContactEvent>;
pub type ProximityEventQueue = SharedEventQueue<ProximityEvent>;

#[derive(Default)]
pub struct ViewMatrix(pub Matrix4<f32>);
