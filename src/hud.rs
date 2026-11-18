use std::f32::consts::E;

use bevy::prelude::*;
use strum::{EnumCount, VariantNames};

use crate::{
    gravity::{GRAVITY_CONSTANT, SimulationSpeed, SimulationState},
    grid::{GridMaterial, GridMode},
    trajectory::TrajectoryMode,
    ui::*,
};

pub const TEXT_STYLE_PRESET_1: TextStyle = TextStyle(UiColorStyle {
    disabled: Color::srgb(0.5, 0.5, 0.5),
    normal: Color::srgb(1.0, 1.0, 1.0),
    hovered: Color::srgb(1.0, 1.0, 1.0),
    pressed: Color::srgb(1.0, 1.0, 1.0),
});
pub const TEXT_STYLE_PRESET_2: TextStyle = TextStyle(UiColorStyle {
    disabled: Color::srgb(0.0, 0.3, 0.0),
    normal: Color::srgb(0.0, 1.0, 0.0),
    hovered: Color::srgb(0.0, 0.7, 0.0),
    pressed: Color::srgb(0.0, 0.5, 0.0),
});
pub const BUTTON_STYLE_PRESET_1: ButtonStyle = ButtonStyle {
    background: UiColorStyle {
        disabled: Color::srgb(0.3, 0.3, 0.3),
        normal: Color::srgb(0.1, 0.2, 0.3),
        hovered: Color::srgb(0.1, 0.2, 0.3),
        pressed: Color::srgb(0.7, 0.8, 0.9),
    },
    text: TEXT_STYLE_PRESET_2.0,
};
pub const SLIDER_STYLE_PRESET_1: SliderStyle = SliderStyle {
    thumb: UiColorStyle {
        disabled: Color::srgb(0.5, 0.5, 0.5),
        normal: Color::srgb(0.1, 0.2, 0.3),
        hovered: Color::srgb(0.7, 0.8, 0.9),
        pressed: Color::srgb(0.7, 0.8, 0.9),
    },
    track: UiColorStyle {
        disabled: Color::srgb(0.5, 0.5, 0.5),
        normal: Color::srgb(0.1, 0.1, 0.1),
        hovered: Color::srgb(0.1, 0.1, 0.1),
        pressed: Color::srgb(0.1, 0.1, 0.1),
    },
};

const SIMULATION_STATE_TEXT_ID: &str = "TEXT_SimulationState";
const SIMULATION_STATE_BUTTON_ID: &str = "BUTTON_SimulationState";
const SIMULATION_SPEED_TEXT_ID: &str = "TEXT_SimulationSpeed";
const SIMULATION_SPEED_SLIDER_ID: &str = "SLIDER_SimulationSpeed";
const TRAJECTORY_MODE_TEXT_ID: &str = "TEXT_TrajectoryMode";
const TRAJECTORY_MODE_BUTTON_ID: &str = "BUTTON_TrajectoryMode";
const GRID_MODE_TEXT_ID: &str = "TEXT_GridMode";
const GRID_MODE_BUTTON_ID: &str = "BUTTON_GridMode";
const LAGRANGE_THRESHOLD_TEXT_ID: &str = "TEXT_LagrangeThreshold";
const LAGRANGE_THRESHOLD_SLIDER_ID: &str = "SLIDER_LagrangeThreshold";

const SIMULATION_SPEED_MIN: f32 = 0.1;
const SIMULATION_SPEED_MAX: f32 = 50.0;
const LAGRANGE_THRESHOLD_MIN: f32 = 0.000_000_001;
const LAGRANGE_THRESHOLD_MAX: f32 = 0.000_001;

pub fn startup_hud(
    mut commands: Commands,
    mut text_table: ResMut<TextTable>,
    simulation_state: Res<State<SimulationState>>,
    mut next_simulation_state: ResMut<NextState<SimulationState>>,
    mut trajectory_mode: ResMut<TrajectoryMode>,
    materials: ResMut<Assets<GridMaterial>>,
) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::FlexStart,
            ..container_node(FlexDirection::Row)
        },
        children![
            (
                Node {
                    width: percent(20.0),
                    height: percent(50.0),
                    padding: UiRect::all(px(5)),
                    border_radius: BorderRadius::all(px(5)),
                    row_gap: px(5),
                    ..container_node(FlexDirection::Column)
                },
                BackgroundColor(Color::srgba(0.4, 0.5, 0.6, 0.7)),
                children![
                    (
                        container_node(FlexDirection::Row),
                        children![
                            text_bundle(SIMULATION_STATE_TEXT_ID, TEXT_STYLE_PRESET_1),
                            button_bundle(SIMULATION_STATE_BUTTON_ID, &BUTTON_STYLE_PRESET_1),
                        ],
                    ),
                    (
                        container_node(FlexDirection::Row),
                        children![
                            text_bundle(SIMULATION_SPEED_TEXT_ID, TEXT_STYLE_PRESET_1),
                            slider_bundle(SIMULATION_SPEED_SLIDER_ID, &SLIDER_STYLE_PRESET_1),
                        ],
                    ),
                    (
                        container_node(FlexDirection::Row),
                        children![
                            text_bundle(TRAJECTORY_MODE_TEXT_ID, TEXT_STYLE_PRESET_1),
                            button_bundle(TRAJECTORY_MODE_BUTTON_ID, &BUTTON_STYLE_PRESET_1),
                        ],
                    ),
                    (
                        container_node(FlexDirection::Row),
                        children![
                            text_bundle(GRID_MODE_TEXT_ID, TEXT_STYLE_PRESET_1),
                            button_bundle(GRID_MODE_BUTTON_ID, &BUTTON_STYLE_PRESET_1),
                        ],
                    ),
                    (
                        container_node(FlexDirection::Row),
                        children![
                            text_bundle(LAGRANGE_THRESHOLD_TEXT_ID, TEXT_STYLE_PRESET_1),
                            slider_bundle(LAGRANGE_THRESHOLD_SLIDER_ID, &SLIDER_STYLE_PRESET_1),
                        ],
                    ),
                ],
            ),
        ],
    ));

    commands.add_observer(button_observer);
    commands.add_observer(slider_observer);

    set_text(SIMULATION_STATE_TEXT_ID, "Sim state: ".into(), &mut text_table);
    set_text(SIMULATION_SPEED_TEXT_ID, "Sim speed: ".into(), &mut text_table);
    set_text(TRAJECTORY_MODE_TEXT_ID, "Traj mode: ".into(), &mut text_table);
    set_text(GRID_MODE_TEXT_ID, "Grid mode: ".into(), &mut text_table);
    set_text(LAGRANGE_THRESHOLD_TEXT_ID, "Threshold: ".into(), &mut text_table);

    set_next_simulation_state(true, &mut text_table, &simulation_state, &mut next_simulation_state);
    set_next_trajectory_mode(true, &mut text_table, &mut trajectory_mode);
    set_next_grid_mode(true, &mut text_table, materials);
}

fn button_observer(
    button_event: On<ButtonEvent>,
    mut text_table: ResMut<TextTable>,
    simulation_state: Res<State<SimulationState>>,
    mut next_simulation_state: ResMut<NextState<SimulationState>>,
    mut trajectory_mode: ResMut<TrajectoryMode>,
    materials: ResMut<Assets<GridMaterial>>,
) {
    let button_event = button_event.event();
    if button_event.1 {
        return;
    }

    match button_event.0 {
        ButtonId(SIMULATION_STATE_BUTTON_ID) => {
            set_next_simulation_state(false, &mut text_table, &simulation_state, &mut next_simulation_state);
        }
        ButtonId(TRAJECTORY_MODE_BUTTON_ID) => {
            set_next_trajectory_mode(false, &mut text_table, &mut trajectory_mode);
        }
        ButtonId(GRID_MODE_BUTTON_ID) => {
            set_next_grid_mode(false, &mut text_table, materials);
        }
        _ => ()
    }
}

fn slider_observer(
    slider_event: On<SliderEvent>,
    mut simulation_speed: ResMut<SimulationSpeed>,
    mut materials: ResMut<Assets<GridMaterial>>,
) {
    let slider_event = slider_event.event();

    match slider_event {
        SliderEvent(SliderId(SIMULATION_SPEED_SLIDER_ID), value) => {
            let min = SIMULATION_SPEED_MIN;
            let max = SIMULATION_SPEED_MAX;
            let value = *value;
            simulation_speed.0 = if value < 0.5 {
                (2.0 * value).powf(1.0 / E) * (1.0 - min) + min
            } else {
                (2.0 * value - 1.0).powf(E) * (max - 1.0) + 1.0
            };
        }
        SliderEvent(SliderId(LAGRANGE_THRESHOLD_SLIDER_ID), value) => {
            let min = GRAVITY_CONSTANT * LAGRANGE_THRESHOLD_MIN;
            let max = GRAVITY_CONSTANT * LAGRANGE_THRESHOLD_MAX;
            let lagrange_threshold = value.powf(E) * (max - min) + min;

            let mat = materials.iter_mut().next().unwrap().1;
            mat.gravity_src_buffer.lagrange_threshold = lagrange_threshold;
        }
        _ => ()
    }
}

fn set_next_simulation_state(
    reset: bool,
    text_table: &mut ResMut<TextTable>,
    simulation_state: &Res<State<SimulationState>>,
    next_simulation_state: &mut ResMut<NextState<SimulationState>>,
) {
    let mut simulation_state = *simulation_state.get();
    simulation_state = if reset {
        default()
    } else {
        SimulationState::from_repr((simulation_state as usize + 1) % SimulationState::COUNT).unwrap()
    };
    next_simulation_state.set(simulation_state);

    set_text(SIMULATION_STATE_BUTTON_ID, SimulationState::VARIANTS[simulation_state as usize].into(), text_table);
}

fn set_next_trajectory_mode(
    reset: bool,
    text_table: &mut ResMut<TextTable>,
    trajectory_mode: &mut ResMut<TrajectoryMode>,
) {
    **trajectory_mode = if reset {
        default()
    } else {
        TrajectoryMode::from_repr((**trajectory_mode as usize + 1) % TrajectoryMode::COUNT).unwrap()
    };

    set_text(TRAJECTORY_MODE_BUTTON_ID, TrajectoryMode::VARIANTS[**trajectory_mode as usize].into(), text_table);
}

fn set_next_grid_mode(
    reset: bool,
    text_table: &mut ResMut<TextTable>,
    mut materials: ResMut<Assets<GridMaterial>>,
) {
    let mat = materials.iter_mut().next().unwrap().1;
    mat.grid_mode = if reset {
        GridMode::default() as u32
    } else {
        (mat.grid_mode + 1) % GridMode::COUNT as u32
    };

    set_text(GRID_MODE_BUTTON_ID, GridMode::VARIANTS[mat.grid_mode as usize].into(), text_table);
}
