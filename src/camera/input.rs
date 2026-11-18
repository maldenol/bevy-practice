use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    math::*,
    picking::backend::ray::RayMap,
    prelude::*,
};

use super::SpaceCamera;

const MOVE_SPEED: f32 = 10.0;
const ROT_SPEED: f32 = 0.01;
const ZOOM_SPEED: f32 = 0.1;
const DEFAULT_DISTANCE: f32 = 500.0;

pub(super) fn handle_keyboard_input(
    key_code_input: Res<ButtonInput<KeyCode>>,
    camera: Single<(&mut Transform, &mut SpaceCamera)>,
) {
    let (mut camera_transform, mut space_camera) = camera.into_inner();

    let mut move_dir = Vec3::ZERO;
    let w = key_code_input.pressed(KeyCode::KeyW);
    let a = key_code_input.pressed(KeyCode::KeyA);
    let s = key_code_input.pressed(KeyCode::KeyS);
    let d = key_code_input.pressed(KeyCode::KeyD);
    if w != s {
        move_dir +=
            camera_transform.right().cross(Vec3::Z).normalize() * if s { 1.0 } else { -1.0 };
        move_dir = move_dir.normalize();
    }
    if a != d {
        move_dir += camera_transform.right() * if d { 1.0 } else { -1.0 };
        move_dir = move_dir.normalize();
    }
    if move_dir != Vec3::ZERO {
        let rel_pos = camera_transform.translation - space_camera.get_look_at();
        let zoom = rel_pos.length() / DEFAULT_DISTANCE;
        let delta_pos = move_dir * MOVE_SPEED * zoom;
        camera_transform.translation += delta_pos;
        *space_camera.get_look_at_mut() += delta_pos;
    }
}

pub(super) fn handle_mouse_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mouse_motion_input: Res<AccumulatedMouseMotion>,
    mouse_scroll_input: Res<AccumulatedMouseScroll>,
    mut ray_cast: MeshRayCast,
    ray_map: Res<RayMap>,
    camera: Single<(&mut Transform, &mut SpaceCamera)>,
) {
    let (mut camera_transform, mut space_camera) = camera.into_inner();

    let lmb = mouse_button_input.pressed(MouseButton::Left);
    let rmb = mouse_button_input.pressed(MouseButton::Right);
    if lmb != rmb {
        if lmb {
            if mouse_button_input.just_pressed(MouseButton::Left) {
                let ray = *ray_map.iter().next().unwrap().1;
                if let Some((hit_entity, _)) = ray_cast
                    .cast_ray(ray, &MeshRayCastSettings::default())
                    .first()
                {
                    space_camera.try_look_at(*hit_entity);
                }
            }
        } else {
            let mut rel_pos = camera_transform.translation - space_camera.get_look_at();
            rel_pos = rel_pos.rotate_axis(Vec3::Z, ROT_SPEED * -mouse_motion_input.delta.x);
            let new_rel_pos = rel_pos.rotate_axis(
                camera_transform.right().into(),
                ROT_SPEED * -mouse_motion_input.delta.y,
            );
            if new_rel_pos.normalize().dot(Vec3::Z).abs() <= 0.95 {
                rel_pos = new_rel_pos;
            }
            *camera_transform = Transform::from_translation(rel_pos + space_camera.get_look_at())
                .looking_at(*space_camera.get_look_at(), Vec3::Z);
        }
    }

    if mouse_scroll_input.delta != Vec2::ZERO {
        let mut rel_pos = camera_transform.translation - space_camera.get_look_at();
        rel_pos *= (ZOOM_SPEED + 1.0).powf(-mouse_scroll_input.delta.y);
        camera_transform.translation = rel_pos + space_camera.get_look_at();
    }
}
