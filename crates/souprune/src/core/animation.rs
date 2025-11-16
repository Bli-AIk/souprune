//! # animation.rs
//!
//! ## Module Overview
//! This module provides the core animation functionalities for sprites within the game.
//!
//! ## Source File Overview
//! This file defines the `AnimationPlugin`, which manages the lifecycle of sprite animations,
//! including synchronization, frame updates, and clip setup.
//!
//! ## 模块概述
//! 该模块为游戏中的精灵提供了核心动画功能。
//!
//! ## 源文件概述
//! 该文件定义了 `AnimationPlugin`，它管理精灵动画的生命周期，
//! 包括同步、帧更新和动画剪辑设置。

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
