#![allow(dead_code)]
#![allow(unused_macros, dead_code, unused_imports)]
#[macro_use]
extern crate log;
extern crate legion;
extern crate nalgebra as na;
extern crate rapier2d;
#[macro_use]
extern crate bitflags;

// TODO:
// asteroids
// damage
// multi-sprite things
// crabs
// the void*
// collecting stuff
// AI
// UI
// collider shapes
// animations

use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use console_log;
#[cfg(not(target_arch = "wasm32"))]
use glfw::WindowEvent;
use log::info;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::{KeyboardEvent, MouseEvent};

#[macro_use]
pub mod utils;
pub mod components;
pub mod constants;
pub mod event_queue;
pub mod factories;
pub mod input;
pub mod physics;
pub mod renderer;
pub mod resources;
pub mod spritesheet;
pub mod systems;
pub mod types;

use crate::factories::{AsteroidBuilder, EntityBuilder, PlayerBuilder};
#[cfg(target_arch = "wasm32")]
use crate::input::KeyState;
use crate::input::{InputEvent, InputState};
use crate::physics::Physics;
use crate::resources::*;
use crate::systems::init as init_systems;
use crate::types::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn init() -> Result<(), JsValue> {
    utils::set_panic_hook();
    console_log::init();
    Ok(())
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Game {
    world: legion::World,
    resources: legion::Resources,
    schedule: legion::Schedule,
    renderer: renderer::Renderer,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Game {
    pub fn new() -> Self {
        info!("Creating game!");

        let mut world = legion::World::default();
        let mut physics = Physics::default();
        let world_bounds = WorldBounds::default();
        let window_dimensions = WindowDimensions::default();

        PlayerBuilder::starting_from((world_bounds.as_f32() / 2.0).into())
            .create(&mut world, &mut physics);
        AsteroidBuilder::default()
            .add_asteroid((50., 30.))
            .add_asteroid((45., 30.))
            .add_asteroid((55., 30.))
            .create(&mut world, &mut physics);
        let mut resources = legion::Resources::default();
        resources.insert(InputState::default());
        resources.insert(InputEventQueue::default());
        resources.insert(physics);
        resources.insert(world_bounds);
        resources.insert(window_dimensions);
        resources.insert(ViewMatrix::default());

        Game {
            renderer: renderer::Renderer::new(&window_dimensions),
            world,
            resources,
            schedule: init_systems(),
        }
    }

    pub fn tick(&mut self) {
        self.schedule.execute(&mut self.world, &mut self.resources);
        self.renderer.draw(&mut self.world, &self.resources);

        //let mut physics = self.resources.get_mut::<Physics>().unwrap();
        //physics.cleanup(&mut self.world);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn log_event(&mut self, e: InputEvent) {
        let input_q = self.resources.get_or_default::<InputEventQueue>();
        input_q.push(e);
    }
    #[cfg(target_arch = "wasm32")]
    fn log_event(&mut self, e: InputEvent) {
        let input_q = self.resources.get_or_default::<InputEventQueue>();
        input_q.push(e);
    }

    #[cfg(target_arch = "wasm32")]
    pub fn log_mouseup_event(&mut self, e: MouseEvent) {
        let mut ie: InputEvent = e.into();
        match &mut ie {
            InputEvent::MouseEvent { state, .. } => {
                *state = KeyState::Released;
            }
            _ => {}
        }
        self.log_event(ie);
    }

    #[cfg(target_arch = "wasm32")]
    pub fn log_mousedown_event(&mut self, e: MouseEvent) {
        let ie: InputEvent = e.into();
        self.log_event(ie);
    }

    #[cfg(target_arch = "wasm32")]
    pub fn log_keydown_event(&mut self, e: KeyboardEvent) {
        let ie: InputEvent = e.into();
        self.log_event(ie);
    }

    #[cfg(target_arch = "wasm32")]
    pub fn log_keyup_event(&mut self, e: KeyboardEvent) {
        let mut ie: InputEvent = e.into();
        match &mut ie {
            InputEvent::KeyboardEvent { state, .. } => {
                *state = KeyState::Released;
            }
            _ => {}
        }
        self.log_event(ie);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn iter_events(&mut self) -> std::sync::mpsc::TryIter<(f64, WindowEvent)> {
        self.renderer.iter_events()
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
