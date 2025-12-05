//! # animation.rs
//!
//! # animation.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module powers the game's core sprite animation functionality.
//!
//! 该模块为游戏精灵提供核心动画能力。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! The file defines `AnimationPlugin`, which manages sprite animation sync, updates, and clip setup.
//!
//! 本文件定义了 `AnimationPlugin`，用于管理精灵动画的同步、更新与剪辑设置。

pub(crate) mod components;
mod systems;

use bevy::app::{App, Plugin, Update};
use bevy::prelude::IntoScheduleConfigs;

pub(crate) struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        use systems::*;
        app.add_systems(
            Update,
            (
                sync_sprite_animation_system,
                animate_sprite_system,
                update_sprite_animation_system,
                setup_sprite_animation_clip_system,
            )
                .chain(),
        );
    }
}
