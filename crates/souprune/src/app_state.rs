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

use bevy::prelude::States;

pub(crate) mod app_setup;
pub(crate) mod overworld;
pub(crate) mod battle;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
#[allow(dead_code)]
pub enum AppState {
    #[default]
    AppSetup,
    Menu,
    Overworld,
    Battle,
}

// TODO: 状态管理、状态转换
