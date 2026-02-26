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
use super::config::{
    TOUCH_FRAME_TRANSITION_SECS, TouchAnchor, TouchButtonDef, TouchControllerDef, TouchLayoutDef,
};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leafwing_input_manager::action_state::ActionKindData;
use leafwing_input_manager::buttonlike::ButtonState;
use leafwing_input_manager::plugin::InputManagerSystem;
use leafwing_input_manager::prelude::ActionState;
use std::collections::HashSet;

// ── Components ──

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
        Self(true)
    }
}

/// Tracks which touch actions were pressed in the previous frame.
#[derive(Resource, Default)]
struct PrevTouchPressed(HashSet<String>);

/// Active direction actions from the controller zone, determined by touch position.
#[derive(Resource, Default)]
pub struct ControllerDirections(pub HashSet<String>);

/// Stores the normal-state image handle for pressed/released visual swap.
#[derive(Component)]
pub struct TouchNormalImage(pub Option<Handle<Image>>);

/// Stores the pressed-state image handle for visual swap.
#[derive(Component)]
pub struct TouchPressedImage(pub Option<Handle<Image>>);

/// Marker for invisible controller touch zones (excluded from visual updates).
#[derive(Component)]
pub struct TouchControllerZone;

/// Animation frame handles for a button [idle, pressing, pressed, releasing].
#[derive(Component)]
pub struct TouchAnimFrames(pub Vec<Handle<Image>>);

/// Current animation state for a button with frame-based animation.
#[derive(Component)]
pub struct TouchAnimState {
    pub current_frame: usize,
    pub timer: Timer,
    pub phase: AnimPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimPhase {
    Idle,
    Pressing,
    Held,
    Releasing,
}

/// Marker for controller direction overlay entities. Stores the action name.
#[derive(Component)]
pub struct TouchControllerOverlay(pub String);

// ── Plugin ──

pub(crate) struct TouchPlugin;

impl Plugin for TouchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TouchOverlayEnabled>()
            .init_resource::<PrevTouchPressed>()
            .init_resource::<ControllerDirections>()
            .add_systems(
                PreUpdate,
                inject_touch_actions
                    .in_set(InputManagerSystem::ManualControl)
                    .run_if(resource_exists::<ActionRegistry>),
            );
    }
}

// ── Constants ──

const FALLBACK_OPACITY: f32 = 0.45;
const FALLBACK_BTN_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, FALLBACK_OPACITY);
const BTN_PRESSED_COLOR: Color = Color::srgba(0.7, 0.9, 1.0, 0.7);

// ── Spawn ──

/// Spawn the touch overlay UI from a `TouchLayoutDef` config.
pub fn spawn_touch_overlay(
    commands: &mut Commands,
    registry: &ActionRegistry,
    asset_server: &AssetServer,
    layout: Option<&TouchLayoutDef>,
) {
    info!("Spawning touch overlay UI");

    let opacity = layout.map(|l| l.opacity).unwrap_or(FALLBACK_OPACITY);
    let scale = layout.map(|l| l.scale).unwrap_or(1.0);
    info!("Touch overlay opacity={opacity}, scale={scale}");

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
        // Spawn controller if defined
        if let Some(ref ctrl) = layout.controller {
            spawn_controller(commands, registry, asset_server, ctrl, opacity, scale, root);
        }

        // Spawn action buttons
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
        spawn_fallback_layout(commands, registry, root, opacity, scale);
    }
}

/// Spawn the controller (D-pad) with base image and direction overlays.
fn spawn_controller(
    commands: &mut Commands,
    registry: &ActionRegistry,
    asset_server: &AssetServer,
    def: &TouchControllerDef,
    opacity: f32,
    scale: f32,
    root: Entity,
) {
    let size = def.size * scale;

    let mut container_node = Node {
        position_type: PositionType::Absolute,
        width: Val::Px(size),
        height: Val::Px(size),
        ..default()
    };
    apply_anchor(
        &mut container_node,
        def.anchor,
        def.offset_x,
        def.offset_y,
        scale,
    );

    let container = commands
        .spawn((
            Name::new("ControllerContainer"),
            container_node,
            Pickable::IGNORE,
        ))
        .id();

    // Base image (fills the entire container)
    let base_handle = asset_server.load::<Image>(&def.base_texture);
    let tint = Color::srgba(1.0, 1.0, 1.0, opacity);
    let base = commands
        .spawn((
            Name::new("ControllerBase"),
            ImageNode::new(base_handle).with_color(tint),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(container).add_child(base);

    // Direction overlays (hidden by default, shown when pressed)
    for (action, texture_path) in &def.overlays {
        let handle = asset_server.load::<Image>(texture_path.clone());
        let overlay = commands
            .spawn((
                Name::new(format!("ControllerOverlay_{action}")),
                TouchControllerOverlay(action.clone()),
                ImageNode::new(handle).with_color(tint),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                Visibility::Hidden,
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(container).add_child(overlay);
    }

    // Single touch zone covering the entire controller for position-based direction detection.
    // Uses RelativeCursorPosition to determine which direction(s) are active.
    let zone = commands
        .spawn((
            Name::new("ControllerZone"),
            TouchControllerZone,
            Button,
            RelativeCursorPosition::default(),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .id();
    commands.entity(container).add_child(zone);

    commands.entity(root).add_child(container);
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

    let mut node = Node {
        position_type: PositionType::Absolute,
        width: Val::Px(w),
        height: Val::Px(h),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    apply_anchor(&mut node, def.anchor, def.offset_x, def.offset_y, scale);

    // Animated frames mode
    if let Some(ref frame_paths) = def.frames {
        let frame_handles: Vec<Handle<Image>> = frame_paths
            .iter()
            .map(|p| asset_server.load::<Image>(p.clone()))
            .collect();
        let initial_handle = frame_handles.first().cloned();

        let mut btn_cmd = commands.spawn((
            Name::new(format!("TouchBtn_{}", def.action)),
            TouchAction(def.action.clone()),
            TouchNormalImage(None),
            TouchPressedImage(None),
            TouchAnimFrames(frame_handles),
            TouchAnimState {
                current_frame: 0,
                timer: Timer::from_seconds(TOUCH_FRAME_TRANSITION_SECS, TimerMode::Once),
                phase: AnimPhase::Idle,
            },
            Button,
            node,
            BackgroundColor(Color::NONE),
        ));

        if let Some(handle) = initial_handle {
            let tint = Color::srgba(1.0, 1.0, 1.0, opacity);
            btn_cmd.insert(ImageNode::new(handle).with_color(tint));
        }

        return btn_cmd.id();
    }

    // Legacy two-texture mode
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
        btn_cmd.entry::<Node>().and_modify(|mut n| {
            n.border = UiRect::all(Val::Px(2.0));
        });
    }

    let btn = btn_cmd.id();

    if let Some(ref label) = def.label {
        if def.texture.is_none() {
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

fn apply_anchor(node: &mut Node, anchor: TouchAnchor, offset_x: f32, offset_y: f32, scale: f32) {
    match anchor {
        TouchAnchor::BottomLeft => {
            node.left = Val::Px(offset_x * scale);
            node.bottom = Val::Px(offset_y * scale);
        }
        TouchAnchor::BottomRight => {
            node.right = Val::Px(offset_x * scale);
            node.bottom = Val::Px(offset_y * scale);
        }
        TouchAnchor::TopLeft => {
            node.left = Val::Px(offset_x * scale);
            node.top = Val::Px(offset_y * scale);
        }
        TouchAnchor::TopRight => {
            node.right = Val::Px(offset_x * scale);
            node.top = Val::Px(offset_y * scale);
        }
    }
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

// ── Controller Direction Detection ──

/// Deadzone threshold for controller zone: positions within this distance from
/// center on either axis are ignored (0.1 = 20% of half-width).
const CONTROLLER_DEADZONE: f32 = 0.1;

/// Determines active direction actions from the controller touch zone position.
pub fn update_controller_directions(
    zones: Query<(&Interaction, &RelativeCursorPosition), With<TouchControllerZone>>,
    mut dirs: ResMut<ControllerDirections>,
) {
    dirs.0.clear();
    for (interaction, rel_pos) in zones.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // normalized: (0,0) = center, (-0.5,-0.5) = top-left, (0.5,0.5) = bottom-right
        if let Some(pos) = rel_pos.normalized {
            if pos.y < -CONTROLLER_DEADZONE {
                dirs.0.insert("Up".to_string());
            }
            if pos.y > CONTROLLER_DEADZONE {
                dirs.0.insert("Down".to_string());
            }
            if pos.x < -CONTROLLER_DEADZONE {
                dirs.0.insert("Left".to_string());
            }
            if pos.x > CONTROLLER_DEADZONE {
                dirs.0.insert("Right".to_string());
            }
        }
    }
}

// ── Action Injection ──

/// Reads Bevy UI Interaction on touch buttons and injects corresponding
/// actions into all entities carrying ActionState<Action>.
fn inject_touch_actions(
    enabled: Res<TouchOverlayEnabled>,
    registry: Res<ActionRegistry>,
    buttons: Query<(&Interaction, &TouchAction)>,
    controller_dirs: Res<ControllerDirections>,
    mut action_states: Query<&mut ActionState<Action>>,
    mut prev: ResMut<PrevTouchPressed>,
) {
    if !enabled.0 {
        return;
    }

    let mut currently_pressed = HashSet::new();
    for (interaction, touch_action) in buttons.iter() {
        if *interaction == Interaction::Pressed {
            currently_pressed.insert(touch_action.0.clone());
        }
    }
    // Merge directions from controller zone position detection
    for dir in controller_dirs.0.iter() {
        currently_pressed.insert(dir.clone());
    }

    for mut state in action_states.iter_mut() {
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

// ── Visual Updates ──

/// Update button visuals: handles both legacy two-texture and animated frame modes.
pub fn update_touch_button_visuals(
    mut legacy_buttons: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &TouchNormalImage,
            &TouchPressedImage,
            Option<&mut ImageNode>,
        ),
        (
            Changed<Interaction>,
            With<TouchAction>,
            Without<TouchAnimFrames>,
            Without<TouchControllerZone>,
        ),
    >,
    mut anim_buttons: Query<
        (
            &Interaction,
            &mut TouchAnimState,
            &TouchAnimFrames,
            &mut ImageNode,
        ),
        (Changed<Interaction>, With<TouchAction>),
    >,
) {
    // Legacy two-texture buttons
    for (interaction, mut bg, normal, pressed_img, image_node) in legacy_buttons.iter_mut() {
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

    // Animated frame buttons: trigger animation on interaction change
    for (interaction, mut anim, frames, mut img) in anim_buttons.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                if anim.phase != AnimPhase::Pressing && anim.phase != AnimPhase::Held {
                    // Start press animation: show frame 1
                    anim.phase = AnimPhase::Pressing;
                    anim.current_frame = 1;
                    anim.timer.reset();
                    if let Some(handle) = frames.0.get(1) {
                        img.image = handle.clone();
                    }
                }
            }
            _ => {
                if anim.phase == AnimPhase::Pressing || anim.phase == AnimPhase::Held {
                    // Start release animation: show frame 3
                    anim.phase = AnimPhase::Releasing;
                    anim.current_frame = 3;
                    anim.timer.reset();
                    if let Some(handle) = frames.0.get(3) {
                        img.image = handle.clone();
                    }
                }
            }
        }
    }
}

/// Tick animation timers and advance frames.
pub fn tick_touch_button_animations(
    time: Res<Time>,
    mut buttons: Query<(&mut TouchAnimState, &TouchAnimFrames, &mut ImageNode)>,
) {
    for (mut anim, frames, mut img) in buttons.iter_mut() {
        anim.timer.tick(time.delta());
        if !anim.timer.is_finished() {
            continue;
        }
        match anim.phase {
            AnimPhase::Pressing => {
                // Transition 1 → 2 (held)
                anim.phase = AnimPhase::Held;
                anim.current_frame = 2;
                if let Some(handle) = frames.0.get(2) {
                    img.image = handle.clone();
                }
            }
            AnimPhase::Releasing => {
                // Transition 3 → 0 (idle)
                anim.phase = AnimPhase::Idle;
                anim.current_frame = 0;
                if let Some(handle) = frames.0.get(0) {
                    img.image = handle.clone();
                }
            }
            _ => {}
        }
    }
}

/// Update controller direction overlays based on active controller directions.
pub fn update_controller_overlays(
    dirs: Res<ControllerDirections>,
    mut overlays: Query<(&TouchControllerOverlay, &mut Visibility)>,
) {
    for (overlay, mut vis) in overlays.iter_mut() {
        *vis = if dirs.0.contains(&overlay.0) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
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
