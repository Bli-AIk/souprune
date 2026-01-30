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

use crate::app_state::overworld::OverworldState;
use crate::app_state::overworld::character::components::PlayerControlled;
use crate::core::basic_components::{Direction, Facing};
use crate::core::input::{Action, ActionRegistry, ActionStateExt};
use bevy::prelude::{Query, Res, State, With};
use leafwing_input_manager::action_state::ActionState;

pub(crate) fn player_direction_control_system(
    mut query: Query<(&mut Facing, &ActionState<Action>), With<PlayerControlled>>,
    registry: Res<ActionRegistry>,
    overworld_state: Res<State<OverworldState>>,
) {
    // Allow direction control in Normal and Chase states.
    //
    // 在 Normal 和 Chase 状态下允许方向控制。
    match *overworld_state.get() {
        OverworldState::Normal | OverworldState::Chase => {}
        _ => return,
    }

    for (mut facing, action_state) in query.iter_mut() {
        let up_pressed = action_state.action_pressed(&registry, "Up");
        let down_pressed = action_state.action_pressed(&registry, "Down");
        let left_pressed = action_state.action_pressed(&registry, "Left");
        let right_pressed = action_state.action_pressed(&registry, "Right");

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
