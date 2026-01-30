//! # utils.rs
//!
//! # utils.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module provides utility functions for querying player state.
//!
//! 本模块提供用于查询玩家状态的实用函数。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It includes functions for checking if the player is walking or performing actions.
//!
//! 包括检查玩家是否在行走或执行操作的函数。

use crate::app_state::overworld::OverworldState;
use crate::app_state::overworld::character::components::PlayerControlled;
use crate::app_state::overworld::player::config::PlayerBehavior;
use crate::core::input::{Action, ActionRegistry, ActionStateExt};
use bevy::prelude;
use bevy::prelude::{Query, Res, State, With};
use leafwing_input_manager::action_state::ActionState;

pub fn is_player_walking(
    query: Query<&ActionState<Action>, With<PlayerControlled>>,
    registry: Res<ActionRegistry>,
    overworld_state: Res<State<OverworldState>>,
) -> prelude::Result<(), ()> {
    // Allow player movement in Normal and Chase states.
    //
    // 在 Normal 和 Chase 状态下允许玩家移动。
    match *overworld_state.get() {
        OverworldState::Normal | OverworldState::Chase => {}
        _ => return Err(()),
    }

    let action_state = query.single().map_err(|_| ())?;

    let up_pressed = action_state.action_pressed(&registry, "Up");
    let down_pressed = action_state.action_pressed(&registry, "Down");
    let left_pressed = action_state.action_pressed(&registry, "Left");
    let right_pressed = action_state.action_pressed(&registry, "Right");

    let has_vertical_input = (up_pressed && !down_pressed) || (down_pressed && !up_pressed);
    let has_horizontal_input = (left_pressed && !right_pressed) || (right_pressed && !left_pressed);

    if has_vertical_input || has_horizontal_input {
        Ok(())
    } else {
        Err(())
    }
}

pub fn is_player_running(
    query: Query<&ActionState<Action>, With<PlayerControlled>>,
    registry: Res<ActionRegistry>,
    overworld_state: Res<State<OverworldState>>,
    player_behavior: Res<PlayerBehavior>,
) -> prelude::Result<(), ()> {
    // Allow player sprinting in Normal and Chase states.
    //
    // 在 Normal 和 Chase 状态下允许玩家跑步。
    match *overworld_state.get() {
        OverworldState::Normal | OverworldState::Chase => {}
        _ => return Err(()),
    }

    let Some(run_action_name) = &player_behavior.run_action else {
        return Err(());
    };

    let Some(run_action) = registry.get(run_action_name) else {
        return Err(());
    };

    let action_state = query.single().map_err(|_| ())?;
    if action_state.pressed(&run_action) {
        Ok(())
    } else {
        Err(())
    }
}
