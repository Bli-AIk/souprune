//! # ui.rs
//!
//! ## Module Overview
//! This module manages the UI components within the Overworld game state.
//! The UI is implemented using SDF rendering provided by bevy_smud and mesh-based text rendering
//! provided by bevy_rich_text3d.
//!
//! ## Source File Overview
//! This file defines the `OverworldUIPlugin`.
//!
//! ## 模块概述
//! 该模块管理着 Overworld 游戏状态中的 UI 组件。此 UI 是基于 bevy_smud 提供的 SDF 渲染
//! 与 bevy_rich_text3d 提供的，基于 Mesh 的文本渲染实现的。  
//!
//! ## 源文件概述
//! 该文件定义了 `OverworldUIPlugin`。

use bevy::prelude::*;
mod components;

pub(crate) struct OverworldUIPlugin;

impl Plugin for OverworldUIPlugin {
    fn build(&self, app: &mut App) {}
}
