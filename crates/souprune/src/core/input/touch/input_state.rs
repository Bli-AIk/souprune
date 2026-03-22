use super::{
    ControllerDirections, MultitouchPressed, PrevTouchPressed, TouchAction, TouchControllerZone,
    TouchOverlayEnabled,
};
use crate::core::input::actions::{Action, ActionRegistry};
use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::ui::{RelativeCursorPosition, UiGlobalTransform};
use bevy::window::PrimaryWindow;
use leafwing_input_manager::action_state::ActionKindData;
use leafwing_input_manager::buttonlike::ButtonState;
use leafwing_input_manager::prelude::ActionState;
use std::collections::HashSet;

const CONTROLLER_DEADZONE: f32 = 0.1;

fn insert_controller_dirs(dirs: &mut HashSet<String>, pos: Vec2) {
    if pos.y < -CONTROLLER_DEADZONE {
        dirs.insert("Up".to_string());
    }
    if pos.y > CONTROLLER_DEADZONE {
        dirs.insert("Down".to_string());
    }
    if pos.x < -CONTROLLER_DEADZONE {
        dirs.insert("Left".to_string());
    }
    if pos.x > CONTROLLER_DEADZONE {
        dirs.insert("Right".to_string());
    }
}

pub(super) fn detect_multitouch_pressed(
    touches: Res<Touches>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<&Camera, With<Camera2d>>,
    buttons: Query<(&ComputedNode, &UiGlobalTransform, &TouchAction), Without<TouchControllerZone>>,
    zones: Query<(&ComputedNode, &UiGlobalTransform), With<TouchControllerZone>>,
    mut multitouch: ResMut<MultitouchPressed>,
) {
    multitouch.0.clear();

    let Ok(window) = windows.single() else {
        return;
    };
    let sf = window.scale_factor();

    let vp_offset = cameras
        .iter()
        .next()
        .and_then(|cam| cam.physical_viewport_rect())
        .map(|rect| rect.min.as_vec2())
        .unwrap_or(Vec2::ZERO);

    for touch in touches.iter() {
        let pos = touch.position() * sf - vp_offset;

        for (node, transform, action) in buttons.iter() {
            if node.contains_point(*transform, pos) {
                multitouch.0.insert(action.0.clone());
            }
        }

        for (node, transform) in zones.iter() {
            if node.contains_point(*transform, pos)
                && let Some(normalized) = node.normalize_point(*transform, pos)
            {
                insert_controller_dirs(&mut multitouch.0, normalized);
            }
        }
    }
}

/// Determines active direction actions from the controller touch zone position.
/// Supports both single-pointer (Interaction) and multitouch (Touches).
pub fn update_controller_directions(
    touches: Res<Touches>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<&Camera, With<Camera2d>>,
    zones: Query<
        (
            &Interaction,
            &ComputedNode,
            &UiGlobalTransform,
            &RelativeCursorPosition,
        ),
        With<TouchControllerZone>,
    >,
    mut dirs: ResMut<ControllerDirections>,
) {
    dirs.0.clear();

    let has_touches = touches.iter().next().is_some();

    if !has_touches {
        for (interaction, _, _, rel_pos) in zones.iter() {
            if *interaction != Interaction::Pressed {
                continue;
            }
            if let Some(pos) = rel_pos.normalized {
                insert_controller_dirs(&mut dirs.0, pos);
            }
        }
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let sf = window.scale_factor();
    let vp_offset = cameras
        .iter()
        .next()
        .and_then(|cam| cam.physical_viewport_rect())
        .map(|rect| rect.min.as_vec2())
        .unwrap_or(Vec2::ZERO);
    for touch in touches.iter() {
        let pos = touch.position() * sf - vp_offset;
        for (_, node, transform, _) in zones.iter() {
            let Some(normalized) = node
                .contains_point(*transform, pos)
                .then(|| node.normalize_point(*transform, pos))
                .flatten()
            else {
                continue;
            };
            insert_controller_dirs(&mut dirs.0, normalized);
        }
    }
}

pub(super) fn inject_touch_actions(
    enabled: Res<TouchOverlayEnabled>,
    registry: Res<ActionRegistry>,
    touches: Res<Touches>,
    multitouch: Res<MultitouchPressed>,
    buttons: Query<(&Interaction, &TouchAction)>,
    controller_dirs: Res<ControllerDirections>,
    mut action_states: Query<&mut ActionState<Action>>,
    mut prev: ResMut<PrevTouchPressed>,
) {
    if !enabled.0 {
        return;
    }

    let mut currently_pressed = HashSet::new();
    let has_touches = touches.iter().next().is_some();

    if !has_touches {
        for (interaction, touch_action) in buttons.iter() {
            if *interaction == Interaction::Pressed {
                currently_pressed.insert(touch_action.0.clone());
            }
        }
    }

    for action_name in &multitouch.0 {
        currently_pressed.insert(action_name.clone());
    }

    for dir in controller_dirs.0.iter() {
        currently_pressed.insert(dir.clone());
    }

    for mut state in action_states.iter_mut() {
        for name in &currently_pressed {
            let Some(slot) = registry.get(name) else {
                continue;
            };
            let was_pressed = prev.0.contains(name);
            let target_state = if was_pressed {
                ButtonState::Pressed
            } else {
                ButtonState::JustPressed
            };
            set_button_state(&mut state, &slot, target_state);
        }

        for name in &prev.0 {
            if !currently_pressed.contains(name)
                && let Some(slot) = registry.get(name)
            {
                set_button_state(&mut state, &slot, ButtonState::JustReleased);
            }
        }
    }

    prev.0 = currently_pressed;
}

fn set_button_state(state: &mut ActionState<Action>, action: &Action, target: ButtonState) {
    let data = state.action_data_mut_or_default(action);
    if let ActionKindData::Button(ref mut btn) = data.kind_data {
        btn.state = target;
        btn.update_state = target;
        btn.value = if target.pressed() { 1.0 } else { 0.0 };
        btn.update_value = btn.value;
    }
}
