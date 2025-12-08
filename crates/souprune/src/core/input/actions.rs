//! # actions.rs
//!
//! # actions.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module defines the semantic input actions for the game.
//!
//! 该模块定义了游戏的语义输入动作。

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect, Serialize, Deserialize)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Cancel,
    Menu,
}
