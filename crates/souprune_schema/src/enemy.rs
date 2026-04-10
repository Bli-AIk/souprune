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
    /// Sequence paths for enemy turns (e.g. danmaku patterns).
    ///
    /// 敌人回合的序列路径（如弹幕 pattern）。
    #[serde(default)]
    pub turns: Vec<String>,
    /// Strategy for selecting the next turn.
    ///
    /// 选择下一个回合的策略。
    #[serde(default)]
    pub turn_strategy: TurnStrategy,
}

/// Strategy for selecting which turn sequence to execute.
///
/// 选择执行哪个回合序列的策略。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
