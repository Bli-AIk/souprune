//! # camera.rs
//!
//! # camera.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module handles UI elements that need to stay anchored to the camera position.
//!
//! 本模块处理需要锚定到摄像机位置的 UI 元素。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It ensures UI elements remain fixed on screen while the world camera moves.
//!
//! 确保 UI 元素在世界摄像机移动时保持固定在屏幕上。

use super::components::CameraAnchored;
use crate::app_state::overworld::OverworldState;
use bevy::prelude::*;

/// Apply camera offsets whenever the Backpack camera actually moves.
///
/// 当背包摄像机移动时才同步锚点，避免逐帧改写 Transform。
pub(crate) fn update_camera_anchored_ui_on_camera_move_system(
    overworld_state: Res<State<OverworldState>>,
    camera_query: Query<&Transform, (With<Camera2d>, Changed<Transform>)>,
    mut anchored_ui_query: Query<(&CameraAnchored, &mut Transform), Without<Camera2d>>,
) {
    if overworld_state.get() != &OverworldState::Backpack {
        return;
    }

    let Ok(camera_transform) = camera_query.single() else {
        // No camera moved this frame, so there is nothing to do.
        //
        // 本帧摄像机未移动，无需执行任何操作。
        return;
    };

    for (anchor, mut transform) in anchored_ui_query.iter_mut() {
        let new_translation = camera_transform.translation + anchor.offset;
        if transform.translation != new_translation {
            transform.translation = new_translation;
        }
    }
}

/// Initialize (or re-sync) anchors only when the entity's offset changes or gets added.
///
/// 仅在新 UI 产生或偏移量改变时同步，避免无意义写入。
#[allow(clippy::type_complexity)]
pub(crate) fn update_camera_anchored_ui_on_change_system(
    overworld_state: Res<State<OverworldState>>,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut anchored_ui_query: Query<
        (&CameraAnchored, &mut Transform),
        (
            Without<Camera2d>,
            Or<(Added<CameraAnchored>, Changed<CameraAnchored>)>,
        ),
    >,
) {
    if overworld_state.get() != &OverworldState::Backpack {
        return;
    }

    let Ok(camera_transform) = camera_query.single() else {
        warn!("No Camera2d available for anchoring UI");
        return;
    };

    for (anchor, mut transform) in anchored_ui_query.iter_mut() {
        let new_translation = camera_transform.translation + anchor.offset;
        if transform.translation != new_translation {
            transform.translation = new_translation;
        }
    }
}
