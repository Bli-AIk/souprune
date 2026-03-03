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
//! It implements `InputPlugin`, which initializes and manages input configuration.
//!
//! 本文件实现了 `InputPlugin`，用于初始化并管理输入配置。

pub mod actions;
pub(crate) mod config;
pub(crate) mod resources;
pub mod touch;

pub use actions::*;
pub(crate) use config::*;
pub(crate) use resources::*;

use crate::core::ron_loader::RonAssetLoader;
use bevy::app::{App, Plugin};
use bevy::asset::AssetApp;

pub(crate) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        // Note: ActionRegistry and PlayerInputSettings are initialized in lib.rs
        // from the RON configuration file before this plugin is added.
        //
        // 注意：ActionRegistry 和 PlayerInputSettings 在此插件添加之前
        // 已在 lib.rs 中从 RON 配置文件初始化。
        app.init_asset::<InputConfig>()
            .register_asset_loader(RonAssetLoader::<InputConfig>::new(&["input.ron"]))
            .add_plugins(touch::TouchPlugin);
    }
}
