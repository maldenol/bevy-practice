use std::time::Duration;

use bevy::{math::*, prelude::*, time::TimeSystems};
use strum::{EnumCount, EnumDiscriminants, FromRepr, VariantNames};

use crate::app::AppSet;

pub const GRAVITY_CONSTANT: f32 = 100.0;

#[derive(Component)]
pub struct Velocity(pub Vec3);

#[derive(Component)]
pub struct Mass(pub f32);

#[derive(Component, Default)]
pub struct CentralBody(pub u32);

#[derive(
    States,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Debug,
    EnumDiscriminants,
    EnumCount,
    FromRepr,
    VariantNames,
)]
pub enum SimulationState {
    #[default]
    Paused,
    Running,
}

#[derive(Resource)]
pub struct SimulationSpeed(pub f32);

impl Default for SimulationSpeed {
    fn default() -> Self {
        SimulationSpeed(1.0)
    }
}

pub struct GravityPlugin;

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SimulationState>()
            .init_resource::<SimulationSpeed>()
            .insert_resource(Time::<Fixed>::from_hz(60.0))
            .add_systems(First, apply_simulation_speed.before(TimeSystems))
            .add_systems(
                FixedUpdate,
                simulate_gravity.run_if(in_state(SimulationState::Running)),
            )
            .add_systems(
                Update,
                update_central_body
                    .run_if(in_state(SimulationState::Running))
                    .in_set(AppSet::Simulation),
            );
    }
}

/// Bevy drops virtual time beyond this much real time so a hitch cannot spiral.
const MAX_REAL_DELTA: Duration = Duration::from_millis(250);

/// Speeds the fixed loop up or down without changing the size of a tick.
///
/// `Time<Fixed>` stays at 60 Hz, so every tick is 1/60 of a second. `SimulationSpeed`
/// scales virtual time, which changes how many of those ticks run per real second.
fn apply_simulation_speed(
    simulation_speed: Res<SimulationSpeed>,
    simulation_state: Res<State<SimulationState>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    let speed = if *simulation_state.get() == SimulationState::Running {
        simulation_speed.0
    } else {
        0.0
    };

    virtual_time.set_relative_speed(speed);
    virtual_time.set_max_delta(MAX_REAL_DELTA.mul_f64(f64::from(speed.max(1.0))));
}

/// Positions and velocities copied out of the world so one step can run without ECS.
pub struct BodySample {
    pub id: u32,
    pub pos: Vec3,
    pub vel: Vec3,
    pub mass: Option<f32>,
}

/// Semi-implicit Euler. Every acceleration uses the positions from the start of the step.
pub fn integrate_bodies(bodies: &mut [BodySample], dt: f32) {
    let mut delta_vels = vec![Vec3::ZERO; bodies.len()];

    for (i, dst) in bodies.iter().enumerate() {
        for src in bodies.iter() {
            let Some(mass) = src.mass else {
                continue;
            };
            if src.id == dst.id {
                continue;
            }

            delta_vels[i] += calc_delta_velocity(dt, dst.pos, src.pos, mass);
        }
    }

    for (body, delta_vel) in bodies.iter_mut().zip(delta_vels) {
        body.vel += delta_vel;
        body.pos += body.vel * dt;
    }
}

/// Heaviest pull from a strictly heavier body.
pub fn dominant_central_body(bodies: &[BodySample], id: u32) -> Option<u32> {
    let dst = bodies.iter().find(|body| body.id == id)?;
    let mut best = (0.0_f32, u32::MAX);

    for other in bodies {
        let Some(mass) = other.mass else {
            continue;
        };
        if other.id == id {
            continue;
        }
        if dst.mass.is_some_and(|dst_mass| dst_mass >= mass) {
            continue;
        }

        let influence = calc_delta_velocity(1.0, dst.pos, other.pos, mass).length();
        if influence > best.0 {
            best = (influence, other.id);
        }
    }

    if best.1 == u32::MAX {
        None
    } else {
        Some(best.1)
    }
}

fn simulate_gravity(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Velocity, Option<&Mass>)>,
) {
    let mut bodies: Vec<BodySample> = query
        .iter()
        .map(|(entity, transform, velocity, mass)| BodySample {
            id: entity.index_u32(),
            pos: transform.translation,
            vel: velocity.0,
            mass: mass.map(|mass| mass.0),
        })
        .collect();

    integrate_bodies(&mut bodies, time.delta_secs());

    for (entity, mut transform, mut velocity, _) in &mut query {
        let body = bodies
            .iter()
            .find(|body| body.id == entity.index_u32())
            .unwrap();
        transform.translation = body.pos;
        velocity.0 = body.vel;
    }
}

fn update_central_body(mut query: Query<(Entity, &Transform, Option<&Mass>, &mut CentralBody)>) {
    let bodies: Vec<BodySample> = query
        .iter()
        .map(|(entity, transform, mass, _)| BodySample {
            id: entity.index_u32(),
            pos: transform.translation,
            vel: Vec3::ZERO,
            mass: mass.map(|mass| mass.0),
        })
        .collect();

    for (entity, _, _, mut central_body) in &mut query {
        central_body.0 = dominant_central_body(&bodies, entity.index_u32()).unwrap_or(u32::MAX);
    }
}

pub fn calc_delta_velocity(delta_time: f32, pos_dst: Vec3, pos_src: Vec3, mass_src: f32) -> Vec3 {
    let dist2 = pos_src.distance_squared(pos_dst);
    let accel = GRAVITY_CONSTANT * mass_src / dist2;
    let dir = (pos_src - pos_dst).normalize();
    dir * accel * delta_time
}
