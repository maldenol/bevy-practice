use bevy::{
    picking::hover::Hovered,
    platform::collections::HashMap,
    prelude::*,
    ui_widgets::{
        CoreSliderDragState, Slider, SliderPlugin, SliderRange, SliderThumb, SliderValue,
        TrackClick, observe, slider_self_update,
    },
};

use crate::app::AppSet;

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextId(&'static str);

#[derive(Resource, Default)]
pub struct TextTable(HashMap<TextId, String>);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct ButtonId(pub &'static str);

#[derive(Component, Default)]
pub struct ButtonPressState(pub bool);

#[derive(Event)]
pub struct ButtonEvent(pub ButtonId, pub bool);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct SliderId(pub &'static str);

#[derive(Event)]
pub struct SliderEvent(pub SliderId, pub f32);

#[derive(Clone, Copy)]
pub struct UiColorStyle {
    pub disabled: Color,
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}

#[derive(Component, Clone, Copy)]
pub struct TextStyle(pub UiColorStyle);

#[derive(Component, Clone, Copy)]
pub struct ButtonStyle {
    pub background: UiColorStyle,
    pub text: UiColorStyle,
}

#[derive(Component, Clone, Copy)]
pub struct SliderStyle {
    pub thumb: UiColorStyle,
    pub track: UiColorStyle,
}

pub fn container_node(flex_direction: FlexDirection) -> Node {
    Node {
        flex_direction,
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::Center,
        ..default()
    }
}

pub fn text_bundle(id: &'static str, text_style: TextStyle) -> impl Bundle {
    (
        TextId(id),
        Text::default(),
        TextColor(text_style.0.normal),
        text_style,
    )
}

pub fn button_bundle(id: &'static str, button_style: &ButtonStyle) -> impl Bundle {
    (
        Node {
            width: px(150),
            height: px(30),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(px(5)),
            border_radius: BorderRadius::all(px(5)),
            ..container_node(FlexDirection::Row)
        },
        ButtonId(id),
        ButtonPressState::default(),
        Button,
        BackgroundColor(button_style.background.normal),
        *button_style,
        children![text_bundle(id, TextStyle(button_style.text)),],
    )
}

pub fn slider_bundle(id: &'static str, slider_style: &SliderStyle) -> impl Bundle {
    (
        Node {
            width: px(150),
            height: px(30),
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
            padding: UiRect::all(px(5)),
            border_radius: BorderRadius::all(px(5)),
            ..container_node(FlexDirection::Column)
        },
        SliderId(id),
        Slider {
            track_click: TrackClick::Snap,
        },
        SliderValue(0.5),
        SliderRange::new(0.0, 1.0),
        Hovered::default(),
        *slider_style,
        observe(slider_self_update),
        children![
            (
                Node {
                    height: px(6),
                    border_radius: BorderRadius::all(px(5)),
                    ..default()
                },
                BackgroundColor(slider_style.track.normal),
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(12),
                    top: px(0),
                    bottom: px(0),
                    display: Display::Flex,
                    ..default()
                },
                children![(
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(0),
                        top: px(9),
                        width: px(12),
                        height: px(12),
                        display: Display::Flex,
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    SliderThumb,
                    BackgroundColor(slider_style.thumb.normal),
                ),],
            ),
        ],
    )
}

pub fn set_text(id: &'static str, text: String, text_table: &mut ResMut<TextTable>) {
    text_table.0.entry(TextId(id)).insert(text);
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SliderPlugin)
            .init_resource::<TextTable>()
            .add_systems(
                Update,
                (update_ui_buttons, update_ui_sliders, update_ui_text)
                    .chain()
                    .in_set(AppSet::Ui),
            );
    }
}

fn update_ui_buttons(
    mut commands: Commands,
    mut button_query: Query<
        (
            &Interaction,
            &ButtonId,
            &mut ButtonPressState,
            &mut Button,
            &mut BackgroundColor,
            &ButtonStyle,
            &Children,
        ),
        Changed<Interaction>,
    >,
    mut button_text_query: Query<(&mut TextColor,)>,
) {
    for (
        interaction,
        button_id,
        mut button_press_state,
        mut button,
        mut background_color,
        button_style,
        children,
    ) in &mut button_query
    {
        let (mut button_text_color,) = button_text_query.get_mut(children[0]).unwrap();
        (background_color.0, button_text_color.0) = match *interaction {
            Interaction::Pressed => (button_style.background.pressed, button_style.text.pressed),
            Interaction::Hovered => (button_style.background.hovered, button_style.text.hovered),
            Interaction::None => (button_style.background.normal, button_style.text.normal),
        };
        button.set_changed();

        let pressed = *interaction == Interaction::Pressed;
        let was_pressed = button_press_state.0;
        if pressed != was_pressed {
            button_press_state.0 = pressed;
            commands.trigger(ButtonEvent(*button_id, pressed));
        }
    }
}

fn update_ui_sliders(
    mut commands: Commands,
    slider_query: Query<
        (
            Entity,
            &SliderId,
            &SliderValue,
            &SliderRange,
            &CoreSliderDragState,
            &Hovered,
            &SliderStyle,
        ),
        (
            Or<(
                Changed<SliderValue>,
                Changed<CoreSliderDragState>,
                Changed<Hovered>,
            )>,
        ),
    >,
    children: Query<&Children>,
    mut slider_thumb_query: Query<(&mut Node, &mut BackgroundColor), With<SliderThumb>>,
) {
    for (
        entity,
        slider_id,
        slider_value,
        slider_range,
        slider_drag_state,
        slider_hovered,
        slider_style,
    ) in slider_query.iter()
    {
        for child in children.iter_descendants(entity) {
            if let Ok((mut slider_thumb_node, mut slider_thumb_color)) =
                slider_thumb_query.get_mut(child)
            {
                let pressed = slider_drag_state.dragging;
                slider_thumb_color.0 = if pressed {
                    slider_style.thumb.pressed
                } else {
                    if slider_hovered.0 {
                        slider_style.thumb.hovered
                    } else {
                        slider_style.thumb.normal
                    }
                };

                slider_thumb_node.left =
                    percent(slider_range.thumb_position(slider_value.0) * 100.0);

                commands.trigger(SliderEvent(*slider_id, slider_value.0));
            }
        }
    }
}

fn update_ui_text(mut text_query: Query<(&TextId, &mut Text)>, mut text_table: ResMut<TextTable>) {
    if text_table.0.is_empty() {
        return;
    }

    for (text_id, mut text) in text_query.iter_mut() {
        if text_id.0.is_empty() {
            continue;
        }

        let Some(new_text) = text_table.0.remove(text_id) else {
            continue;
        };

        **text = new_text.into();
    }
}
