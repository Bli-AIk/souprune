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
use crate::app_state::overworld::OverworldState;
use bevy::prelude::*;
use evalexpr::{
    ContextWithMutableFunctions, ContextWithMutableVariables, DefaultNumericTypes, EvalexprError,
    Function, HashMapContext, Value,
};

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

pub(crate) fn update_dynamic_camera_anchors_system(
    mut anchored_query: Query<
        (&mut CameraAnchored, &CameraAnchoredDynamic, &mut Transform),
        Without<Camera2d>,
    >,
    player_query: Query<
        &Transform,
        (
            With<crate::app_state::overworld::character::components::PlayerControlled>,
            Without<CameraAnchored>,
            Without<Camera2d>,
        ),
    >,
    camera_query: Query<
        &Transform,
        (
            With<Camera2d>,
            Without<CameraAnchored>,
            Without<crate::app_state::overworld::character::components::PlayerControlled>,
        ),
    >,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let mut context: HashMapContext<DefaultNumericTypes> = HashMapContext::new();
    let _ = context.set_function(
        "if".to_string(),
        Function::new(|argument| {
            if let Value::Tuple(args) = argument {
                if args.len() != 3 {
                    return Err(EvalexprError::wrong_function_argument_amount(3, args.len()));
                }
                if args[0].as_boolean()? {
                    Ok(args[1].clone())
                } else {
                    Ok(args[2].clone())
                }
            } else {
                Err(EvalexprError::wrong_function_argument_amount(3, 1))
            }
        }),
    );
    let _ = context.set_value(
        "player.x".to_string(),
        evalexpr::Value::Float(player_transform.translation.x as f64),
    );
    let _ = context.set_value(
        "player.y".to_string(),
        evalexpr::Value::Float(player_transform.translation.y as f64),
    );
    let _ = context.set_value(
        "camera.x".to_string(),
        evalexpr::Value::Float(camera_transform.translation.x as f64),
    );
    let _ = context.set_value(
        "camera.y".to_string(),
        evalexpr::Value::Float(camera_transform.translation.y as f64),
    );

    for (mut anchor, dynamic, mut transform) in anchored_query.iter_mut() {
        if let Some(expr) = &dynamic.y_expression {
            match evalexpr::eval_with_context(expr, &context) {
                Ok(val) => {
                    if let Ok(f) = val.as_float() {
                        let f: f64 = f;
                        let new_y = f as f32;
                        if anchor.offset.y != new_y {
                            info!(
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
        if let Some(expr) = &dynamic.x_expression {
            if let Ok(val) = evalexpr::eval_with_context(expr, &context) {
                if let Ok(f) = val.as_float() {
                    let f: f64 = f;
                    anchor.offset.x = f as f32;
                }
            }
        }
        if let Some(expr) = &dynamic.z_expression {
            if let Ok(val) = evalexpr::eval_with_context(expr, &context) {
                if let Ok(f) = val.as_float() {
                    let f: f64 = f;
                    anchor.offset.z = f as f32;
                }
            }
        }

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
