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
//! | F8 | Toggle Debug Camera | 切换调试摄像机（滚轮缩放、中键平移） |
//! | F9 | Restart Game | 重启游戏（重新启动进程） |
//! | F10 | Toggle Image Diff | 切换图像 diff 对比 |
//! | F11 | Toggle View Layout Observer | 切换 View 布局观察器 |
//! | F12 | Toggle Debug Help Text | 切换调试帮助文本 |

#[cfg(feature = "debug")]
mod battle_test;
#[cfg(feature = "debug")]
mod camera_debug;
mod collider;
#[cfg(feature = "debug")]
mod fre_panel;
mod freeze;
#[cfg(feature = "debug")]
mod image_overlay;
#[cfg(feature = "debug")]
mod inspector;
#[cfg(feature = "debug")]
mod restart;
#[cfg(feature = "debug")]
mod state_overlay;
#[cfg(feature = "debug")]
mod view_layout_observer;

use bevy::app::{App, Plugin};
use bevy::prelude::*;
use std::collections::HashMap;

/// Marker component for debug cameras (inspector, FRE panel, etc.).
/// This is used to exclude debug cameras from game systems that query Camera2d.
///
/// 调试相机的标记组件（检查器、FRE 面板等）。
/// 用于将调试相机排除在查询 Camera2d 的游戏系统之外。
#[derive(Component)]
pub struct DebugCamera;

/// Event to show a debug toast notification in the top-left corner.
///
/// 在左上角显示调试提示通知的事件。
#[cfg(feature = "debug")]
#[derive(bevy::ecs::message::Message)]
pub struct DebugToastEvent {
    pub message: String,
}

// Collider gizmos and freeze are available without the debug feature.
pub use collider::debug_collider::{ColliderGizmos, setup_collider_debug};
pub use freeze::GameFreezeState;
pub use freeze::debug_freeze::setup_freeze_debug;

/// Resource to track recently triggered rules for visual feedback.
/// Always available so runtime debug UI can show trigger highlights.
///
/// 始终可用，让运行时调试 UI 可以显示规则触发高亮。
#[derive(Resource, Default)]
pub struct RuleTriggerHistory {
    /// Map from rule_id to last trigger timestamp (in seconds).
    pub triggered_rules: HashMap<String, f64>,
}

impl RuleTriggerHistory {
    pub fn record_trigger(&mut self, rule_id: &str, current_time: f64) {
        self.triggered_rules
            .insert(rule_id.to_string(), current_time);
    }

    pub fn was_recently_triggered(&self, rule_id: &str, current_time: f64, duration: f64) -> bool {
        if let Some(&trigger_time) = self.triggered_rules.get(rule_id) {
            current_time - trigger_time < duration
        } else {
            false
        }
    }

    pub fn cleanup_old_triggers(&mut self, current_time: f64) {
        self.triggered_rules
            .retain(|_, &mut trigger_time| current_time - trigger_time < 5.0);
    }
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // RuleTriggerHistory is always available (used by both debug FRE panel and editor)
        app.init_resource::<RuleTriggerHistory>()
            .add_systems(Update, cleanup_rule_trigger_history_system);

        #[cfg(feature = "debug")]
        {
            app.add_message::<DebugToastEvent>();

            setup_collider_debug(app);
            setup_freeze_debug(app);

            inspector::setup_debug_features(app);

            fre_panel::debug_fre_panel::setup_fre_panel_debug(app);

            image_overlay::debug_image_overlay::setup_image_overlay_debug(app);

            battle_test::debug_battle_test::setup_battle_test_debug(app);

            camera_debug::debug_camera::setup_camera_debug(app);

            restart::debug_restart::setup_restart_debug(app);

            state_overlay::debug_state_overlay::setup_state_overlay(app);

            view_layout_observer::setup_view_layout_observer_debug(app);
        }
    }
}

fn cleanup_rule_trigger_history_system(mut history: ResMut<RuleTriggerHistory>, time: Res<Time>) {
    history.cleanup_old_triggers(time.elapsed_secs_f64());
}
