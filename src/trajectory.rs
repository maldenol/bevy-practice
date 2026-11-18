use std::cell::Cell;

use bevy::{
    math::*,
    platform::collections::HashMap,
    prelude::*
};
use strum::{EnumCount, EnumDiscriminants, FromRepr, VariantNames};

use crate::{
    camera::SpaceCamera,
    gravity::{Velocity, Mass, calc_delta_velocity},
};

const TRAJECTORY_LENGTH: u16 = 60 * 600;
const TRAJECTORY_COLOR: f32 = 0.1;
/// Minimum spacing between drawn points, so sub-pixel gizmo segments don't spike.
const MIN_VERTEX_SPACING: f32 = 4.0;
/// Also space points by this fraction of the distance to the central body.
const VERTEX_SPACING_FRACTION: f32 = 0.015;

#[derive(Resource, Default, Copy, Clone, PartialEq, Eq, Hash, Debug, EnumDiscriminants, EnumCount, FromRepr, VariantNames)]
pub enum TrajectoryMode {
    None,
    Focused,
    #[default]
    All,
}

struct GravityPrediction {
    pos: Vec3,
    vel: Cell<Vec3>,
    mass: Option<f32>,
    traj: Cell<TrajectoryParams>,
}

#[derive(Clone, Copy)]
struct TrajectoryParams {
    initial_pos: Vec3,
    central_body_index: u32,
    central_body_vec_initial: Vec3,
    central_body_angle: f32,
    central_body_angle_decreases: bool,
    trajectory_length: u16,
}

impl Default for TrajectoryParams {
    fn default() -> Self {
        TrajectoryParams {
            initial_pos: Vec3::ZERO,
            central_body_index: u32::MAX,
            central_body_vec_initial: Vec3::ZERO,
            central_body_angle: 0.0,
            central_body_angle_decreases: false,
            trajectory_length: TRAJECTORY_LENGTH,
        }
    }
}

pub fn startup_trajectory(
    mut commands: Commands,
) {
    commands.insert_resource(TrajectoryMode::default());
}

pub fn draw_trajectory(
    mut gizmos: Gizmos,
    query: Query<(Entity, &Transform, &Velocity, Option<&Mass>)>,
    time: Res<Time::<Fixed>>,
    trajectory_mode: Res<TrajectoryMode>,
    space_camera: Single<&SpaceCamera>,
) {
    if *trajectory_mode == TrajectoryMode::None {
        return;
    }

    let mut polyline_vertices: HashMap<u32, Vec<Vec3>> = HashMap::new();
    let mut grav_predictions: HashMap<u32, GravityPrediction> = HashMap::new();

    for (entity, transform, velocity, mass) in query {
        polyline_vertices.insert(entity.index_u32(), Vec::with_capacity(TRAJECTORY_LENGTH as usize));
        grav_predictions.insert(entity.index_u32(), GravityPrediction {
            pos: transform.translation,
            vel: Cell::new(velocity.0),
            mass: mass.map(|mass| mass.0),
            traj: Cell::new(TrajectoryParams {
                initial_pos: transform.translation,
                ..default()
            }),
        });
    }

    // One central body for the whole line. Switching it later jumps the draw frame by the
    // distance between the two bodies, which is the chord / spike in the polyline.
    let central_bodies: Vec<(u32, Option<u32>)> = grav_predictions
        .keys()
        .copied()
        .map(|entity| (entity, dominant_central_body(entity, &grav_predictions)))
        .collect();
    for (entity, central_index) in central_bodies {
        let Some(central_index) = central_index else {
            continue;
        };
        let grav_pred = grav_predictions.get(&entity).unwrap();
        let central_body_pos = grav_predictions.get(&central_index).unwrap().pos;
        let central_body_vec = grav_pred.pos - central_body_pos;
        grav_pred.traj.set(TrajectoryParams {
            initial_pos: grav_pred.traj.get().initial_pos,
            central_body_index: central_index,
            central_body_vec_initial: central_body_vec,
            ..default()
        });
    }

    let delta_time = time.timestep().as_secs_f32();
    for iteration_index in 0..TRAJECTORY_LENGTH {
        for (entity_dst, grav_pred_dst) in &grav_predictions {
            let mut delta_vel = Vec3::ZERO;

            for (entity_src, grav_pred_src) in &grav_predictions {
                if grav_pred_src.mass.is_none() {
                    continue;
                }

                if entity_src == entity_dst {
                    continue;
                }

                delta_vel += calc_delta_velocity(
                    delta_time,
                    grav_pred_dst.pos,
                    grav_pred_src.pos,
                    grav_pred_src.mass.unwrap(),
                );
            }

            let vel = &grav_pred_dst.vel;
            vel.set(vel.get() + delta_vel);

            let mut traj = grav_pred_dst.traj.get();
            if traj.central_body_index == u32::MAX || traj.trajectory_length != TRAJECTORY_LENGTH {
                continue;
            }

            let central_body_pos = grav_predictions.get(&traj.central_body_index).unwrap().pos;
            let central_body_vec = grav_pred_dst.pos - central_body_pos;
            let central_body_angle = central_body_vec.angle_between(traj.central_body_vec_initial);
            if traj.central_body_angle > central_body_angle {
                traj.central_body_angle_decreases = true;
            } else if traj.central_body_angle_decreases {
                traj.trajectory_length = iteration_index + 1;
            }
            traj.central_body_angle = central_body_angle;
            grav_pred_dst.traj.set(traj);
        }

        for (_, grav_pred) in &mut grav_predictions {
            grav_pred.pos += delta_time * grav_pred.vel.get();
        }

        for (entity, grav_pred) in &grav_predictions {
            let traj = grav_pred.traj.get();
            if traj.central_body_index == u32::MAX || iteration_index >= traj.trajectory_length {
                continue;
            }

            let central_body = grav_predictions.get(&traj.central_body_index).unwrap();
            let traj_pos = grav_pred.pos - central_body.pos + central_body.traj.get().initial_pos;
            let radius = grav_pred.pos.distance(central_body.pos);
            let spacing = MIN_VERTEX_SPACING.max(radius * VERTEX_SPACING_FRACTION);
            let vertices = polyline_vertices.get_mut(entity).unwrap();
            let closing = iteration_index + 1 == traj.trajectory_length;
            let far_enough = vertices.last().is_none_or(|last| last.distance(traj_pos) >= spacing);
            if far_enough || closing {
                vertices.push(traj_pos);
            }
        }
    }

    let space_camera = space_camera.into_inner();

    for (entity, vertices) in polyline_vertices {
        if *trajectory_mode == TrajectoryMode::Focused {
            let Some(focused_entity) = space_camera.get_look_at_entity() else {
                continue;
            };

            if entity != focused_entity.index_u32() {
                continue;
            }
        }

        if vertices.len() < 2 {
            continue;
        }

        gizmos.primitive_3d(
            &Polyline3d{ vertices },
            Isometry3d::default(),
            Color::srgb(TRAJECTORY_COLOR, TRAJECTORY_COLOR, TRAJECTORY_COLOR),
        );
    }
}

/// Heaviest pull from a strictly heavier body. Chosen once per prediction.
fn dominant_central_body(
    entity: u32,
    bodies: &HashMap<u32, GravityPrediction>,
) -> Option<u32> {
    let dst_pos = bodies[&entity].pos;
    let dst_mass = bodies[&entity].mass;
    let mut best = (0.0_f32, u32::MAX);

    for (src, other) in bodies {
        let Some(mass) = other.mass else {
            continue;
        };
        if *src == entity {
            continue;
        }
        if dst_mass.is_some_and(|dst_mass| dst_mass >= mass) {
            continue;
        }

        let influence = calc_delta_velocity(1.0, dst_pos, other.pos, mass).length();
        if influence > best.0 {
            best = (influence, *src);
        }
    }

    if best.1 == u32::MAX {
        None
    } else {
        Some(best.1)
    }
}
