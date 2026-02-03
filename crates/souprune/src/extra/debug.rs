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
//!
//! ## Debug Hotkeys / 调试快捷键
//!
//! | Key | Function | 功能 |
//! |-----|----------|------|
//! | F1 | Toggle Inspector Window | 切换检查器独立窗口 |
//! | F2 | Toggle FRE Debug Panel | 切换 FRE 调试面板（通用 Fact 操作） |
//! | F3 | Toggle Perf UI | 切换性能监控 UI |
//! | F4 | Toggle Collider Gizmos | 切换碰撞体显示 |
//! | F5 | Toggle Image Overlay | 切换图像覆盖 |
//! | F6 | Enter Battle Test | 进入战斗测试 |
//! | F7 | Toggle Game Freeze | 切换游戏冻结状态 |
//! | F12 | Toggle Debug Help Text | 切换调试帮助文本 |

#[cfg(feature = "debug")]
mod battle_test;
#[cfg(feature = "debug")]
mod collider;
#[cfg(feature = "debug")]
mod fre_panel;
#[cfg(feature = "debug")]
mod freeze;
#[cfg(feature = "debug")]
mod image_overlay;
mod inspector;

use bevy::app::{App, Plugin};
use bevy::prelude::Component;

/// Marker component for debug cameras (inspector, FRE panel, etc.).
/// This is used to exclude debug cameras from game systems that query Camera2d.
///
/// 调试相机的标记组件（检查器、FRE 面板等）。
/// 用于将调试相机排除在查询 Camera2d 的游戏系统之外。
#[derive(Component)]
pub struct DebugCamera;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, _app: &mut App) {
        #[cfg(feature = "debug")]
        {
            use inspector::debug_inspector;
            debug_inspector::setup_debug_features(_app);

            // F2: Set up FRE debug panel.
            // Use this for generic fact manipulation instead of game-specific shortcuts.
            // 设置 FRE 调试面板 (F2)。
            // 使用此面板进行通用事实操作，而非游戏特定的快捷键。
            fre_panel::debug_fre_panel::setup_fre_panel_debug(_app);

            // F4: Set up collider debugging features.
            // 设置碰撞体调试功能 (F4)。
            collider::debug_collider::setup_collider_debug(_app);

            // F5: Set up image overlay debugging features.
            // 设置图像覆盖调试功能 (F5)。
            image_overlay::debug_image_overlay::setup_image_overlay_debug(_app);

            // F6: Set up battle test debugging features.
            // 设置战斗测试调试功能 (F6)。
            battle_test::debug_battle_test::setup_battle_test_debug(_app);

            // F7: Set up game freeze debugging features.
            // 设置游戏冻结调试功能 (F7)。
            freeze::debug_freeze::setup_freeze_debug(_app);
        }
    }
}
