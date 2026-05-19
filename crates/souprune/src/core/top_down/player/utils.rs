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

use crate::core::input::InputCommandState;
use crate::core::mode::SequenceSubState;
use crate::core::state_config::LoadedStateConfig;
use crate::core::top_down::player::config::PlayerBehavior;
use bevy::prelude;
use bevy::prelude::{Res, State};

pub fn is_player_walking(
    input_state: Res<InputCommandState>,
    sub_state: Res<State<SequenceSubState>>,
    state_config: Option<Res<LoadedStateConfig>>,
) -> prelude::Result<(), ()> {
    // Check if current state allows player movement via config
    // 通过配置检查当前状态是否允许玩家移动
    let player_movable = state_config
        .as_ref()
        .map(|c| c.is_player_movable(sub_state.name()))
        .unwrap_or(true);

    if !player_movable {
        return Err(());
    }

    if input_state.has_navigation_input() {
        Ok(())
    } else {
        Err(())
    }
}

pub fn is_player_running(
    input_state: Res<InputCommandState>,
    sub_state: Res<State<SequenceSubState>>,
    player_behavior: Res<PlayerBehavior>,
    state_config: Option<Res<LoadedStateConfig>>,
) -> prelude::Result<(), ()> {
    // Check if current state allows player movement via config
    // 通过配置检查当前状态是否允许玩家移动
    let player_movable = state_config
        .as_ref()
        .map(|c| c.is_player_movable(sub_state.name()))
        .unwrap_or(true);

    if !player_movable {
        return Err(());
    }

    let Some(run_action_name) = &player_behavior.run_action else {
        return Err(());
    };

    if input_state.source_action_pressed(run_action_name) {
        Ok(())
    } else {
        Err(())
    }
}
