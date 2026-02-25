//! # touch.rs
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module implements a virtual touch overlay for mobile/touch platforms.
//! It renders on-screen controls (D-pad, action buttons) and translates touch
//! events into game actions via the leafwing_input_manager system.
//!
//! 该模块实现了移动/触屏平台的虚拟触控覆盖层。
//! 它渲染屏幕上的控件（方向键、动作按钮），并通过 leafwing_input_manager 系统
//! 将触摸事件转换为游戏动作。

use super::actions::{Action, ActionRegistry};
use super::config::{InputConfig, TouchOverlayConfig};
use crate::core::ron_loader::RonAssetLoader;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Touch overlay layout asset loaded from RON files.
///
/// 从 RON 文件加载的触控覆盖层布局资产。
#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
pub struct TouchLayoutAsset {
    /// Virtual button definitions.
    ///
    /// 虚拟按钮定义。
    pub buttons: Vec<TouchButtonDef>,
}

/// Defines a virtual touch button.
///
/// 定义一个虚拟触控按钮。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TouchButtonDef {
    /// Unique identifier for this button.
    pub id: String,

    /// Kind of button (single action or D-pad).
    pub kind: TouchButtonKind,

    /// Position relative to screen anchor (in logical pixels, from bottom-center).
    /// Negative X = left side, Positive X = right side.
    pub position: (f32, f32),

    /// Size of the button (radius for circular, half-size for D-pad).
    pub size: f32,
}

/// Kind of virtual touch button.
///
/// 虚拟触控按钮类型。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TouchButtonKind {
    /// A single action button (e.g., Confirm, Cancel).
    Single {
        /// The action name this button triggers.
        action: String,
        /// Path to the button texture (relative to assets).
        #[serde(default)]
        texture: Option<String>,
    },

    /// A directional pad (4-way).
    DPad {
        /// Map of direction names to action names.
        actions: HashMap<String, String>,
        /// Path to the D-pad texture (relative to assets).
        #[serde(default)]
        texture: Option<String>,
    },
}

/// Marker component for the touch overlay root entity.
///
/// 触控覆盖层根实体的标记组件。
#[derive(Component)]
pub struct TouchOverlayRoot;

/// Component on individual virtual touch buttons.
///
/// 单个虚拟触控按钮上的组件。
#[derive(Component)]
pub struct TouchButton {
    /// Action slot(s) this button triggers.
    pub actions: Vec<(String, Action)>,
    /// Whether this button is currently pressed.
    pub pressed: bool,
    /// Hit detection radius in logical pixels.
    pub hit_radius: f32,
}

/// Resource tracking which touch buttons are pressed.
///
/// 追踪哪些触控按钮被按下的资源。
#[derive(Resource, Default)]
pub struct TouchInputState {
    pub pressed_actions: Vec<Action>,
}

pub(crate) struct TouchPlugin;

impl Plugin for TouchPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TouchLayoutAsset>()
            .register_asset_loader(RonAssetLoader::<TouchLayoutAsset>::new(&[
                "touch_layout.ron",
            ]))
            .init_resource::<TouchInputState>()
            .add_systems(Update, process_touch_input_system);
    }
}

/// System that processes raw touch input and maps it to action states.
/// This reads `TouchInput` events and checks against registered touch button regions.
///
/// 处理原始触摸输入并映射到动作状态的系统。
fn process_touch_input_system(
    touches: Res<Touches>,
    mut touch_state: ResMut<TouchInputState>,
    buttons: Query<(&TouchButton, &GlobalTransform)>,
) {
    touch_state.pressed_actions.clear();

    for touch in touches.iter() {
        let touch_pos = touch.position();

        for (button, transform) in buttons.iter() {
            let button_pos = transform.translation().truncate();
            let half = button.hit_radius;

            let in_bounds = touch_pos.x >= button_pos.x - half
                && touch_pos.x <= button_pos.x + half
                && touch_pos.y >= button_pos.y - half
                && touch_pos.y <= button_pos.y + half;

            if in_bounds {
                for (_, action) in &button.actions {
                    touch_state.pressed_actions.push(*action);
                }
            }
        }
    }
}
