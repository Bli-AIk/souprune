//! # basic_components.rs
//!
//! # basic_components.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This file defines fundamental ECS components and data types used across the application, such as `Speed`, `Facing`, and the `Direction` enum.
//!
//! 此文件定义了应用程序中通用的基础 ECS 组件和数据类型，例如 `Speed`（速度）、`Facing`（朝向）以及 `Direction`（方向）枚举。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Component)]
pub(crate) struct Speed {
    pub value: f32,
}

#[derive(Component)]
pub(crate) struct Facing {
    pub value: Direction,
}

#[derive(Default, Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    #[default]
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}
impl Direction {
    pub fn as_vec2(&self) -> Vec2 {
        match self {
            Direction::Up => Vec2::Y,
            Direction::Down => -Vec2::Y,
            Direction::Left => -Vec2::X,
            Direction::Right => Vec2::X,
            Direction::UpLeft => Vec2::new(-1.0, 1.0).normalize(),
            Direction::UpRight => Vec2::new(1.0, 1.0).normalize(),
            Direction::DownLeft => Vec2::new(-1.0, -1.0).normalize(),
            Direction::DownRight => Vec2::new(1.0, -1.0).normalize(),
        }
    }
}
