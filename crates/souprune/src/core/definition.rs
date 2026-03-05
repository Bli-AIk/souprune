//! # definition.rs
//!
//! Shared data structures for typed game definitions.
//! Common sub-structs (`LocaleInfo`, `CombatStats`) for composition.
//!
//! 游戏定义的共享数据结构。
//! 通用子结构体（`LocaleInfo`、`CombatStats`）用于组合。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Localization information shared across game definitions.
///
/// 跨游戏定义共享的本地化信息。
#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Default)]
pub struct LocaleInfo {
    /// Localization key for display name
    pub name: String,
    /// Locale file path (e.g. "items", "enemies")
    #[serde(default)]
    pub file: String,
}

/// Combat statistics shared across enemies, bosses, etc.
///
/// 跨敌人、Boss 等共享的战斗数值。
#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Default)]
pub struct CombatStats {
    pub hp: i64,
    #[serde(default)]
    pub attack: i64,
    #[serde(default)]
    pub defense: i64,
}
