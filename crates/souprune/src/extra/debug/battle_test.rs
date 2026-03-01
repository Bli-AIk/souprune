//! # battle_test.rs
//!
//! # battle_test.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Implements a debug system to switch AppState to Battle using the F6 key for testing purposes.
//!
//! 实现一个调试系统，使用 F6 键将 AppState 切换到 Battle 以用于测试目的。

#[cfg(feature = "debug")]
pub mod debug_battle_test {
    use crate::app_state::SequenceMode;
    use bevy::prelude::*;

    pub(crate) fn setup_battle_test_debug(app: &mut App) {
        app.add_systems(Update, debug_battle_test_system);
    }

    fn debug_battle_test_system(
        input: Res<ButtonInput<KeyCode>>,
        mut sequence_mode: ResMut<SequenceMode>,
    ) {
        if input.just_pressed(KeyCode::F6) {
            if sequence_mode.0.as_deref() != Some("battle") {
                sequence_mode.0 = Some("battle".to_string());
                info!("Debug: Switching to SequenceMode battle");
            } else {
                info!("Debug: Already in battle mode");
            }
        }
    }
}
