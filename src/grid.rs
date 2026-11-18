use std::f32::consts::PI;

use bevy::{
    math::*,
    mesh::MeshVertexBufferLayoutRef,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};
use encase::ShaderType;
use strum::{EnumCount, EnumDiscriminants, FromRepr, VariantNames};

use crate::{
    app::{AppSet, StartupSet},
    camera::SpaceCamera,
    simulation::{CentralBody, GRAVITY_CONSTANT, Mass, SimulationState, Velocity},
};

#[derive(
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
#[allow(unused)]
pub enum GridMode {
    #[default]
    Off,
    Space,
    Potential,
    Acceleration,
    Lagrange,
}

#[derive(Clone, ShaderType, Default)]
struct CameraSettings {
    pos: Vec3,
    look_at: Vec3,
}

#[derive(Clone, ShaderType, Default)]
struct GravitySrc {
    pos: Vec3,
    vel: Vec3,
    mass: f32,
    index: u32,
    central_body_index: u32,
}

#[derive(Clone, ShaderType, Default)]
pub struct GravitySrcBuffer {
    num: u32,
    buffer: [GravitySrc; 32],
    max_mass: f32,
    max_mass_radius: f32,
    gravity_constant: f32,
    pub lagrange_threshold: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Default)]
pub struct GridMaterial {
    #[uniform(0)]
    pub grid_mode: u32,
    #[uniform(1)]
    camera_settings: CameraSettings,
    #[uniform(2)]
    pub gravity_src_buffer: GravitySrcBuffer,
}

impl Material for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/grid.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayoutRef,
        _: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;

        Ok(())
    }
}

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<GridMaterial>::default())
            .add_systems(Startup, startup_grid.in_set(StartupSet::Scene))
            .add_systems(
                Update,
                update_grid
                    .run_if(in_state(SimulationState::Running))
                    .in_set(AppSet::View),
            );
    }
}

fn startup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GridMaterial>>,
) {
    commands.spawn((
        Transform::from_translation(Vec3::ZERO)
            .with_rotation(Quat::from_axis_angle(Vec3::X, PI / 2.0))
            .with_scale(Vec3::splat(1_000_000.0)),
        Mesh3d(meshes.add(Plane3d::default())),
        MeshMaterial3d(materials.add(GridMaterial::default())),
    ));
}

fn update_grid(
    camera: Single<(&Transform, &SpaceCamera)>,
    gravity_src: Query<(Entity, &Transform, Option<&Velocity>, &Mass, &CentralBody)>,
    mut materials: ResMut<Assets<GridMaterial>>,
) {
    let mat = materials.iter_mut().next().unwrap().1;

    let (camera_transform, space_camera) = camera.into_inner();
    mat.camera_settings.pos = camera_transform.translation;
    mat.camera_settings.look_at = *space_camera.get_look_at();

    mat.gravity_src_buffer.num = 0;

    let index = &mut mat.gravity_src_buffer.num;
    for (entity, transform, velocity, mass, central_body) in gravity_src {
        mat.gravity_src_buffer.buffer[*index as usize] = GravitySrc {
            pos: transform.translation,
            vel: velocity.map(|velocity| velocity.0).unwrap_or_default(),
            mass: mass.0,
            index: entity.index_u32(),
            central_body_index: central_body.0,
        };
        *index += 1;

        if mat.gravity_src_buffer.max_mass < mass.0 {
            mat.gravity_src_buffer.max_mass = mass.0;
            mat.gravity_src_buffer.max_mass_radius = transform.scale.x / 2.0;
        }
    }

    mat.gravity_src_buffer.gravity_constant = GRAVITY_CONSTANT;
}
