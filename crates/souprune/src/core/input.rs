//! # input.rs
//!
//! ## Module Overview
//! This module provides core functionalities for handling player input, including defining actions and managing input resources.
//!
//! ## Source File Overview
//! This file defines the `InputPlugin`, which initializes and manages `PlayerInputSettings` and related input configurations.
//!
//! ## 模块概述
//! 该模块提供了处理玩家输入的核心功能，包括定义动作和管理输入资源。
//!
//! ## 源文件概述
//! 该文件定义了 `InputPlugin`，它初始化并管理 `PlayerInputSettings` 和相关的输入配置。

pub(crate) mod actions;
pub(crate) mod resources;

pub(crate) use actions::*;
pub(crate) use resources::*;

use bevy::app::{App, Plugin};

pub(crate) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInputSettings>();
    }
}
