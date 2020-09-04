use std::collections::{HashSet, VecDeque};

#[cfg(not(target_arch = "wasm32"))]
use glfw::Key as GKey;
#[cfg(target_arch = "wasm32")]
use web_sys::{KeyboardEvent, MouseEvent};

#[derive(Default, Debug)]
pub struct InputState {
    pressed_keys: HashSet<Key>,
    repeated_key: Option<Key>,
}

impl InputState {
    pub fn release_key(&mut self, code: Key) {
        self.pressed_keys.remove(&code);
    }
    pub fn press_key(&mut self, code: Key, repeated: bool) {
        self.pressed_keys.insert(code);
        if repeated {
            self.repeated_key = Some(code);
        } else {
            self.repeated_key = None;
        }
    }
    pub fn is_pressed(&self, code: Key) -> bool {
        self.pressed_keys.contains(&code)
    }
}

#[derive(Debug, PartialEq)]
pub enum MouseButton {
    Main,
    Aux,
    Secondary,
    Fourth,
    Fifth,
}

#[derive(Debug, PartialEq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug, PartialEq)]
pub enum InputEvent {
    KeyboardEvent {
        code: Key,
        state: KeyState,
        repeated: bool,
    },
    MouseEvent {
        button: MouseButton,
        state: KeyState,
        x: i32,
        y: i32,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Key {
    W,
    A,
    S,
    D,
    Left,
    Right,
    Up,
    Down,
    Space,
    Unmapped,
}

#[cfg(target_arch = "wasm32")]
impl From<MouseEvent> for InputEvent {
    fn from(e: MouseEvent) -> InputEvent {
        InputEvent::MouseEvent {
            button: match e.button() {
                0 => MouseButton::Main,
                1 => MouseButton::Aux,
                2 => MouseButton::Secondary,
                3 => MouseButton::Fourth,
                4 => MouseButton::Fifth,
                _ => unreachable!("What is this mouse button?"),
            },
            state: KeyState::Pressed,
            x: e.client_x(),
            y: e.client_y(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<KeyboardEvent> for InputEvent {
    fn from(e: KeyboardEvent) -> InputEvent {
        InputEvent::KeyboardEvent {
            code: (e.code().as_str()).into(),
            state: KeyState::Pressed,
            repeated: e.repeat(),
        }
    }
}

impl From<&str> for Key {
    fn from(s: &str) -> Key {
        match s {
            "KeyW" => Key::W,
            "KeyA" => Key::A,
            "KeyS" => Key::S,
            "KeyD" => Key::D,
            "ArrowLeft" => Key::Left,
            "ArrowRight" => Key::Right,
            "ArrowUp" => Key::Up,
            "ArrowDown" => Key::Down,
            "Space" => Key::Space,
            _ => Key::Unmapped,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<GKey> for Key {
    fn from(k: glfw::Key) -> Key {
        match k {
            GKey::W => Key::W,
            GKey::A => Key::A,
            GKey::S => Key::S,
            GKey::D => Key::D,
            GKey::Up => Key::Up,
            GKey::Left => Key::Left,
            GKey::Right => Key::Right,
            GKey::Down => Key::Down,
            GKey::Space => Key::Space,
            _ => Key::Unmapped,
        }
    }
}
