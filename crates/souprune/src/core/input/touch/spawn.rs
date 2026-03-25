//! Spawns the touch-overlay entity tree from touch layout configuration.
//!
//! 根据触控布局配置生成触摸覆盖层的实体树。
//!
//! Acts as the construction step for touch controls. It translates
//! `TouchLayoutDef` data into Bevy UI nodes, controller zones, button images,
//! fallback layouts, and the marker components that later input/visual systems
//! rely on.
//!
//! 触控控制的构建步骤。它把 `TouchLayoutDef` 数据转换成 Bevy UI 节点、
//! 控制杆区域、按钮贴图、回退布局以及后续输入/表现系统依赖的各种标记组件。

use super::{
    AnimPhase, FALLBACK_OPACITY, TouchAction, TouchAnimFrames, TouchAnimState,
    TouchControllerOverlay, TouchControllerZone, TouchNormalImage, TouchOverlayRoot,
    TouchPressedImage,
};
use crate::core::input::actions::ActionRegistry;
use crate::core::input::config::{
    TOUCH_FRAME_TRANSITION_SECS, TouchAnchor, TouchButtonDef, TouchControllerDef, TouchLayoutDef,
};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

/// Spawn the touch overlay UI from a `TouchLayoutDef` config.
/// `resolution_scale` and `base_width` from SoupruneConfig are used to auto-scale.
/// Layout sizes are designed for `base_width * resolution_scale` logical pixels.
/// On screens with different widths, sizes scale proportionally.
pub fn spawn_touch_overlay(
    commands: &mut Commands,
    registry: &ActionRegistry,
    asset_server: &AssetServer,
    layout: Option<&TouchLayoutDef>,
    window_width: Option<f32>,
    resolution_scale: u32,
    base_width: u32,
) {
    info!("Spawning touch overlay UI");

    let opacity = layout.map(|l| l.opacity).unwrap_or(FALLBACK_OPACITY);
    let mut scale = layout.map(|l| l.scale).unwrap_or(1.0);

    let design_width = (base_width * resolution_scale) as f32;
    if let Some(win_w) = window_width
        && design_width > 0.0
    {
        scale *= win_w / design_width;
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let ms = layout.map(|l| l.mobile_scale).unwrap_or(0.5);
        info!("Applying mobile_scale={ms}");
        scale *= ms;
    }
    info!(
        "Touch overlay opacity={opacity}, scale={scale}, design_width={design_width}, window_width={window_width:?}"
    );

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
        if let Some(ref ctrl) = layout.controller {
            spawn_controller(commands, registry, asset_server, ctrl, opacity, scale, root);
        }

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

fn spawn_controller(
    commands: &mut Commands,
    _registry: &ActionRegistry,
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

    if let Some(ref label) = def.label
        && def.texture.is_none()
    {
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
