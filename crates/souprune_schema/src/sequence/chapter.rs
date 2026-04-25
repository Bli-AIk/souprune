//! # Sequence Chapter Schema
//!
//! Chapter variants for `.sequence.ron` assets.
//!
//! # 序列章节 Schema
//!
//! `.sequence.ron` 资源的章节变体。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    AggregateRule, CameraAction, DataBinding, EaseKindRepr, ElementModification, ElementSelector,
    FactCondition, FactModificationDef, FactValueMatch, GapPolicy, LogLevel, PlayerAction,
    SplitAxis, TweenTarget, UIAction,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Chapter {
    SpawnView {
        view_layout: String,
        #[serde(default, serialize_with = "crate::ordered_map::serialize_ordered_map")]
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
        #[serde(default)]
        easing: EaseKindRepr,
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
        #[serde(default, serialize_with = "crate::ordered_map::serialize_ordered_map")]
        data: HashMap<String, String>,
    },
    ModifyFact {
        modifications: Vec<FactModificationDef>,
    },
    LoadFre {
        files: Vec<String>,
        #[serde(default, serialize_with = "crate::ordered_map::serialize_ordered_map")]
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
        #[serde(default, serialize_with = "crate::ordered_map::serialize_ordered_map")]
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
        #[serde(default)]
        easing: EaseKindRepr,
    },
    MergeBattleBoxes {
        sources: (String, String),
        result: String,
        #[serde(default)]
        gap_policy: GapPolicy,
        #[serde(default)]
        duration: f32,
        #[serde(default)]
        easing: EaseKindRepr,
    },
    SpawnBehavior {
        behavior_id: String,
        #[serde(default)]
        context: Option<String>,
    },
    Loop {
        body: Vec<Chapter>,
        #[serde(default)]
        max_iterations: Option<u32>,
    },
    Break,
    RandomPick {
        candidates: Vec<Chapter>,
        #[serde(default = "default_one")]
        count: usize,
        #[serde(default)]
        allow_repeat: bool,
    },
    PickEnemyTurn {
        #[serde(default)]
        enemy_id: Option<String>,
        #[serde(default)]
        enemy_id_fact: Option<String>,
        #[serde(default)]
        group: Option<String>,
        #[serde(default)]
        group_fact: Option<String>,
    },
    Log {
        text: String,
        #[serde(default)]
        level: LogLevel,
    },
    Custom {
        action_type: String,
        #[serde(default, serialize_with = "crate::ordered_map::serialize_ordered_map")]
        params: HashMap<String, String>,
    },
}

fn default_true() -> bool {
    true
}

fn default_one() -> usize {
    1
}
