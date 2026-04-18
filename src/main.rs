mod camera;
mod gravity;
mod grid;
mod hud;
mod input;
mod planets;
mod trajectory;
mod ui;

use bevy::{
    prelude::*,
    ui_widgets::SliderPlugin,
};

use camera::*;
use gravity::*;
use grid::*;
use hud::*;
use input::*;
use planets::*;
use trajectory::*;
use ui::*;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MaterialPlugin::<GridMaterial>::default(),
            SliderPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .init_state::<SimulationState>()
        .add_systems(Startup, (
            startup_ui,
            startup_camera,
            startup_gravity,
            startup_grid,
            startup_trajectory,
            startup_hud,
            spawn_planets,
        ).chain())
        .add_systems(PreUpdate, (
            handle_keyboard_input,
            handle_mouse_input,
        ).chain())
        .add_systems(FixedUpdate, (
            update_velocity.run_if(in_state(SimulationState::Running)),
            update_position.run_if(in_state(SimulationState::Running)),
        ).chain())
        .add_systems(Update, (
            update_central_body.run_if(in_state(SimulationState::Running)),
            update_camera,
            update_grid.run_if(in_state(SimulationState::Running)),
            draw_trajectory,
            (update_ui_buttons, update_ui_sliders, update_ui_text),
        ).chain())
        .run();
}
