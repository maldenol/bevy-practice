use bevy::{
    math::*,
    prelude::*
};
use strum::{EnumCount, EnumDiscriminants, FromRepr, VariantNames};

use crate::{
    camera::SpaceCamera,
    gravity::{BodySample, Mass, Velocity, dominant_central_body, integrate_bodies},
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

#[derive(Clone, Copy)]
struct TrajectoryParams {
    initial_pos: Vec3,
    central_body_vec_initial: Vec3,
    central_body_angle: f32,
    central_body_angle_decreases: bool,
    trajectory_length: u16,
}

impl Default for TrajectoryParams {
    fn default() -> Self {
        TrajectoryParams {
            initial_pos: Vec3::ZERO,
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

    let mut bodies = Vec::new();
    let mut trajs = Vec::new();
    let mut polyline_vertices: Vec<Vec<Vec3>> = Vec::new();

    for (entity, transform, velocity, mass) in query {
        bodies.push(BodySample {
            id: entity.index_u32(),
            pos: transform.translation,
            vel: velocity.0,
            mass: mass.map(|mass| mass.0),
        });
        trajs.push(TrajectoryParams {
            initial_pos: transform.translation,
            ..default()
        });
        polyline_vertices.push(Vec::new());
    }

    // One central body for the whole line. Switching it later jumps the draw frame by the
    // distance between the two bodies, which is the chord / spike in the polyline.
    let mut central_slots: Vec<Option<usize>> = vec![None; bodies.len()];
    for i in 0..bodies.len() {
        let Some(central_id) = dominant_central_body(&bodies, bodies[i].id) else {
            continue;
        };
        let central_slot = bodies.iter().position(|body| body.id == central_id).unwrap();
        central_slots[i] = Some(central_slot);
        trajs[i].central_body_vec_initial = bodies[i].pos - bodies[central_slot].pos;
    }

    let delta_time = time.timestep().as_secs_f32();
    for iteration_index in 0..TRAJECTORY_LENGTH {
        integrate_bodies(&mut bodies, delta_time);

        for i in 0..bodies.len() {
            let Some(central_slot) = central_slots[i] else {
                continue;
            };
            if trajs[i].trajectory_length != TRAJECTORY_LENGTH {
                continue;
            }

            let central_body_vec = bodies[i].pos - bodies[central_slot].pos;
            let central_body_angle = central_body_vec.angle_between(trajs[i].central_body_vec_initial);
            if trajs[i].central_body_angle > central_body_angle {
                trajs[i].central_body_angle_decreases = true;
            } else if trajs[i].central_body_angle_decreases {
                trajs[i].trajectory_length = iteration_index + 1;
            }
            trajs[i].central_body_angle = central_body_angle;
        }

        for i in 0..bodies.len() {
            let Some(central_slot) = central_slots[i] else {
                continue;
            };
            if iteration_index >= trajs[i].trajectory_length {
                continue;
            }

            let traj_pos = bodies[i].pos - bodies[central_slot].pos + trajs[central_slot].initial_pos;
            let radius = bodies[i].pos.distance(bodies[central_slot].pos);
            let spacing = MIN_VERTEX_SPACING.max(radius * VERTEX_SPACING_FRACTION);
            let vertices = &mut polyline_vertices[i];
            let closing = iteration_index + 1 == trajs[i].trajectory_length;
            let far_enough = vertices.last().is_none_or(|last| last.distance(traj_pos) >= spacing);
            if far_enough || closing {
                vertices.push(traj_pos);
            }
        }
    }

    let space_camera = space_camera.into_inner();

    for (i, vertices) in polyline_vertices.into_iter().enumerate() {
        if *trajectory_mode == TrajectoryMode::Focused {
            let Some(focused_entity) = space_camera.get_look_at_entity() else {
                continue;
            };

            if bodies[i].id != focused_entity.index_u32() {
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
