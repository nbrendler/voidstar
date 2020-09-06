use na::Matrix4;

use crate::event_queue::SharedEventQueue;
use crate::input::InputEvent;

pub type InputEventQueue = SharedEventQueue<InputEvent>;

#[derive(Default)]
pub struct ViewMatrix(pub Matrix4<f32>);
