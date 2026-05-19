//! # chapter_schema.rs
//!
//! # chapter_schema.rs 文件
//!
//! Defines the schema for Sequence Chapters in RON files.
//! This module contains pure data structures that map directly to the `.sequence.ron` format.
//!
//! 定义 Sequence Chapter 在 RON 文件中的 Schema。
//! 本模块包含直接映射到 `.sequence.ron` 格式的纯数据结构。

mod actions;
mod element;
mod facts;
#[cfg(test)]
mod tests;
mod values;

pub use actions::{CameraAction, PlayerAction, UIAction};
pub use element::{ElementModification, ElementSelector, TweenTarget};
pub use facts::{AggregateRule, DataBinding, FactCondition, FactModificationDef, FactValueMatch};
pub use values::{ColorTuple, LogLevel, Value, Vec2Tuple, Vec3Tuple};

use self::element::{default_easing, ease_kind_serde};
use bevy_tween::interpolation::EaseKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Chapter {
    SpawnView {
        view_layout: String,
        #[serde(default)]
        bindings: HashMap<String, DataBinding>,
    },
    AwaitFact {
        condition: String,
        #[serde(default = "default_true")]
        local: bool,
    },
    SetViewFact {
        key: String,
        value: FactValueMatch,
    },
    DanmakuPerformance {
        performance: String,
        #[serde(default)]
        translation: Option<(f32, f32)>,
        /// Whether to block until the performance's `duration` elapses.
        /// Defaults to `true`.
        ///
        /// 是否阻塞直到演出的 `duration` 耗尽。默认为 `true`。
        #[serde(default = "default_true")]
        wait_for_completion: bool,
    },
    AlightMotionPerformance {
        amproj_path: String,
        #[serde(default)]
        alight_motion_config: Option<String>,
        #[serde(default = "default_true")]
        wait_for_completion: bool,
    },
    SetViewElement {
        selector: ElementSelector,
        target: TweenTarget,
        #[serde(default)]
        duration: Option<f32>,
        #[serde(default = "default_easing", with = "ease_kind_serde")]
        easing: EaseKind,
        #[serde(default)]
        wait_for_completion: bool,
    },
    Wait(f32),
    Sequence(Vec<Chapter>),
    Parallel(Vec<Chapter>),
    SetPlayer(PlayerAction),
    SetUI(UIAction),
    ModifyViewElement {
        selector: ElementSelector,
        modification: ElementModification,
    },
    SetCamera(CameraAction),
    Conditional {
        condition: FactCondition,
        then_branch: Box<Chapter>,
        #[serde(default)]
        else_branch: Option<Box<Chapter>>,
    },
    FactSwitch {
        fact_key: String,
        cases: Vec<(FactValueMatch, Chapter)>,
        #[serde(default)]
        default: Option<Box<Chapter>>,
    },
    EmitFactEvent {
        event_id: String,
        #[serde(default)]
        data: HashMap<String, String>,
    },
    ModifyFact {
        modifications: Vec<FactModificationDef>,
    },
    LoadFre {
        files: Vec<String>,
        #[serde(default)]
        aggregate: HashMap<String, AggregateRule>,
    },
    LoadEnemies {
        enemies: Vec<String>,
    },
    RunSequence {
        #[serde(default)]
        path: Option<String>,
        #[serde(default)]
        path_fact: Option<String>,
        #[serde(default)]
        params: HashMap<String, FactValueMatch>,
    },
    LoadMap {
        path: String,
        #[serde(default = "default_true")]
        generate_collision: bool,
        #[serde(default = "default_true")]
        process_objects: bool,
    },
    SetBgm {
        path: Option<String>,
        #[serde(default)]
        fade_in: Option<f32>,
    },
    /// Start a battle enemy speech bubble presentation.
    ///
    /// 启动战斗敌人对话气泡表现。
    BattleSpeechBubble(souprune_schema::battle::BattleSpeechBubbleDef),
    Log {
        text: String,
        #[serde(default)]
        level: LogLevel,
    },
    Custom {
        action_type: String,
        #[serde(default)]
        params: HashMap<String, String>,
    },
    /// Spawn a headless entity with a WASM behavior attached.
    /// The entity lives until the current mode is cleaned up (ModeScoped).
    ///
    /// 生成一个附带 WASM 行为的无头实体。
    /// 实体在当前模式被清理时销毁（ModeScoped）。
    SpawnBehavior {
        behavior_id: String,
        #[serde(default)]
        context: Option<String>,
    },
    /// Repeat a sequence of chapters until a `Break` is encountered.
    ///
    /// 重复执行一组章节，直到遇到 `Break`。
    Loop {
        body: Vec<Chapter>,
        /// Safety limit — `None` means unlimited.
        ///
        /// 安全上限 — `None` 表示无限循环。
        #[serde(default)]
        max_iterations: Option<u32>,
    },
    /// Exit the innermost `Loop`.
    ///
    /// 退出最内层的 `Loop`。
    Break,
    /// Randomly select one or more chapters from a candidate list and execute them.
    ///
    /// 从候选列表中随机选择一个或多个章节并执行。
    RandomPick {
        candidates: Vec<Chapter>,
        /// Number of candidates to pick (default: 1).
        ///
        /// 要选择的候选数量（默认：1）。
        #[serde(default = "default_one")]
        count: usize,
        /// Allow picking the same candidate more than once (default: false).
        ///
        /// 是否允许重复选择同一候选（默认：false）。
        #[serde(default)]
        allow_repeat: bool,
    },
    /// Internal sentinel — marks end of a loop iteration body.
    /// **Not for use in `.sequence.ron` files.**
    ///
    /// 内部标记 — 标识循环迭代体的结束。
    /// **不要在 `.sequence.ron` 文件中使用。**
    #[serde(skip)]
    LoopIterationEnd,
}

fn default_true() -> bool {
    true
}

fn default_one() -> usize {
    1
}
