//! # sprite.rs
//!
//! # sprite.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module handles sprite management fundamentals such as loading, parameters, and resources.
//!
//! 该模块负责精灵加载、参数与资源等核心管理功能。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It defines `SpritePlugin`, which initializes and manages `ModuleSpriteRegistry` along with related resources.
//!
//! 本文件定义了 `SpritePlugin`，用于初始化并管理 `ModuleSpriteRegistry` 及相关资源。

pub(crate) mod load_context;
pub(crate) mod params;
pub(crate) mod resources;
mod utils;

pub(crate) use resources::*;

use bevy::app::{App, Plugin};

pub(crate) struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModuleSpriteRegistry>();
    }
}
