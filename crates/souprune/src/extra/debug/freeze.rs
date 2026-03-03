//! # freeze.rs
//!
//! # freeze.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module provides game freeze/unfreeze functionality for debugging.
//!
//! 该模块提供用于调试的游戏冻结/解冻功能。
//!
//! ## Hotkey / 快捷键
//!
//! | Key | Function | 功能 |
//! |-----|----------|------|
//! | F7 | Toggle Game Freeze | 切换游戏冻结状态 |

use bevy::prelude::*;
use bevy::time::Virtual;

/// Resource to track freeze state and provide visual feedback
///
/// 用于跟踪冻结状态并提供视觉反馈的资源
#[derive(Resource, Default)]
pub struct GameFreezeState {
    pub is_frozen: bool,
}

/// System to toggle game freeze state with F7 key
///
/// 使用 F7 键切换游戏冻结状态的系统
pub fn toggle_game_freeze_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut time: ResMut<Time<Virtual>>,
    mut freeze_state: ResMut<GameFreezeState>,
) {
    if keyboard.just_pressed(KeyCode::F7) {
        freeze_state.is_frozen = !freeze_state.is_frozen;

        if freeze_state.is_frozen {
            time.pause();
            info!("Game FROZEN (press F7 to unfreeze)");
        } else {
            time.unpause();
            info!("Game UNFROZEN");
        }
    }
}

pub mod debug_freeze {
    use super::*;

    pub fn setup_freeze_debug(app: &mut App) {
        app.init_resource::<GameFreezeState>()
            .add_systems(Update, toggle_game_freeze_system);
    }
}
