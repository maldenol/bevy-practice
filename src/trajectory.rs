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

    let delta_time = time.timestep().as_secs_f32();
    for iteration_index in 0..TRAJECTORY_LENGTH {
        for (entity_dst, grav_pred_dst) in &grav_predictions {
            let mut delta_vel = Vec3::ZERO;
            let mut max_influence = (0.0, u32::MAX);

            for (entity_src, grav_pred_src) in &grav_predictions {
                if grav_pred_src.mass.is_none() {
                    continue;
                }

                if entity_src == entity_dst {
                    continue;
                }

                let current_delta_vel = calc_delta_velocity(
                    delta_time,
                    grav_pred_dst.pos,
                    grav_pred_src.pos,
                    grav_pred_src.mass.unwrap(),
                );

                let current_influence = current_delta_vel.length();
                if max_influence.0 < current_influence && (grav_pred_dst.mass.is_none() || grav_pred_dst.mass.unwrap() < grav_pred_src.mass.unwrap()) {
                    max_influence.0 = current_influence;
                    max_influence.1 = *entity_src;
                }

                delta_vel += current_delta_vel;
            }

            let vel = &grav_pred_dst.vel;
            vel.set(vel.get() + delta_vel);

            if max_influence.1 == u32::MAX {
                continue;
            }

            let central_body_pos = grav_predictions.get(&max_influence.1).unwrap().pos;
            let central_body_vec = grav_pred_dst.pos - central_body_pos;
            if grav_pred_dst.traj.get().central_body_index != max_influence.1 {
                grav_pred_dst.traj.set(TrajectoryParams {
                    initial_pos: grav_pred_dst.traj.get().initial_pos,
                    central_body_index: max_influence.1,
                    central_body_vec_initial: central_body_vec,
                    ..default()
                });
            } else if grav_pred_dst.traj.get().trajectory_length == TRAJECTORY_LENGTH && grav_pred_dst.traj.get().central_body_index != u32::MAX {
                let central_body_angle = central_body_vec.angle_between(grav_pred_dst.traj.get().central_body_vec_initial);
                if grav_pred_dst.traj.get().central_body_angle > central_body_angle {
                    grav_pred_dst.traj.set(TrajectoryParams {
                        central_body_angle_decreases: true,
                        ..grav_pred_dst.traj.get()
                    });
                } else if grav_pred_dst.traj.get().central_body_angle_decreases {
                    grav_pred_dst.traj.set(TrajectoryParams {
                        trajectory_length: iteration_index + 1,
                        ..grav_pred_dst.traj.get()
                    });
                }
                grav_pred_dst.traj.set(TrajectoryParams {
                    central_body_angle,
                    ..grav_pred_dst.traj.get()
                });
            }
        }

        for (_, grav_pred) in &mut grav_predictions {
            grav_pred.pos += delta_time * grav_pred.vel.get();
        }

        for (entity, grav_pred) in &grav_predictions {
            let mut traj_pos = grav_pred.pos;
            if grav_pred.traj.get().central_body_index != u32::MAX {
                let central_body = grav_predictions.get(&grav_pred.traj.get().central_body_index).unwrap();
                traj_pos -= central_body.pos;
                traj_pos += central_body.traj.get().initial_pos;
            }
            polyline_vertices.get_mut(entity).unwrap().push(traj_pos);
        }
    }

    let space_camera = space_camera.into_inner();

    for (entity, mut vertices) in polyline_vertices {
        if *trajectory_mode == TrajectoryMode::Focused {
            let Some(focused_entity) = space_camera.get_look_at_entity() else {
                continue;
            };

            if entity != focused_entity.index_u32() {
                continue;
            }
        }

        let trajectory_length = grav_predictions.get(&entity).unwrap().traj.get().trajectory_length as usize;
        vertices.truncate(trajectory_length);

        gizmos.primitive_3d(
            &Polyline3d{ vertices },
            Isometry3d::default(),
            Color::srgb(TRAJECTORY_COLOR, TRAJECTORY_COLOR, TRAJECTORY_COLOR),
        );
    }
}
