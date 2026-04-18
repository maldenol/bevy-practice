use bevy::{
    math::*,
    prelude::*,
};
use strum::{EnumCount, EnumDiscriminants, FromRepr, VariantNames};

pub const GRAVITY_CONSTANT: f32 = 100.0;

#[derive(Component)]
pub struct Velocity(pub Vec3);

#[derive(Component)]
pub struct Mass(pub f32);

#[derive(Component, Default)]
pub struct CentralBody(pub u32);

#[derive(States, Default, Copy, Clone, PartialEq, Eq, Hash, Debug, EnumDiscriminants, EnumCount, FromRepr, VariantNames)]
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

pub fn startup_gravity(
    mut commands: Commands,
) {
    commands.insert_resource(SimulationSpeed::default());
}

pub fn update_velocity(
    time: Res<Time>,
    mut gravity_dst: Query<(Entity, &Transform, &mut Velocity)>,
    gravity_src: Query<(Entity, &Transform, &Mass)>,
) {
    for (entity_dst, transform_dst, mut velocity_dst) in gravity_dst.iter_mut() {
        for (entity_src, transform_src, mass_src) in gravity_src {
            if entity_src == entity_dst {
                continue;
            }

            let delta_velocity = calc_delta_velocity(
                time.delta_secs(),
                transform_dst.translation,
                transform_src.translation,
                mass_src.0,
            );

            velocity_dst.0 += delta_velocity;
        }
    }
}

pub fn update_position(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity)>,
) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0 * time.delta_secs();
    }
}

pub fn update_central_body(
    mut gravity_dst: Query<(Entity, &Transform, Option<&Mass>, &mut CentralBody)>,
    gravity_src: Query<(Entity, &Transform, &Mass)>,
) {
    for (entity_dst, transform_dst, mass_dst, mut central_body_dst) in gravity_dst.iter_mut() {
        let mut max_influence = (0.0, u32::MAX);

        for (entity_src, transform_src, mass_src) in gravity_src {
            if entity_src == entity_dst {
                continue;
            }

            if mass_dst.is_some() && mass_dst.unwrap().0 > mass_src.0 {
                continue;
            }

            let delta_velocity = calc_delta_velocity(
                1.0,
                transform_dst.translation,
                transform_src.translation,
                mass_src.0,
            );

            let current_influence = delta_velocity.length();
            if max_influence.0 < current_influence {
                max_influence.0 = current_influence;
                max_influence.1 = entity_src.index_u32();
            }
        }

        central_body_dst.0 = max_influence.1;
    }
}

pub fn calc_delta_velocity(
    delta_time: f32,
    pos_dst: Vec3,
    pos_src: Vec3,
    mass_src: f32
) -> Vec3 {
    let dist2 = pos_src.distance_squared(pos_dst);
    let accel = GRAVITY_CONSTANT * mass_src / dist2;
    let dir = (pos_src - pos_dst).normalize();
    dir * accel * delta_time
}
