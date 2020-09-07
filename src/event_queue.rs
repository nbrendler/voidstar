use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};
use std::vec::Drain as D;

pub trait Drain {
    type Item;
    fn drain(&mut self) -> D<Self::Item>;
}

#[derive(Clone, Debug)]
pub struct EventQueue<EventType>(Rc<RefCell<Vec<EventType>>>);

impl<T> Default for EventQueue<T> {
    fn default() -> Self {
        EventQueue(Rc::new(RefCell::new(vec![])))
    }
}

// We don't really care about Arc<Mutex> but the physics library requires event handlers to be Send
// + Sync, and the interface (rapier2d::pipeline::EventHandler) requires an immutable reference so
// I want to have interior mutability. It was designed to work with channels but I'd rather keep it
// simple here.
#[derive(Clone, Debug)]
pub struct SharedEventQueue<EventType>(Arc<Mutex<Vec<EventType>>>);

impl<T> Default for SharedEventQueue<T> {
    fn default() -> Self {
        SharedEventQueue(Arc::new(Mutex::new(vec![])))
    }
}

impl<EventType> EventQueue<EventType> {
    pub fn push(&self, val: EventType) {
        self.0.borrow_mut().push(val)
    }

    pub fn get_mut(&self) -> EventQueueWrapper<EventType> {
        EventQueueWrapper {
            r: self.0.borrow_mut(),
        }
    }
}

impl<EventType> SharedEventQueue<EventType> {
    pub fn push(&self, val: EventType) {
        self.0.lock().unwrap().push(val)
    }

    pub fn get_mut(&self) -> SharedEventQueueWrapper<EventType> {
        SharedEventQueueWrapper {
            r: self.0.lock().unwrap(),
        }
    }
}

pub struct SharedEventQueueWrapper<'a, T: 'a> {
    r: MutexGuard<'a, Vec<T>>,
}

pub struct EventQueueWrapper<'a, T: 'a> {
    r: RefMut<'a, Vec<T>>,
}

impl<'a, 'b: 'a, T> Drain for EventQueueWrapper<'b, T> {
    type Item = T;

    fn drain(&mut self) -> D<'_, T> {
        self.r.drain(..)
    }
}

impl<'a, 'b: 'a, T> Drain for SharedEventQueueWrapper<'b, T> {
    type Item = T;

    fn drain(&mut self) -> D<'_, T> {
        self.r.drain(..)
    }
}

mod test {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum MyEventType {
        SomeEvent,
        AnotherEvent,
    }

    #[test]
    fn drain_events() {
        let e: EventQueue<MyEventType> = EventQueue::default();
        e.push(MyEventType::AnotherEvent);
        e.push(MyEventType::SomeEvent);
        let drained = e.get_mut().drain().collect::<Vec<MyEventType>>();
        assert_eq!(
            &drained[..],
            &[MyEventType::AnotherEvent, MyEventType::SomeEvent]
        );
        let q = e.0.borrow();
        assert!(q.is_empty());
    }

    #[test]
    fn shared_drain_events() {
        let e: SharedEventQueue<MyEventType> = SharedEventQueue::default();
        e.push(MyEventType::AnotherEvent);
        e.push(MyEventType::SomeEvent);
        let drained = e.get_mut().drain().collect::<Vec<MyEventType>>();
        assert_eq!(
            &drained[..],
            &[MyEventType::AnotherEvent, MyEventType::SomeEvent]
        );
        let q = e.0.lock().unwrap();
        assert!(q.is_empty());
    }
}
