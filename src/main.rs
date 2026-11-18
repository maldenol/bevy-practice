mod app;
mod camera;
mod grid;
mod simulation;
mod ui;

use bevy::prelude::*;

use app::AppPlugin;

#[bevy_main]
fn main() {
    App::new().add_plugins((DefaultPlugins, AppPlugin)).run();
}
