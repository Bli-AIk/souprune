//! # input.rs
//!
//! # input.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module defines player actions and input resources for the game.
//!
//! 该模块定义游戏的玩家动作与输入资源。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It implements `InputPlugin`, which initializes and manages `PlayerInputSettings` plus related configuration.
//!
//! 本文件实现了 `InputPlugin`，用于初始化并管理 `PlayerInputSettings` 与相关配置。

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
