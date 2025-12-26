//! # player_level.rs
//!
//! # player_level.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Implements a debug system to toggle the player's Level and HP (LV 20/99HP vs LV 1/20HP) using the F7 key.
//!
//! 实现一个调试系统，使用 F7 键切换玩家的等级和血量（LV 20/99HP 对比 LV 1/20HP）。

#[cfg(feature = "debug")]
pub mod debug_player_level {
    use crate::core::data::PlayerData;
    use bevy::prelude::*;

    pub(crate) fn setup_player_level_debug(app: &mut App) {
        app.add_systems(Update, debug_player_level_system);
    }

    fn debug_player_level_system(
        input: Res<ButtonInput<KeyCode>>,
        mut player_data: ResMut<PlayerData>,
    ) {
        if input.just_pressed(KeyCode::F7) {
            if player_data.lv == 20 {
                // Switch back to LV 1
                // 切回 LV 1
                player_data.lv = 1;
                player_data.hp_max = 20;
                player_data.hp = 20;
                info!(
                    "[F7 DEBUG] Player Reset to LV 1, HP 20/20 | hp_max={}",
                    player_data.hp_max
                );
            } else {
                // Switch to LV 20
                // 切换到 LV 20
                player_data.lv = 20;
                player_data.hp_max = 99;
                player_data.hp = 99;
                info!(
                    "[F7 DEBUG] Player Boosted to LV 20, HP 99/99 | hp_max={}",
                    player_data.hp_max
                );
            }
        }
    }
}
