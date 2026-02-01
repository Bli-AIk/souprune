//! # player_level.rs
//!
//! # player_level.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Implements a debug system to toggle the player's Level and HP (LV 20/99HP vs LV 1/20HP) using the F8 key.
//!
//! 实现一个调试系统，使用 F8 键切换玩家的等级和血量（LV 20/99HP 对比 LV 1/20HP）。

#[cfg(feature = "debug")]
pub mod debug_player_level {
    use bevy::prelude::*;
    use bevy_fact_rule_event::LayeredFactDatabase;

    pub(crate) fn setup_player_level_debug(app: &mut App) {
        app.add_systems(Update, debug_player_level_system);
    }

    fn debug_player_level_system(
        input: Res<ButtonInput<KeyCode>>,
        mut layered_db: ResMut<LayeredFactDatabase>,
    ) {
        if input.just_pressed(KeyCode::F8) {
            let current_lv = layered_db.get_int("player_lv").unwrap_or(1);

            if current_lv == 20 {
                // Switch back to LV 1
                // 切回 LV 1
                layered_db.set_global("player_lv", 1i64);
                layered_db.set_global("player_hp_max", 20i64);
                layered_db.set_global("player_hp", 20i64);
                info!("[F8 DEBUG] Player Reset to LV 1, HP 20/20");
            } else {
                // Switch to LV 20
                // 切换到 LV 20
                layered_db.set_global("player_lv", 20i64);
                layered_db.set_global("player_hp_max", 99i64);
                layered_db.set_global("player_hp", 99i64);
                info!("[F8 DEBUG] Player Boosted to LV 20, HP 99/99");
            }
        }
    }
}
