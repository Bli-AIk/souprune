//! # app_state.rs
//!
//! ## Module Overview
//! app_state module contains the specific modules for its states.
//!
//! ## Source File Overview
//! This file defines the application's state enumeration.
//!
//! It includes states for setup, menu, overworld, and battle.
//!
//! The entire game's state management is based on this enumeration, with the Setup state being the first to be entered.
//!
//! ## 模块概述
//! app_state 模块包含了其状态的具体模块。
//!
//! ## 源文件概述
//! 此处定义了应用程序的状态枚举。
//!
//! 包含初始化、菜单界面、Overworld 和战斗状态。
//!
//! 整个游戏的状态管理都基于此枚举进行。且 Setup 状态会最先被进入。

use bevy::prelude::States;

pub(crate) mod app_setup;
pub(crate) mod overworld;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum AppState {
    #[default]
    AppSetup,
    Menu,
    Overworld,
    Battle,
}

// TODO: 状态管理、状态转换
