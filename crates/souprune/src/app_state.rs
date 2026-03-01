//! # app_state.rs
//!
//! # app_state.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! The `app_state` module contains the concrete modules for each application state.
//!
//! app_state 模块包含了每个应用程序状态的具体模块。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! This file defines the application's state enumeration used throughout the game.
//!
//! 此处定义了贯穿整个游戏的应用程序状态枚举。
//!
//! It includes states for setup, menu, overworld, and battle.
//!
//! 包含初始化、菜单界面、Overworld 和战斗状态。
//!
//! The entire game's state management is based on this enumeration, with the Setup state entered first.
//!
//! 整个游戏的状态管理都基于此枚举，且 Setup 状态会最先被进入。

use bevy::prelude::{Commands, Component, Entity, Query, States, With};

pub(crate) mod app_setup;
pub(crate) mod battle;
pub(crate) mod overworld;

/// Application lifecycle state.
/// Controls the overall application phase (setup → menu → playing).
///
/// 应用程序生命周期状态。
/// 控制整体应用阶段（设置 → 菜单 → 游戏中）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
#[allow(dead_code)]
pub enum AppState {
    #[default]
    AppSetup,
    Menu,
    Playing,
}

/// Game mode state, active during `AppState::Playing`.
/// Determines which game systems run (overworld exploration vs battle).
/// Controlled by sequences — not hardcoded transitions.
///
/// 游戏模式状态，在 `AppState::Playing` 期间激活。
/// 决定哪些游戏系统运行（大地图探索 vs 战斗）。
/// 由序列控制，而非硬编码转换。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum GameMode {
    #[default]
    None,
    Overworld,
    Battle,
}

pub fn cleanup_entities_system<T: Component>(
    mut commands: Commands,
    query: Query<Entity, With<T>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
