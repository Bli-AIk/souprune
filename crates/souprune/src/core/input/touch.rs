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
use super::config::{TouchAnchor, TouchButtonDef, TouchLayoutDef};
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionKindData;
use leafwing_input_manager::buttonlike::ButtonState;
use leafwing_input_manager::plugin::InputManagerSystem;
use leafwing_input_manager::prelude::ActionState;
use std::collections::HashSet;

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

const FALLBACK_OPACITY: f32 = 0.45;
const FALLBACK_BTN_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, FALLBACK_OPACITY);
const BTN_PRESSED_COLOR: Color = Color::srgba(0.7, 0.9, 1.0, 0.7);

/// Stores the normal-state image handle for pressed/released visual swap.
#[derive(Component)]
pub struct TouchNormalImage(pub Option<Handle<Image>>);

/// Stores the pressed-state image handle for visual swap.
#[derive(Component)]
pub struct TouchPressedImage(pub Option<Handle<Image>>);

/// Spawn the touch overlay UI from a `TouchLayoutDef` config.
/// Falls back to simple colored rectangles if no textures are configured.
pub fn spawn_touch_overlay(
    commands: &mut Commands,
    registry: &ActionRegistry,
    asset_server: &AssetServer,
    layout: Option<&TouchLayoutDef>,
) {
    info!("Spawning touch overlay UI");

    let opacity = layout.map(|l| l.opacity).unwrap_or(FALLBACK_OPACITY);
    let scale = layout.map(|l| l.scale).unwrap_or(1.0);

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
            BackgroundColor(Color::NONE),
            GlobalZIndex(1000),
            Pickable::IGNORE,
        ))
        .id();

    if let Some(layout) = layout {
        // Config-driven: spawn buttons from layout definition
        for btn_def in &layout.buttons {
            if registry.get(&btn_def.action).is_none() {
                warn!(
                    "Touch button action '{}' not registered, skipping",
                    btn_def.action
                );
                continue;
            }
            let btn = spawn_config_button(commands, asset_server, btn_def, opacity, scale);
            commands.entity(root).add_child(btn);
        }
    } else {
        // Fallback: hardcoded layout (for mods without touch_layout.ron)
        spawn_fallback_layout(commands, registry, root, opacity, scale);
    }
}

/// Spawn a single button from config definition.
fn spawn_config_button(
    commands: &mut Commands,
    asset_server: &AssetServer,
    def: &TouchButtonDef,
    opacity: f32,
    scale: f32,
) -> Entity {
    let w = def.width * scale;
    let h = def.height * scale;

    // Position based on anchor
    let mut node = Node {
        position_type: PositionType::Absolute,
        width: Val::Px(w),
        height: Val::Px(h),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    match def.anchor {
        TouchAnchor::BottomLeft => {
            node.left = Val::Px(def.offset_x * scale);
            node.bottom = Val::Px(def.offset_y * scale);
        }
        TouchAnchor::BottomRight => {
            node.right = Val::Px(def.offset_x * scale);
            node.bottom = Val::Px(def.offset_y * scale);
        }
        TouchAnchor::TopLeft => {
            node.left = Val::Px(def.offset_x * scale);
            node.top = Val::Px(def.offset_y * scale);
        }
        TouchAnchor::TopRight => {
            node.right = Val::Px(def.offset_x * scale);
            node.top = Val::Px(def.offset_y * scale);
        }
    }

    let normal_handle = def
        .texture
        .as_ref()
        .map(|p| asset_server.load::<Image>(p.clone()));
    let pressed_handle = def
        .pressed_texture
        .as_ref()
        .map(|p| asset_server.load::<Image>(p.clone()));

    let bg_color = Color::srgba(1.0, 1.0, 1.0, opacity);

    let mut btn_cmd = commands.spawn((
        Name::new(format!("TouchBtn_{}", def.action)),
        TouchAction(def.action.clone()),
        TouchNormalImage(normal_handle.clone()),
        TouchPressedImage(pressed_handle),
        Button,
        node,
    ));

    if let Some(ref handle) = normal_handle {
        btn_cmd.insert((
            ImageNode::new(handle.clone()),
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, opacity)),
        ));
    } else {
        btn_cmd.insert((
            BackgroundColor(bg_color),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.6)),
        ));
        // Insert border only for non-textured buttons
        btn_cmd.entry::<Node>().and_modify(|mut n| {
            n.border = UiRect::all(Val::Px(2.0));
        });
    }

    let btn = btn_cmd.id();

    // Add text label if defined (shown even with textures if label is set)
    if let Some(ref label) = def.label {
        // Only show text on non-textured buttons or when explicitly set
        if def.texture.is_none() || true {
            let text = commands
                .spawn((
                    Text::new(label.clone()),
                    TextFont {
                        font_size: 18.0 * scale,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Pickable::IGNORE,
                ))
                .id();
            commands.entity(btn).add_child(text);
        }
    }

    btn
}

/// Fallback hardcoded layout when no config is provided.
fn spawn_fallback_layout(
    commands: &mut Commands,
    registry: &ActionRegistry,
    root: Entity,
    opacity: f32,
    scale: f32,
) {
    let btn_size = 52.0 * scale;
    let action_btn_size = 56.0 * scale;
    let margin = 8.0 * scale;
    let bg = Color::srgba(1.0, 1.0, 1.0, opacity);

    // D-pad container
    let dpad = commands
        .spawn((
            Name::new("DPadContainer"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(24.0 * scale),
                bottom: Val::Px(24.0 * scale),
                width: Val::Px(btn_size * 3.0 + margin * 2.0),
                height: Val::Px(btn_size * 3.0 + margin * 2.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();

    let dpad_btns = [
        ("Up", "▲", 1, 0),
        ("Down", "▼", 1, 2),
        ("Left", "◀", 0, 1),
        ("Right", "▶", 2, 1),
    ];
    for (action, label, col, row) in &dpad_btns {
        if registry.get(action).is_none() {
            continue;
        }
        let btn = spawn_simple_button(
            commands,
            action,
            label,
            btn_size,
            Val::Px(*col as f32 * (btn_size + margin)),
            Val::Px(*row as f32 * (btn_size + margin)),
            bg,
            scale,
        );
        commands.entity(dpad).add_child(btn);
    }
    commands.entity(root).add_child(dpad);

    // Action buttons container
    let actions = commands
        .spawn((
            Name::new("ActionButtonContainer"),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(24.0 * scale),
                bottom: Val::Px(24.0 * scale),
                width: Val::Px(action_btn_size * 3.0 + margin * 2.0),
                height: Val::Px(action_btn_size * 3.0 + margin * 2.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();

    let action_btns: &[(&str, &str, f32, f32)] = &[
        ("Confirm", "Z", 1.0, 2.0),
        ("Cancel", "X", 2.0, 1.0),
        ("Menu", "C", 0.0, 1.0),
    ];
    for (action, label, col, row) in action_btns {
        if registry.get(action).is_none() {
            continue;
        }
        let btn = spawn_simple_button(
            commands,
            action,
            label,
            action_btn_size,
            Val::Px(*col * (action_btn_size + margin)),
            Val::Px(*row * (action_btn_size + margin)),
            bg,
            scale,
        );
        commands.entity(actions).add_child(btn);
    }
    commands.entity(root).add_child(actions);
}

fn spawn_simple_button(
    commands: &mut Commands,
    action_name: &str,
    label: &str,
    size: f32,
    left: Val,
    top: Val,
    bg: Color,
    scale: f32,
) -> Entity {
    let btn = commands
        .spawn((
            Name::new(format!("TouchBtn_{}", action_name)),
            TouchAction(action_name.to_string()),
            TouchNormalImage(None),
            TouchPressedImage(None),
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
            BackgroundColor(bg),
        ))
        .id();

    let text = commands
        .spawn((
            Text::new(label.to_string()),
            TextFont {
                font_size: 18.0 * scale,
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
            if !currently_pressed.contains(name)
                && let Some(slot) = registry.get(name)
            {
                set_button_state(&mut state, &slot, ButtonState::JustReleased);
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
/// Swaps textures if pressed_texture is available, otherwise tints.
pub fn update_touch_button_visuals(
    mut buttons: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &TouchNormalImage,
            &TouchPressedImage,
            Option<&mut ImageNode>,
        ),
        (Changed<Interaction>, With<TouchAction>),
    >,
) {
    for (interaction, mut bg, normal, pressed_img, image_node) in buttons.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                if let Some(mut img) = image_node
                    && let Some(ref handle) = pressed_img.0
                {
                    img.image = handle.clone();
                }
                *bg = BackgroundColor(BTN_PRESSED_COLOR);
            }
            _ => {
                if let Some(mut img) = image_node
                    && let Some(ref handle) = normal.0
                {
                    img.image = handle.clone();
                }
                *bg = BackgroundColor(FALLBACK_BTN_COLOR);
            }
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
