use bevy::prelude::*;

use crate::{
    camera::CameraPlugin,
    grid::GridPlugin,
    simulation::{GravityPlugin, PlanetsPlugin, TrajectoryPlugin},
    ui::{HudPlugin, UiPlugin},
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum StartupSet {
    Scene,
    Hud,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppSet {
    Simulation,
    Camera,
    View,
    Ui,
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .configure_sets(Startup, (StartupSet::Scene, StartupSet::Hud).chain())
            .configure_sets(
                Update,
                (AppSet::Simulation, AppSet::Camera, AppSet::View, AppSet::Ui).chain(),
            )
            .add_plugins((
                UiPlugin,
                GravityPlugin,
                PlanetsPlugin,
                CameraPlugin,
                GridPlugin,
                TrajectoryPlugin,
                HudPlugin,
            ));
    }
}
