//! # debug.rs
//!
//! # debug.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module offers debugging functionality guarded by the `debug` feature flag.
//!
//! 该模块在启用 `debug` 功能标志时提供调试功能。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It defines `DebugPlugin`, which wires up tools like the inspector whenever the `debug` feature is active.
//!
//! 本文件定义了 `DebugPlugin`，当 `debug` 功能启用时会初始化诸如检查器之类的工具。

#[cfg(feature = "debug")]
mod battle_test;
#[cfg(feature = "debug")]
mod collider;
#[cfg(feature = "debug")]
mod image_overlay;
mod inspector;
#[cfg(feature = "debug")]
mod player_hp;
#[cfg(feature = "debug")]
mod player_level;

use bevy::app::{App, Plugin};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, _app: &mut App) {
        #[cfg(feature = "debug")]
        {
            use inspector::debug_inspector;
            debug_inspector::setup_debug_features(_app);

            // Set up collider debugging features.
            //
            // 设置碰撞体调试功能。
            collider::debug_collider::setup_collider_debug(_app);

            // Set up image overlay debugging features.
            //
            // 设置图像覆盖调试功能。
            image_overlay::debug_image_overlay::setup_image_overlay_debug(_app);

            // Set up player HP debugging features.
            //
            // 设置玩家 HP 调试功能。
            player_hp::debug_player_hp::setup_player_hp_debug(_app);

            // Set up player Level debugging features.
            //
            // 设置玩家等级调试功能。
            player_level::debug_player_level::setup_player_level_debug(_app);

            // Set up battle test debugging features.
            //
            // 设置战斗测试调试功能。
            battle_test::debug_battle_test::setup_battle_test_debug(_app);
        }
    }
}
