use std::collections::{HashSet, VecDeque};

use web_sys::{KeyboardEvent, MouseEvent};

pub type InputQueue = VecDeque<InputEvent>;

#[derive(Default, Debug)]
pub struct InputState {
    pressed_keys: HashSet<String>,
    repeated_key: Option<String>,
}

impl InputState {
    pub fn release_key(&mut self, code: &str) {
        self.pressed_keys.remove(code);
    }
    pub fn press_key(&mut self, code: &str, repeated: bool) {
        self.pressed_keys.insert(code.to_owned());
        if repeated {
            self.repeated_key = Some(code.to_owned());
        } else {
            self.repeated_key = None;
        }
    }
    pub fn is_pressed(&self, code: &str) -> bool {
        self.pressed_keys.contains(code)
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
        code: String,
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

impl From<KeyboardEvent> for InputEvent {
    fn from(e: KeyboardEvent) -> InputEvent {
        InputEvent::KeyboardEvent {
            code: e.code(),
            state: KeyState::Pressed,
            repeated: e.repeat(),
        }
    }
}
