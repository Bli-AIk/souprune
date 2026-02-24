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

use super::components::{CameraAnchored, CameraAnchoredDynamic};
use super::expr_eval::eval_number;
use crate::app_state::overworld::OverworldSubState;
use crate::extra::debug::DebugCamera;
use bevy::prelude::*;
use std::collections::BTreeMap;

/// Apply camera offsets whenever the camera actually moves (works in states with UI interaction or chase config).
///
/// 当摄像机移动时同步锚点，支持有 UI 交互或追逐战配置的状态。
pub(crate) fn update_camera_anchored_ui_on_camera_move_system(
    app_state: Res<State<crate::app_state::AppState>>,
    overworld_state: Option<Res<State<OverworldSubState>>>,
    state_config: Option<Res<crate::core::state_config::LoadedStateConfig>>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<DebugCamera>, Changed<Transform>)>,
    mut anchored_ui_query: Query<
        (&CameraAnchored, &mut Transform),
        (Without<Camera2d>, Without<DebugCamera>),
    >,
) {
    // Only run in Battle state or Overworld states with UI interaction or chase config
    // 仅在 Battle 状态或具有 UI 交互或追逐战配置的 Overworld 状态下运行
    let should_run = match app_state.get() {
        crate::app_state::AppState::Battle => true,
        crate::app_state::AppState::Overworld => {
            if let (Some(ow_state), Some(config)) =
                (overworld_state.as_ref(), state_config.as_ref())
            {
                let state_name = ow_state.name();
                config.is_view_interactive(state_name) || config.is_chase_state(state_name)
            } else {
                false
            }
        }
        _ => false,
    };

    if !should_run {
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

#[allow(clippy::type_complexity)]
pub(crate) fn update_dynamic_camera_anchors_system(
    mut anchored_query: Query<
        (&mut CameraAnchored, &CameraAnchoredDynamic, &mut Transform),
        (Without<Camera2d>, Without<DebugCamera>),
    >,
    player_query: Query<
        (Ref<Transform>, ()),
        (
            With<crate::app_state::overworld::character::components::PlayerControlled>,
            Without<CameraAnchored>,
            Without<Camera2d>,
            Without<DebugCamera>,
        ),
    >,
    camera_query: Query<
        (Ref<Transform>, ()),
        (
            With<Camera2d>,
            Without<DebugCamera>,
            Without<CameraAnchored>,
            Without<crate::app_state::overworld::character::components::PlayerControlled>,
        ),
    >,
) {
    // Early exit if no dynamic anchors exist
    // 如果没有动态锚点则提前退出
    if anchored_query.is_empty() {
        return;
    }

    let Ok((player_transform_ref, _)) = player_query.single() else {
        return;
    };
    let Ok((camera_transform_ref, _)) = camera_query.single() else {
        return;
    };

    // Only update when player or camera transform changed
    // 仅在玩家或摄像机 Transform 变化时更新
    if !player_transform_ref.is_changed() && !camera_transform_ref.is_changed() {
        return;
    }

    let player_transform = &*player_transform_ref;
    let camera_transform = &*camera_transform_ref;

    // Build variable map for expression evaluation
    // 构建表达式求值所需的变量映射
    let mut vars: BTreeMap<String, f64> = BTreeMap::new();
    vars.insert(
        "player_x".to_string(),
        player_transform.translation.x as f64,
    );
    vars.insert(
        "player_y".to_string(),
        player_transform.translation.y as f64,
    );
    vars.insert(
        "camera_x".to_string(),
        camera_transform.translation.x as f64,
    );
    vars.insert(
        "camera_y".to_string(),
        camera_transform.translation.y as f64,
    );

    for (mut anchor, dynamic, mut transform) in anchored_query.iter_mut() {
        if let Some(expr) = &dynamic.y_expression {
            match eval_number(expr, &vars) {
                Ok(f) => {
                    let new_y = f as f32;
                    if anchor.offset.y != new_y {
                        trace!(
                            "Updating dynamic anchor Y: expr='{}', result={}, old_y={}, new_y={}, player_y={}, camera_y={}",
                            expr,
                            f,
                            anchor.offset.y,
                            new_y,
                            player_transform.translation.y,
                            camera_transform.translation.y
                        );
                        anchor.offset.y = new_y;
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to evaluate dynamic anchor expression '{}': {}",
                        expr, e
                    );
                }
            }
        }

        // Similar logic for X and Z if needed, but for now focusing on Y as per user report.
        if let Some(expr) = &dynamic.x_expression
            && let Ok(f) = eval_number(expr, &vars)
        {
            anchor.offset.x = f as f32;
        }
        if let Some(expr) = &dynamic.z_expression
            && let Ok(f) = eval_number(expr, &vars)
        {
            anchor.offset.z = f as f32;
        }

        let new_translation = camera_transform.translation + anchor.offset;
        if transform.translation != new_translation {
            transform.translation = new_translation;
        }
    }
}

/// Initialize (or re-sync) anchors only when the entity's offset changes or gets added (works in states with UI interaction or chase config).
///
/// 仅在新 UI 产生或偏移量改变时同步，支持有 UI 交互或追逐战配置的状态。
#[allow(clippy::type_complexity)]
pub(crate) fn update_camera_anchored_ui_on_change_system(
    app_state: Res<State<crate::app_state::AppState>>,
    overworld_state: Option<Res<State<OverworldSubState>>>,
    state_config: Option<Res<crate::core::state_config::LoadedStateConfig>>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<DebugCamera>)>,
    mut anchored_ui_query: Query<
        (&CameraAnchored, &mut Transform),
        (
            Without<Camera2d>,
            Without<DebugCamera>,
            Or<(Added<CameraAnchored>, Changed<CameraAnchored>)>,
        ),
    >,
) {
    // Only run in Battle state or Overworld states with UI interaction or chase config
    // 仅在 Battle 状态或具有 UI 交互或追逐战配置的 Overworld 状态下运行
    let should_run = match app_state.get() {
        crate::app_state::AppState::Battle => true,
        crate::app_state::AppState::Overworld => {
            if let (Some(ow_state), Some(config)) =
                (overworld_state.as_ref(), state_config.as_ref())
            {
                let state_name = ow_state.name();
                config.is_view_interactive(state_name) || config.is_chase_state(state_name)
            } else {
                false
            }
        }
        _ => false,
    };

    if !should_run {
        return;
    }

    let Ok(camera_transform) = camera_query.single() else {
        warn_once!("No Camera2d available for anchoring UI");
        return;
    };

    for (anchor, mut transform) in anchored_ui_query.iter_mut() {
        let new_translation = camera_transform.translation + anchor.offset;
        if transform.translation != new_translation {
            transform.translation = new_translation;
        }
    }
}
