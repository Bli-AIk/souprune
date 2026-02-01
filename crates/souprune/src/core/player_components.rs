//! # player_components.rs
//!
//! # player_components.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module defines the Player marker component.
//! All player DATA (HP, level, inventory, etc.) is stored in LayeredFactDatabase,
//! NOT in ECS components. This ensures a single source of truth.
//!
//! 本模块定义 Player 标记组件。
//! 所有玩家数据（HP、等级、物品栏等）存储在 LayeredFactDatabase 中，
//! 而非 ECS 组件中。这确保了单一数据来源。
//!
//! ## Data Access
//!
//! ## 数据访问
//!
//! To read player data, use `PlayerDataView` from `core::view::ron_view::parsing`.
//! To write player data, modify the `LayeredFactDatabase` directly.
//!
//! 要读取玩家数据，请使用 `core::view::ron_view::parsing` 中的 `PlayerDataView`。
//! 要写入玩家数据，请直接修改 `LayeredFactDatabase`。
//!
//! ## Fact Keys
//!
//! ## 事实键
//!
//! - `player_name`: String - Player display name
//! - `player_lv`: Int - Current level
//! - `player_exp`: Int - Current experience
//! - `player_next_exp`: Int - Experience for next level
//! - `player_hp`: Int - Current health
//! - `player_hp_max`: Int - Maximum health
//! - `player_atk`: Int - Attack power
//! - `player_def`: Int - Defense power
//! - `player_gold`: Int - Gold amount
//! - `player_weapon`: String - Equipped weapon ID
//! - `player_armor`: String - Equipped armor ID
//! - `player_inventory`: String - Comma-separated item IDs
//! - `player_inventory_capacity`: Int - Max inventory size

use bevy::prelude::*;

/// Marker component to identify the player entity.
/// Note: This entity has NO data components - all player data is in LayeredFactDatabase.
///
/// 标记组件，用于识别玩家实体。
/// 注意：此实体没有数据组件 - 所有玩家数据都在 LayeredFactDatabase 中。
#[derive(Component, Default)]
pub struct Player;
