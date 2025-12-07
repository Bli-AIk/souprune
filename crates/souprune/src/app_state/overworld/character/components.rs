//! # components.rs
//!
//! # components.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module defines components for character states and animation transitions.
//!
//! 本模块定义角色状态和动画转换的组件。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It includes components for idle, walking, and running states.
//!
//! 包括空闲、行走和奔跑状态的组件。

use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct PlayerControlled;

#[derive(Clone, Component)]
#[component(storage = "SparseSet")]
pub(crate) struct StateIdle;

#[derive(Clone, Component)]
#[component(storage = "SparseSet")]
pub(crate) struct StateWalking;

#[derive(Clone, Component)]
#[component(storage = "SparseSet")]
pub(crate) struct StateRunning;

// TODO: 添加 CharacterBundle。即简化的 PlayerBundle，用于非玩家角色
