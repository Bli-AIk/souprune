//! # character.rs
//!
//! ## Module Overview
//! This module manages character-related logic and systems within the overworld, focusing on movement states like walking and running.
//!
//! ## Source File Overview
//! This file defines the `CharacterPlugin`, which integrates systems for updating character movement and behavior in the overworld.
//!
//! ## 模块概述
//! 该模块管理着 Overworld 中与角色相关的逻辑和系统，主要关注行走和奔跑等移动状态。
//!
//! ## 源文件概述
//! 该文件定义了 `CharacterPlugin`，它集成了用于更新 Overworld 中角色移动和行为的系统。

use bevy::app::{App, Plugin};
use bevy::prelude::*;

pub(crate) mod components;
pub(crate) mod systems;

pub struct CharacterPlugin;
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::update_walking_system,
                systems::update_running_system,
            ),
        );
    }
}
