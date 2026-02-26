//! # touch.rs
//!
//! ## Module Overview / 模块概述
//!
//! Virtual touch overlay for mobile/touch platforms.
//! Renders on-screen D-pad and action buttons, translates Bevy UI interactions
//! into leafwing_input_manager actions via `InputManagerSystem::ManualControl`.
//!
//! 移动/触屏平台的虚拟触控覆盖层。
//! 渲染屏幕上的方向键和动作按钮，通过 `InputManagerSystem::ManualControl`
//! 将 Bevy UI 交互转换为 leafwing_input_manager 动作。
//!
//! ## Press/Hold/Release 处理
//!
//! leafwing 的 Update 系统会在物理按键未按下时调用 release()，
//! 导致 ManualControl 中再调用 press() 会每帧触发 JustPressed。
//! 因此我们自行追踪上一帧的触控状态，直接设置 ButtonState，
//! 绕过 press()/release() 的状态机问题。

use super::actions::{Action, ActionRegistry};
use bevy::prelude::*;
use std::collections::HashSet;
use leafwing_input_manager::action_state::{ActionKindData, ButtonData};
use leafwing_input_manager::buttonlike::ButtonState;
use leafwing_input_manager::plugin::InputManagerSystem;
use leafwing_input_manager::prelude::ActionState;

/// Marker component for the touch overlay root entity.
#[derive(Component)]
pub struct TouchOverlayRoot;

/// Component linking a UI button to a game action name.
#[derive(Component)]
pub struct TouchAction(pub String);

/// Resource controlling touch overlay visibility.
#[derive(Resource)]
pub struct TouchOverlayEnabled(pub bool);

impl Default for TouchOverlayEnabled {
    fn default() -> Self {
        // Enabled by default — mouse clicks work as touch input on desktop
        Self(true)
    }
}

/// Tracks which touch actions were pressed in the previous frame.
#[derive(Resource, Default)]
struct PrevTouchPressed(HashSet<String>);

pub(crate) struct TouchPlugin;

impl Plugin for TouchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TouchOverlayEnabled>()
            .init_resource::<PrevTouchPressed>()
            .add_systems(
                PreUpdate,
                inject_touch_actions
                    .in_set(InputManagerSystem::ManualControl)
                    .run_if(resource_exists::<ActionRegistry>),
            );
    }
}

const BTN_SIZE: f32 = 56.0;
const DPAD_BTN: f32 = 52.0;
const MARGIN: f32 = 8.0;
const OVERLAY_OPACITY: f32 = 0.45;
const BTN_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, OVERLAY_OPACITY);
const BTN_PRESSED_COLOR: Color = Color::srgba(0.7, 0.9, 1.0, 0.7);

/// Spawn the touch overlay UI. Call once after ActionRegistry is ready.
pub fn spawn_touch_overlay(commands: &mut Commands, registry: &ActionRegistry) {
    info!("Spawning touch overlay UI");

    // Root container: full-screen, no background, pass-through for non-button areas
    let root = commands
        .spawn((
            TouchOverlayRoot,
            Name::new("TouchOverlay"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            // Transparent background
            BackgroundColor(Color::NONE),
            // High z-index to render above game content
            GlobalZIndex(1000),
            Pickable::IGNORE,
        ))
        .id();

    // ── Left side: D-pad ──
    let dpad_container = commands
        .spawn((
            Name::new("DPadContainer"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(24.0),
                bottom: Val::Px(24.0),
                width: Val::Px(DPAD_BTN * 3.0 + MARGIN * 2.0),
                height: Val::Px(DPAD_BTN * 3.0 + MARGIN * 2.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();

    // D-pad buttons: positioned within the 3x3 grid
    let dpad_buttons = [
        ("Up", 1, 0),    // top-center
        ("Down", 1, 2),  // bottom-center
        ("Left", 0, 1),  // center-left
        ("Right", 2, 1), // center-right
    ];

    let dpad_labels = ["▲", "▼", "◀", "▶"];

    for (i, (action_name, col, row)) in dpad_buttons.iter().enumerate() {
        if registry.get(action_name).is_none() {
            continue;
        }
        let x = *col as f32 * (DPAD_BTN + MARGIN);
        let y = *row as f32 * (DPAD_BTN + MARGIN);

        let btn = spawn_button(
            commands,
            action_name,
            dpad_labels[i],
            DPAD_BTN,
            Val::Px(x),
            Val::Px(y),
        );
        commands.entity(dpad_container).add_child(btn);
    }

    commands.entity(root).add_child(dpad_container);

    // ── Right side: Action buttons ──
    let action_container = commands
        .spawn((
            Name::new("ActionButtonContainer"),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(24.0),
                bottom: Val::Px(24.0),
                width: Val::Px(BTN_SIZE * 3.0 + MARGIN * 2.0),
                height: Val::Px(BTN_SIZE * 3.0 + MARGIN * 2.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();

    // Action buttons in a diamond-ish layout
    let action_buttons: &[(&str, &str, f32, f32)] = &[
        ("Confirm", "Z", 1.0, 2.0), // bottom-center
        ("Cancel", "X", 2.0, 1.0),  // center-right
        ("Menu", "C", 0.0, 1.0),    // center-left
    ];

    for (action_name, label, col, row) in action_buttons {
        if registry.get(action_name).is_none() {
            continue;
        }
        let x = *col * (BTN_SIZE + MARGIN);
        let y = *row * (BTN_SIZE + MARGIN);

        let btn = spawn_button(
            commands,
            action_name,
            label,
            BTN_SIZE,
            Val::Px(x),
            Val::Px(y),
        );
        commands.entity(action_container).add_child(btn);
    }

    commands.entity(root).add_child(action_container);
}

fn spawn_button(
    commands: &mut Commands,
    action_name: &str,
    label: &str,
    size: f32,
    left: Val,
    top: Val,
) -> Entity {
    let btn = commands
        .spawn((
            Name::new(format!("TouchBtn_{}", action_name)),
            TouchAction(action_name.to_string()),
            Button,
            Node {
                position_type: PositionType::Absolute,
                left,
                top,
                width: Val::Px(size),
                height: Val::Px(size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.6)),
            BackgroundColor(BTN_COLOR),
        ))
        .id();

    let text = commands
        .spawn((
            Text::new(label.to_string()),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Pickable::IGNORE,
        ))
        .id();

    commands.entity(btn).add_child(text);
    btn
}

/// Reads Bevy UI Interaction on touch buttons and injects corresponding
/// actions into all entities carrying ActionState<Action>.
/// Scheduled in PreUpdate/ManualControl, after leafwing's own input processing.
///
/// 直接设置 ButtonState 而非调用 press()/release()，
/// 因为 leafwing Update 阶段会对无物理按键的动作调用 release()，
/// 导致 ManualControl 中 press() 每帧产生 JustPressed。
fn inject_touch_actions(
    enabled: Res<TouchOverlayEnabled>,
    registry: Res<ActionRegistry>,
    buttons: Query<(&Interaction, &TouchAction)>,
    mut action_states: Query<&mut ActionState<Action>>,
    mut prev: ResMut<PrevTouchPressed>,
) {
    if !enabled.0 {
        return;
    }

    // Collect currently pressed touch actions
    let mut currently_pressed = HashSet::new();
    for (interaction, touch_action) in buttons.iter() {
        if *interaction == Interaction::Pressed {
            currently_pressed.insert(touch_action.0.clone());
        }
    }

    // Determine transitions and set ButtonState directly
    for mut state in action_states.iter_mut() {
        // Handle newly pressed and held buttons
        for name in &currently_pressed {
            if let Some(slot) = registry.get(name) {
                let was_pressed = prev.0.contains(name);
                let target_state = if was_pressed {
                    ButtonState::Pressed
                } else {
                    ButtonState::JustPressed
                };
                set_button_state(&mut state, &slot, target_state);
            }
        }

        // Handle just-released buttons (were pressed last frame, not pressed now)
        for name in &prev.0 {
            if !currently_pressed.contains(name) {
                if let Some(slot) = registry.get(name) {
                    set_button_state(&mut state, &slot, ButtonState::JustReleased);
                }
            }
        }
    }

    prev.0 = currently_pressed;
}

/// Directly set the ButtonState for an action, bypassing press()/release()
/// state machine which conflicts with leafwing's Update-phase release() calls.
fn set_button_state(state: &mut ActionState<Action>, action: &Action, target: ButtonState) {
    let data = state.action_data_mut_or_default(action);
    if let ActionKindData::Button(ref mut btn) = data.kind_data {
        btn.state = target;
        btn.update_state = target;
        btn.value = if target.pressed() { 1.0 } else { 0.0 };
        btn.update_value = btn.value;
    }
}

/// Update button visuals based on interaction state.
pub fn update_touch_button_visuals(
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<TouchAction>),
    >,
) {
    for (interaction, mut bg) in buttons.iter_mut() {
        *bg = match interaction {
            Interaction::Pressed => BackgroundColor(BTN_PRESSED_COLOR),
            _ => BackgroundColor(BTN_COLOR),
        };
    }
}

/// Toggle touch overlay visibility.
pub fn toggle_touch_overlay(
    mut enabled: ResMut<TouchOverlayEnabled>,
    mut overlays: Query<&mut Visibility, With<TouchOverlayRoot>>,
) {
    enabled.0 = !enabled.0;
    let vis = if enabled.0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut v in overlays.iter_mut() {
        *v = vis;
    }
}
