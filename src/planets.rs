use bevy::{
    camera::visibility::NoFrustumCulling,
    math::*,
    prelude::*,
};

use crate::gravity::{Velocity, Mass, CentralBody};

struct PlanetParams {
    translation: Vec3,
    scale: f32,
    velocity: Vec3,
    mass: f32,
    material: Handle<StandardMaterial>,
}

struct StarParams {
    light_range: f32,
    light_intensity: f32,
}

pub fn spawn_planets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Sphere::default());

    commands.spawn(star_bundle(
        &PlanetParams {
            translation: Vec3::ZERO,
            scale: 10.0,
            velocity: Vec3::ZERO,
            mass: 1_000.0,
            material: materials.add(StandardMaterial{
                base_color: Color::srgb(1.0, 1.0, 0.0),
                perceptual_roughness: 1.0,
                emissive: LinearRgba::rgb(100.0, 100.0, 0.0),
                ..default()
            }),
        },
        &StarParams {
            light_range: 1_000.0,
            light_intensity: 1_000_000_000.0,
        },
        mesh.clone(),
    ));

    commands.spawn(planet_bundle(
        &PlanetParams {
            translation: Vec3::new(100.0, 0.0, 0.0),
            scale: 2.0,
            velocity: Vec3::new(0.0, 30.0, 0.0),
            mass: 1.0,
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 0.0),
                perceptual_roughness: 1.0,
                ..default()
            }),
        },
        mesh.clone(),
    ));

    commands.spawn(planet_bundle(
        &PlanetParams {
            translation: Vec3::new(200.0, 0.0, 0.0),
            scale: 2.0,
            velocity: Vec3::new(0.0, 15.0, 0.0),
            mass: 1.0,
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 0.0, 1.0),
                perceptual_roughness: 1.0,
                ..default()
            }),
        },
        mesh.clone(),
    ));

    commands.spawn(planet_bundle(
        &PlanetParams {
            translation: Vec3::new(500.0, 0.0, 0.0),
            scale: 5.0,
            velocity: Vec3::new(0.0, 14.142, 0.0),
            mass: 10.0,
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 1.0, 0.0),
                perceptual_roughness: 1.0,
                ..default()
            }),
        },
        mesh.clone(),
    ));

    commands.spawn(planet_bundle(
        &PlanetParams {
            translation: Vec3::new(525.0, 0.0, 0.0),
            scale: 1.0,
            velocity: Vec3::new(0.0, 20.0, 0.0),
            mass: 0.1,
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 0.0),
                perceptual_roughness: 1.0,
                ..default()
            }),
        },
        mesh.clone(),
    ));
}

fn planet_bundle(
    planet_params: &PlanetParams,
    mesh: Handle<Mesh>,
) -> impl Bundle {
    (
        Transform::from_translation(planet_params.translation).with_scale(Vec3::splat(planet_params.scale)),
        Velocity(planet_params.velocity),
        Mass(planet_params.mass),
        CentralBody::default(),
        Mesh3d(mesh),
        MeshMaterial3d(planet_params.material.clone()),
    )
}

fn star_bundle(
    planet_params: &PlanetParams,
    star_params: &StarParams,
    mesh: Handle<Mesh>,
) -> impl Bundle {
    (
        planet_bundle(planet_params, mesh),
        PointLight{
            range: star_params.light_range,
            intensity: star_params.light_intensity,
            ..default()
        },
        NoFrustumCulling,
    )
}
