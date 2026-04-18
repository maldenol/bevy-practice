use bevy::{
    math::*,
    prelude::*,
};

use crate::gravity::Velocity;

#[derive(Component)]
pub struct SpaceCamera {
    look_at: Vec3,
    look_at_entity: Option<Entity>,
    try_look_at_entity: Option<Entity>,
}

impl Default for SpaceCamera {
    fn default() -> Self {
        SpaceCamera {
            look_at: Vec3::ZERO,
            look_at_entity: None,
            try_look_at_entity: None,
        }
    }
}

impl SpaceCamera {
    pub fn try_look_at(&mut self, entity: Entity) {
        self.try_look_at_entity = Some(entity);
    }

    pub fn get_look_at(&self) -> &Vec3 {
        &self.look_at
    }

    pub fn get_look_at_mut(&mut self) -> &mut Vec3 {
        self.look_at_entity = None;

        &mut self.look_at
    }

    pub fn get_look_at_entity(&self) -> &Option<Entity> {
        &self.look_at_entity
    }
}

pub fn startup_camera(
    mut commands: Commands,
) {
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 300.0, 300.0)).looking_at(Vec3::ZERO, Vec3::Z),
        Camera3d::default(),
        SpaceCamera::default(),
    ));
}

pub fn update_camera(
    camera: Single<(&mut Transform, &mut SpaceCamera)>,
    other_entities: Query<(Entity, &Transform), (Without<SpaceCamera>, With<Velocity>)>,
) {
    let (mut camera_transform, mut space_camera) = camera.into_inner();

    if let Some(try_look_at_entity) = space_camera.try_look_at_entity {
        if let Ok((look_at_entity, _)) = other_entities.get(try_look_at_entity) {
            space_camera.look_at_entity = Some(look_at_entity);
        }

        space_camera.try_look_at_entity = None;
    }

    let Some(look_at_entity) = space_camera.look_at_entity else {
        return;
    };

    let Ok((_, look_at_entity_transform)) = other_entities.get(look_at_entity) else {
        space_camera.look_at_entity = None;
        return;
    };

    let offset = look_at_entity_transform.translation - space_camera.look_at;
    camera_transform.translation += offset;
    space_camera.look_at += offset;
    camera_transform.look_at(space_camera.look_at, Vec3::Z);
}
