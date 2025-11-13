//! # app_state.rs
//!
//! ## Module Overview
//! This file does not contain any submodules; it is a standalone file. Keep it simple.
//!
//! ## Source File Overview
//! This file defines the application's state enumeration.
//!
//! It includes states for setup, menu, overworld, and battle.
//!
//! The entire game's state management is based on this enumeration, with the Setup state being the first to be entered.
//!
//! ## 模块概述
//! app_state.rs 不包含任何子模块，它只是一个单独的文件，一切从简。
//! 
//! ## 源文件概述
//! 此处定义了应用程序的状态枚举。
//!
//! 包含初始化、菜单界面、Overworld 和战斗状态。
//!
//! 整个游戏的状态管理都基于此枚举进行。且 Setup 状态会最先被进入。

use bevy::prelude::States;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum AppState {
    #[default]
    Setup,
    Menu,
    Overworld,
    Battle,
}

// TODO: 状态管理、状态转换
