//! # enemy.rs
//!
//! EnemyDef schema types for `.enemy.ron` files.
//! Mirrors `souprune::core::enemy` without Bevy dependency.
//!
//! `.enemy.ron` 文件的敌人定义 Schema 类型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// A named group of turn sequences with its own selection strategy.
///
/// 命名的回合组，每组有独立的选择策略。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnGroup {
    pub turns: Vec<String>,
    #[serde(default)]
    pub strategy: TurnStrategy,
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
    /// Named turn groups — each group has its own turn list and selection strategy.
    /// Use `PickEnemyTurn(group: "group_name")` to select from a specific group.
    ///
    /// 命名回合组 — 每组有独立的回合列表和选择策略。
    /// 通过 `PickEnemyTurn(group: "group_name")` 从指定组中选择。
    #[serde(default)]
    pub turn_groups: HashMap<String, TurnGroup>,
}

/// Strategy for selecting which turn sequence to execute.
///
/// 选择执行哪个回合序列的策略。
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum TurnStrategy {
    /// Play turns in order, cycling back to the start.
    ///
    /// 按顺序播放，循环回到开头。
    #[default]
    Sequential,
    /// Pick a random turn each time.
    ///
    /// 每次随机选择一个回合。
    Random,
    /// Shuffle the turn pool, play through, re-shuffle when exhausted.
    ///
    /// 随机打乱回合池，全部播放完后重新打乱。
    Shuffle,
}
