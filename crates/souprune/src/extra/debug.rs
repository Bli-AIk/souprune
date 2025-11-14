//! # debug.rs
//!
//! ## Module Overview
//! This module provides debugging functionalities for the game, conditionally enabled by a feature flag.
//!
//! ## Source File Overview
//! This file defines the `DebugPlugin`, which sets up various debug features, such as an inspector,
//! when the "debug" feature is active.
//!
//! ## 模块概述
//! 该模块为游戏提供了调试功能，通过功能标志有条件地启用。
//!
//! ## 源文件概述
//! 该文件定义了 `DebugPlugin`，当“debug”功能激活时，它会设置各种调试功能，
//! 例如检查器。

#[cfg(feature = "debug")]
mod collider;
mod inspector;

use bevy::app::{App, Plugin};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, _app: &mut App) {
        #[cfg(feature = "debug")]
        {
            use inspector::debug_inspector;
            debug_inspector::setup_debug_features(_app);

            // Setup collider debug features
            collider::debug_collider::setup_collider_debug(_app);
        }
    }
}
