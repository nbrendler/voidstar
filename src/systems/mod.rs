use std::collections::HashMap;
use std::time::Duration;

use instant::Instant;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;
use na::Vector2;

use crate::components::*;
use crate::constants::SPRITES_PER_HALF_SCREEN;
use crate::event_queue::Drain;
use crate::factories::{BulletBuilder, EntityBuilder};
use crate::input::{InputEvent, InputState, Key, KeyState};
use crate::physics::{Physics, RigidBodyHandle};
use crate::resources::*;
use crate::types::*;

const MAX_VELOCITY: f32 = 10.0;
const MAX_ANGULAR_VELOCITY: f32 = 2.0;
const FRICTION: f32 = 50.0;

#[system]
#[read_component(EntityTag)]
fn physics(world: &mut SubWorld, #[resource] physics: &mut Physics) {
    physics.step();

    for e in physics.proximity_events().iter() {
        let (etag1, etag2) = {
            let entity1 = world.entry_ref(e.e1).unwrap();
            let entity2 = world.entry_ref(e.e2).unwrap();
            (
                entity1.into_component::<EntityTag>().ok().unwrap(),
                entity2.into_component::<EntityTag>().ok().unwrap(),
            )
        };

        match (*etag1, *etag2) {
            (EntityTag::PROJECTILE, EntityTag::ASTEROID) => {
                info!("hit! {:?}", e.e2);
            }
            (EntityTag::ASTEROID, EntityTag::PROJECTILE) => {
                info!("hit! {:?}", e.e1);
            }
            (_, _) => {}
        }
    }
    for e in physics.contact_events().iter() {
        info!("{:?}", e);
    }
}

#[system(for_each)]
fn physics_transform(t: &mut Transform, handle: &RigidBodyHandle, #[resource] physics: &Physics) {
    // updates transforms with information from the physics system.
    let rb = physics.bodies.get(*handle).unwrap();

    t.set_isometry_2d(rb.position);
}

#[system(for_each)]
fn culling(
    cmd: &mut CommandBuffer,
    cull_t: &Transform,
    _: &Cull,
    e: &Entity,
    #[resource] dims: &WindowDimensions,
    #[resource] view: &ViewMatrix,
) {
    // Manual culling of things that are offscreen, like bullets.

    // BUG: this culls things that wrap around the world boundary before the player has.
    // try all 9 possible positions?
    let pos = view.0 * cull_t.isometry.translation.vector.push(1.);
    if pos.x < -SPRITES_PER_HALF_SCREEN
        || pos.x > SPRITES_PER_HALF_SCREEN
        || pos.y < -SPRITES_PER_HALF_SCREEN / dims.aspect_ratio
        || pos.y > SPRITES_PER_HALF_SCREEN / dims.aspect_ratio
    {
        debug!("Culling bullet {:?} (pos: {:?}", e, pos);
        cmd.remove(*e);
    }
}

#[system]
fn input(#[resource] input_queue: &mut InputEventQueue, #[resource] input_state: &mut InputState) {
    for e in input_queue.get_mut().drain() {
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
            let builder = BulletBuilder::starting_from(*t, 30.0);
            let e = cmd.push(builder.components()[0]);
            cmd.add_component(e, builder.create_physics(physics, &[e])[0]);

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
#[system(for_each)]
fn player_position(_p: &Player, t: &Transform, #[resource] view: &mut ViewMatrix) {
    *view.0 = *t
        .isometry
        .translation
        .to_homogeneous()
        .try_inverse()
        .unwrap();
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
        .add_system(player_position_system())
        .add_system(physics_system())
        .add_system(world_wrap_system())
        .add_system(culling_system())
        .add_system(fps_system(0, Instant::now()))
        .build()
}
