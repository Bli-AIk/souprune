//! # definition.rs
//!
//! Shared data structures and traits for typed game definitions.
//! Provides `GameDefinition` trait for unified access and
//! common sub-structs (`LocaleInfo`, `CombatStats`) for composition.
//!
//! 游戏定义的共享数据结构和 trait。
//! 提供 `GameDefinition` trait 用于统一访问，
//! 以及通用子结构体（`LocaleInfo`、`CombatStats`）用于组合。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// --- Shared Sub-Structs ---

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

// --- Traits ---

/// Unified interface for all typed game definitions.
///
/// 所有类型化游戏定义的统一接口。
pub trait GameDefinition {
    /// Unique identifier for this definition.
    fn id(&self) -> &str;
    /// Human-readable kind name (e.g. "Enemy", "Item").
    fn kind() -> &'static str;
}

/// Unified query interface for definition registries.
///
/// 定义注册表的统一查询接口。
pub trait DefinitionRegistry {
    type Def: GameDefinition;
    fn get(&self, id: &str) -> Option<&Self::Def>;
    fn ids(&self) -> Vec<&str>;
}
