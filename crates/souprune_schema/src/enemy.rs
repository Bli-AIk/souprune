//! # enemy.rs
//!
//! EnemyDef schema types for `.enemy.ron` files.
//! Mirrors `souprune::core::enemy` without Bevy dependency.
//!
//! `.enemy.ron` 文件的敌人定义 Schema 类型。

use serde::{Deserialize, Serialize};

/// Localization information shared across definitions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocaleInfo {
    pub name: String,
    #[serde(default)]
    pub file: String,
}

/// Combat statistics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CombatStats {
    pub hp: i64,
    #[serde(default)]
    pub attack: i64,
    #[serde(default)]
    pub defense: i64,
}

/// An action option available to the player (ACT or MERCY).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionOption {
    pub label: String,
    pub sequence: String,
    #[serde(default)]
    pub param: String,
}

/// Typed enemy definition — top-level `.enemy.ron` schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyDef {
    pub id: String,
    #[serde(default)]
    pub locale: LocaleInfo,
    #[serde(default)]
    pub stats: CombatStats,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub mortar_path: String,
    #[serde(default)]
    pub acts: Vec<ActionOption>,
    #[serde(default)]
    pub mercies: Vec<ActionOption>,
}
