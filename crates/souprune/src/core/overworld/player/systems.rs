//! # systems.rs
//!
//! # systems.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module implements systems for processing player input and updating movement.
//!
//! 本模块实现处理玩家输入和更新移动的系统。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It manages character animation states based on player actions.
//!
//! 根据玩家操作管理角色动画状态。

use crate::core::basic_components::{Direction, Facing};
use crate::core::input::{Direction as InputDirection, InputCommandState};
use crate::core::mode::SequenceSubState;
use crate::core::overworld::character::components::PlayerControlled;
use crate::core::state_config::LoadedStateConfig;
use bevy::prelude::{Query, Res, State, With};

pub(crate) fn player_direction_control_system(
    mut query: Query<&mut Facing, With<PlayerControlled>>,
    input_state: Res<InputCommandState>,
    sub_state: Res<State<SequenceSubState>>,
    state_config: Option<Res<LoadedStateConfig>>,
) {
    // Check if current state allows player movement via config
    // 通过配置检查当前状态是否允许玩家移动
    let player_movable = state_config
        .as_ref()
        .map(|c| c.is_player_movable(sub_state.name()))
        .unwrap_or(true);

    if !player_movable {
        return;
    }

    for mut facing in query.iter_mut() {
        let up_pressed = input_state.navigation_pressed(InputDirection::Up);
        let down_pressed = input_state.navigation_pressed(InputDirection::Down);
        let left_pressed = input_state.navigation_pressed(InputDirection::Left);
        let right_pressed = input_state.navigation_pressed(InputDirection::Right);

        // Only update the facing direction when opposite inputs are not pressed simultaneously.
        //
        // 只有在相反方向没有同时按下时才更新朝向。
        if up_pressed && !down_pressed {
            facing.value = Direction::Up;
        } else if down_pressed && !up_pressed {
            facing.value = Direction::Down;
        } else if left_pressed && !right_pressed {
            facing.value = Direction::Left;
        } else if right_pressed && !left_pressed {
            facing.value = Direction::Right;
        }
    }
}
