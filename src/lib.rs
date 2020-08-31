#![allow(dead_code)]
#![allow(unused_macros)]
extern crate nalgebra as na;

// TODO:
// * Debug Logging
// * FPS limiter/recorder
// * Draw colliders
// * Friction/Inertia

#[macro_use]
pub mod utils;
pub mod components;
pub mod input;
pub mod physics;
pub mod renderer;
pub mod resources;
pub mod spritesheet;
pub mod systems;

use crate::components::{Player, Sprite, Transform};
#[cfg(target_arch = "wasm32")]
use crate::input::KeyState;
use crate::input::{InputEvent, InputQueue, InputState};
use crate::physics::Physics;
use crate::resources::WorldBounds;
use crate::systems::init as init_systems;
#[cfg(not(target_arch = "wasm32"))]
use glfw::WindowEvent;
use na::Vector3;
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::{KeyboardEvent, MouseEvent};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn init() -> Result<(), JsValue> {
    utils::set_panic_hook();
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
        let mut world = legion::World::default();
        let mut physics = Physics::default();
        let world_bounds = WorldBounds::default();

        create_player(&mut world, &mut physics);
        create_static(&mut world, &mut physics, (10., 15.));
        create_static(&mut world, &mut physics, (20., 15.));
        create_static(&mut world, &mut physics, (15., 20.));
        create_static(&mut world, &mut physics, (15., 10.));
        let mut resources = legion::Resources::default();
        resources.insert(InputState::default());
        resources.insert(InputQueue::default());
        resources.insert(physics);
        resources.insert(world_bounds);

        Game {
            renderer: renderer::Renderer::new(),
            world,
            resources,
            schedule: init_systems(),
        }
    }

    pub fn tick(&mut self) {
        self.schedule.execute(&mut self.world, &mut self.resources);
        self.renderer.draw(&mut self.world, &self.resources);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn log_event(&mut self, e: InputEvent) {
        let mut input_q = self.resources.get_mut_or_default::<InputQueue>();
        input_q.push_back(e);
    }

    #[cfg(target_arch = "wasm32")]
    fn log_event(&mut self, e: InputEvent) {
        let mut input_q = self.resources.get_mut_or_default::<InputQueue>();
        input_q.push_back(e);
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

fn create_static(world: &mut legion::World, physics: &mut Physics, pos: (f32, f32)) {
    world.push((
        Transform::default()
            .with_translation(Vector3::new(pos.0, pos.1, 0.))
            .with_scale(Vector3::new(2., 2., 1.)),
        physics.create_static(|rbb| rbb.translation(pos.0, pos.1)),
        Sprite {
            index: 0,
            color: [1., 0., 1.],
        },
    ));
}

fn create_player(world: &mut legion::World, physics: &mut Physics) {
    world.push((
        Player,
        Transform::default().with_translation(Vector3::new(15., 15., 1.)),
        physics.create_dynamic(|rbb| rbb.translation(15., 15.)),
        Sprite {
            index: 1,
            color: [1., 1., 1.],
        },
    ));
}
