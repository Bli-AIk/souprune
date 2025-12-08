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
use crate::core::input::Action;
use bevy::prelude;
use bevy::prelude::{Query, Res, State, With};
use leafwing_input_manager::action_state::ActionState;

pub fn is_player_walking(
    query: Query<&ActionState<Action>, With<PlayerControlled>>,
    overworld_state: Res<State<OverworldState>>,
) -> prelude::Result<(), ()> {
    // Allow player movement only when the overworld state is Normal.
    //
    // 只在 Normal 状态下允许玩家移动。
    if *overworld_state != OverworldState::Normal {
        return Err(());
    }

    let action_state = query.single().map_err(|_| ())?;

    let up_pressed = action_state.pressed(&Action::Up);
    let down_pressed = action_state.pressed(&Action::Down);
    let left_pressed = action_state.pressed(&Action::Left);
    let right_pressed = action_state.pressed(&Action::Right);

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
    overworld_state: Res<State<OverworldState>>,
    player_behavior: Res<PlayerBehavior>,
) -> prelude::Result<(), ()> {
    // Allow player sprinting only when the overworld state is Normal.
    //
    // 只在 Normal 状态下允许玩家跑步。
    if *overworld_state != OverworldState::Normal {
        return Err(());
    }

    let Some(run_action) = player_behavior.run_action else {
        return Err(());
    };

    let action_state = query.single().map_err(|_| ())?;
    if action_state.pressed(&run_action) {
        Ok(())
    } else {
        Err(())
    }
}
