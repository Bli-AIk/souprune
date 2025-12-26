//! # player_hp.rs
//!
//! # player_hp.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Implements a debug system to cycle the player's HP (1, Half, Full) using the F5 key for testing purposes.
//!
//! 实现一个调试系统，使用 F5 键循环切换玩家的 HP（1、半血、满血）以用于测试目的。

#[cfg(feature = "debug")]
pub mod debug_player_hp {
    use crate::core::data::PlayerData;
    use bevy::prelude::*;

    pub(crate) fn setup_player_hp_debug(app: &mut App) {
        app.add_systems(Update, debug_player_hp_system);
    }

    fn debug_player_hp_system(
        input: Res<ButtonInput<KeyCode>>,
        mut player_data: ResMut<PlayerData>,
    ) {
        if input.just_pressed(KeyCode::F5) {
            let max = player_data.hp_max;
            let current = player_data.hp;

            // Logic: Max -> Max/2 -> 1 -> Max
            // 逻辑：满血 -> 半血 -> 1 -> 满血
            let new_hp = if current == max {
                max / 2
            } else if current == max / 2 {
                1
            } else {
                max
            };

            player_data.hp = new_hp;
            info!("Debug: Player HP set to {}/{}", new_hp, max);
        }
    }
}
