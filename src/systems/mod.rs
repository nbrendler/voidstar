use legion::*;
use na::Vector2;
use rapier2d::dynamics::RigidBodyHandle;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

use crate::components::{Player, Transform};
use crate::input::{InputEvent, InputQueue, InputState, Key, KeyState};
use crate::physics::Physics;
use crate::resources::WorldBounds;

const MAX_VELOCITY: f32 = 100.0;
const MAX_ANGULAR_VELOCITY: f32 = 5.0;
const FRICTION: f32 = 50.0;

#[system]
fn physics(#[resource] physics: &mut Physics) {
    physics.step();
}

#[system(for_each)]
fn physics_transform(t: &mut Transform, handle: &RigidBodyHandle, #[resource] physics: &Physics) {
    // updates transforms with information from the physics system.
    let rb = physics.bodies.get(*handle).unwrap();

    t.set_isometry_2d(rb.position);
}

#[system]
fn input(#[resource] input_queue: &mut InputQueue, #[resource] input_state: &mut InputState) {
    for e in input_queue.drain(..) {
        match e {
            InputEvent::KeyboardEvent {
                code,
                state,
                repeated,
            } => {
                if state == KeyState::Pressed {
                    input_state.press_key(code, repeated)
                } else {
                    input_state.release_key(code)
                }
            }
            // TODO: mouse events
            _ => {}
        }
    }
}

#[system(for_each)]
fn world_wrap(
    handle: &mut RigidBodyHandle,
    #[resource] bounds: &WorldBounds,
    #[resource] physics: &mut Physics,
) {
    let mut rb = physics.bodies.get_mut(*handle).unwrap();
    let v = &mut rb.position.translation.vector;
    let w = bounds.width as f32;
    let h = bounds.height as f32;

    if v.x < 0.0 || v.x > w {
        v.x = (v.x + w) % w;
    }

    if v.y < 0.0 || v.y > h {
        v.y = (v.y + h) % h;
    }
}

#[system(for_each)]
fn player_movement(
    _p: &Player,
    handle: &mut RigidBodyHandle,
    #[resource] input_state: &InputState,
    #[resource] physics: &mut Physics,
) {
    // Ideally this should look at some kind of Key mapping data to figure out which keys do what.
    let mut rb = physics.bodies.get_mut(*handle).unwrap();
    if input_state.is_pressed(Key::Left) || input_state.is_pressed(Key::A) {
        rb.apply_torque_impulse(0.5);
    } else if rb.angvel > 0.0 {
        rb.angvel -= rb.angvel * (1. / FRICTION);
    }
    if input_state.is_pressed(Key::Right) || input_state.is_pressed(Key::D) {
        rb.apply_torque_impulse(-0.5);
    } else if rb.angvel < 0.0 {
        rb.angvel -= rb.angvel * (1. / FRICTION);
    }
    if input_state.is_pressed(Key::Up) || input_state.is_pressed(Key::W) {
        let angle = rb.position.rotation.angle();
        rb.apply_force(Vector2::new(100. * -angle.sin(), 100. * angle.cos()));
    } else if rb.linvel.norm() > 0.0 {
        let m = rb.linvel.norm();
        rb.linvel.set_magnitude(m - m / FRICTION);
    }
    let m = rb.linvel.norm();
    if m > MAX_VELOCITY {
        rb.linvel.set_magnitude(MAX_VELOCITY);
    }
    rb.angvel = rb
        .angvel
        .min(MAX_ANGULAR_VELOCITY)
        .max(-MAX_ANGULAR_VELOCITY);
}

#[cfg(not(target_arch = "wasm32"))]
#[system]
fn fps(#[state] frame_count: &mut u64, #[state] last_call: &mut std::time::Instant) {
    *frame_count += 1;

    let elapsed = Instant::now() - *last_call;
    if elapsed > Duration::from_secs(3) {
        info!(
            "FPS: {:02.02}",
            *frame_count as f64 / elapsed.as_secs() as f64
        );
        *frame_count = 0;
        *last_call = Instant::now();
    }
}

fn init_common(builder: &mut legion::systems::Builder) -> &mut legion::systems::Builder {
    builder
        .add_system(input_system())
        .add_system(player_movement_system())
        .add_system(physics_transform_system())
        .add_system(physics_system())
        .add_system(world_wrap_system())
}

#[cfg(target_arch = "wasm32")]
pub fn init() -> Schedule {
    let mut builder = Schedule::builder();
    init_common(&mut builder).build()
}
#[cfg(not(target_arch = "wasm32"))]
pub fn init() -> Schedule {
    let mut builder = Schedule::builder();
    init_common(&mut builder);

    builder.add_system(fps_system(0, Instant::now()));
    builder.build()
}
