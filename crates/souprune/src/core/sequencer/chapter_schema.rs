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
use crate::preset::battle_box::{GapPolicy, SplitAxis};
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
    },
    AlightMotionPerformance {
        amproj_path: String,
        #[serde(default, alias = "am_config")]
        alight_motion_config: Option<String>,
        #[serde(default = "default_true")]
        wait_for_completion: bool,
    },
    TweenViewElement {
        selector: ElementSelector,
        target: TweenTarget,
        duration: f32,
        #[serde(default = "default_easing", with = "ease_kind_serde")]
        easing: EaseKind,
        #[serde(default = "default_true")]
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
        #[serde(default = "default_true")]
        setup_camera_bounds: bool,
    },
    SetBgm {
        path: Option<String>,
        #[serde(default)]
        fade_in: Option<f32>,
    },
    SplitBattleBox {
        source: String,
        result: (String, String),
        axis: SplitAxis,
        #[serde(default)]
        position: f32,
        #[serde(default)]
        gap: f32,
        #[serde(default)]
        gap_policy: GapPolicy,
        #[serde(default)]
        duration: f32,
        #[serde(default = "default_easing", with = "ease_kind_serde")]
        easing: EaseKind,
    },
    MergeBattleBoxes {
        sources: (String, String),
        result: String,
        #[serde(default)]
        gap_policy: GapPolicy,
        #[serde(default)]
        duration: f32,
        #[serde(default = "default_easing", with = "ease_kind_serde")]
        easing: EaseKind,
    },
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
}

fn default_true() -> bool {
    true
}
