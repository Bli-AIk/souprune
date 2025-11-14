//! # sprite.rs
//!
//! ## Module Overview
//! This module provides core functionalities for sprite management, including loading, parameters, and resources.
//!
//! ## Source File Overview
//! This file defines the `SpritePlugin`, which initializes and manages the `ModuleSpriteRegistry` and related sprite resources.
//!
//! ## 模块概述
//! 该模块提供了精灵管理的核心功能，包括加载、参数和资源。
//!
//! ## 源文件概述
//! 该文件定义了 `SpritePlugin`，它初始化并管理 `ModuleSpriteRegistry` 和相关的精灵资源。

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