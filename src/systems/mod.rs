use std::time::Duration;

use instant::Instant;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;
use na::Vector2;
use rapier2d::dynamics::RigidBodyHandle;

use crate::components::{Player, Projectile, Sprite, Transform};
use crate::factories::create_bullet;
use crate::input::{InputEvent, InputQueue, InputState, Key, KeyState};
use crate::physics::Physics;
use crate::resources::WorldBounds;

const MAX_VELOCITY: f32 = 10.0;
const MAX_ANGULAR_VELOCITY: f32 = 2.0;
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
    let bounds = bounds.as_f32();

    if v.x < 0.0 || v.x > bounds.x {
        v.x = (v.x + bounds.x) % bounds.x;
    }

    if v.y < 0.0 || v.y > bounds.y {
        v.y = (v.y + bounds.y) % bounds.y;
    }
}

#[system]
#[read_component(Player)]
#[read_component(Transform)]
fn player_shoot(
    world: &mut SubWorld,
    cmd: &mut CommandBuffer,
    #[resource] input_state: &InputState,
    #[resource] physics: &mut Physics,
    #[state] last_shot: &mut Instant,
) {
    for (_, t) in <(&Player, &Transform)>::query().iter(world) {
        let now = Instant::now();
        if input_state.is_pressed(Key::Space) && now - *last_shot > Duration::from_millis(300) {
            cmd.push(create_bullet(physics, t.clone()));

            *last_shot = now;
        }
    }
}

#[system(for_each)]
fn player_input(
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

#[system]
fn fps(#[state] frame_count: &mut u64, #[state] last_call: &mut Instant) {
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

pub fn init() -> Schedule {
    Schedule::builder()
        .add_system(input_system())
        .add_system(player_input_system())
        .add_system(player_shoot_system(Instant::now()))
        .add_system(physics_transform_system())
        .add_system(physics_system())
        .add_system(world_wrap_system())
        .add_system(fps_system(0, Instant::now()))
        .build()
}
