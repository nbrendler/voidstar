use legion::*;
use na::{UnitComplex, Vector2};
use rapier2d::dynamics::RigidBodyHandle;

use crate::components::{Player, Transform};
use crate::input::{InputEvent, InputQueue, InputState, Key, KeyState};
use crate::physics::Physics;

const MAX_VELOCITY: f32 = 100.0;

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
fn player_movement(
    _p: &Player,
    handle: &mut RigidBodyHandle,
    #[resource] input_state: &InputState,
    #[resource] physics: &mut Physics,
) {
    // Ideally this should look at some kind of Key mapping data to figure out which keys do what.
    let mut rb = physics.bodies.get_mut(*handle).unwrap();
    if input_state.is_pressed(Key::Left) || input_state.is_pressed(Key::A) {
        rb.position
            .append_rotation_wrt_center_mut(&UnitComplex::new(0.05));
    }
    if input_state.is_pressed(Key::Right) || input_state.is_pressed(Key::D) {
        rb.position
            .append_rotation_wrt_center_mut(&UnitComplex::new(-0.05));
    }
    if input_state.is_pressed(Key::Up) || input_state.is_pressed(Key::W) {
        let angle = rb.position.rotation.angle();
        rb.apply_force(Vector2::new(100. * -angle.sin(), 100. * angle.cos()));
        let m = rb.linvel.norm();
        if m > MAX_VELOCITY {
            rb.linvel.set_magnitude(MAX_VELOCITY)
        }
    }
}

pub fn init() -> Schedule {
    Schedule::builder()
        .add_system(input_system())
        .add_system(player_movement_system())
        .add_system(physics_transform_system())
        .add_system(physics_system())
        .build()
}
