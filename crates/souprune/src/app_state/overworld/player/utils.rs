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

use crate::app_state::overworld::OverworldSubState;
use crate::app_state::overworld::character::components::PlayerControlled;
use crate::app_state::overworld::player::config::PlayerBehavior;
use crate::core::input::{Action, ActionRegistry, ActionStateExt, InputBehaviorConfig};
use crate::core::state_config::LoadedStateConfig;
use bevy::prelude;
use bevy::prelude::{Query, Res, State, With};
use leafwing_input_manager::action_state::ActionState;

pub fn is_player_walking(
    query: Query<&ActionState<Action>, With<PlayerControlled>>,
    registry: Res<ActionRegistry>,
    behavior_config: Res<InputBehaviorConfig>,
    overworld_state: Res<State<OverworldSubState>>,
    state_config: Option<Res<LoadedStateConfig>>,
) -> prelude::Result<(), ()> {
    // Check if current state allows player movement via config
    // 通过配置检查当前状态是否允许玩家移动
    let player_movable = state_config
        .as_ref()
        .map(|c| c.is_player_movable(overworld_state.name()))
        .unwrap_or(true);

    if !player_movable {
        return Err(());
    }

    let action_state = query.single().map_err(|_| ())?;

    // Use behavior config to get action names for navigation
    // 使用行为配置获取导航的动作名称
    let up_pressed = behavior_config
        .nav_up()
        .map(|name| action_state.action_pressed(&registry, name))
        .unwrap_or(false);
    let down_pressed = behavior_config
        .nav_down()
        .map(|name| action_state.action_pressed(&registry, name))
        .unwrap_or(false);
    let left_pressed = behavior_config
        .nav_left()
        .map(|name| action_state.action_pressed(&registry, name))
        .unwrap_or(false);
    let right_pressed = behavior_config
        .nav_right()
        .map(|name| action_state.action_pressed(&registry, name))
        .unwrap_or(false);

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
    overworld_state: Res<State<OverworldSubState>>,
    player_behavior: Res<PlayerBehavior>,
    state_config: Option<Res<LoadedStateConfig>>,
) -> prelude::Result<(), ()> {
    // Check if current state allows player movement via config
    // 通过配置检查当前状态是否允许玩家移动
    let player_movable = state_config
        .as_ref()
        .map(|c| c.is_player_movable(overworld_state.name()))
        .unwrap_or(true);

    if !player_movable {
        return Err(());
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
