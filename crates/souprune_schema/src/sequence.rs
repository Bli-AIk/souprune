//! # sequence.rs
//!
//! SequenceAsset schema types for `.sequence.ron` files.
//! Mirrors `souprune::core::sequencer::chapter_schema` without Bevy dependency.
//!
//! `.sequence.ron` 文件的序列资产 Schema 类型。

use crate::val::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Top-level Asset
// ============================================================================

/// Sequence configuration asset — top-level `.sequence.ron` schema.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SequenceAsset {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub rules_file: Option<String>,
    #[serde(default)]
    pub exits: HashMap<String, String>,
    pub chapters: Vec<Chapter>,
}

// ============================================================================
// Chapter Enum
// ============================================================================

/// Chapter — minimal unit of a linear sequence.
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
    AmPerformance {
        amproj_path: String,
        #[serde(default)]
        am_config: Option<String>,
        #[serde(default = "default_true")]
        wait_for_completion: bool,
    },
    TweenViewElement {
        selector: ElementSelector,
        target: TweenTarget,
        duration: f32,
        #[serde(default)]
        easing: EaseKindRepr,
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
    Custom {
        action_type: String,
        #[serde(default)]
        params: HashMap<String, String>,
    },
}

// ============================================================================
// Supporting Types
// ============================================================================

/// Fact condition for conditional chapters.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FactCondition {
    Equals { key: String, value: FactValueMatch },
    GreaterThan { key: String, value: i64 },
    LessThan { key: String, value: i64 },
    GreaterOrEqual { key: String, value: i64 },
    LessOrEqual { key: String, value: i64 },
    Exists(String),
    NotExists(String),
    IsTrue(String),
    IsFalse(String),
    And(Vec<FactCondition>),
    Or(Vec<FactCondition>),
    Not(Box<FactCondition>),
    Always,
}

/// Fact value for matching in conditions and switches.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FactValueMatch {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Expr(String),
}

/// Fact modification for ModifyFact chapter.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FactModificationDef {
    Set { key: String, value: FactValueMatch },
    Increment { key: String, amount: i64 },
    Remove(String),
    Toggle(String),
}

/// Aggregation rule for LoadFre chapter.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AggregateRule {
    Collect(String),
    CollectKeys(String),
}

/// Data binding for SpawnView's requires interfaces.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DataBinding {
    File(String),
    Files(Vec<String>),
    LocalLayer,
    Expr(String),
}

/// Camera action.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum CameraAction {
    SetPosition((f32, f32)),
    SetZoom(f32),
    Shake { duration: f32, intensity: f32 },
    FollowPlayer(bool),
}

/// UI action.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum UIAction {
    LoadLayout(String),
    Show(String),
    Hide(String),
    SetText { id: String, content: String },
    SetVariable { name: String, value: String },
    PlayAnimation { id: String, clip: String },
}

/// Player action.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PlayerAction {
    SetMode(Vec<String>),
    Spawn {
        config_path: String,
        #[serde(default)]
        position: Option<(f32, f32)>,
    },
    Teleport((f32, f32)),
    SetActive(bool),
    Despawn,
}

/// Element selector.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ElementSelector {
    FullName(String),
    LocalName(String),
    Tag(String),
}

/// Element modification.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ElementModification {
    SetTexture(String),
    SetPosition(Val<f32>, Val<f32>, Val<f32>),
    SetScale(Val<f32>, Val<f32>, Val<f32>),
    SetColor(Val<f32>, Val<f32>, Val<f32>, Val<f32>),
    SetVisibility(Val<bool>),
    SetBoxSize(Val<f32>, Val<f32>),
    Undo,
    Redo,
    Reset,
}

/// Easing function representation (PascalCase, matches bevy_tween EaseKind).
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub enum EaseKindRepr {
    #[default]
    Linear,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    SineIn,
    SineOut,
    SineInOut,
    ExpoIn,
    ExpoOut,
    ExpoInOut,
    ElasticIn,
    ElasticOut,
    ElasticInOut,
    BounceIn,
    BounceOut,
    BounceInOut,
    BackIn,
    BackOut,
    BackInOut,
}

/// Tween target property to animate (sequence context).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TweenTarget {
    Position {
        #[serde(default)]
        from: Option<Vec3Tuple>,
        to: Vec3Tuple,
    },
    Scale {
        #[serde(default)]
        from: Option<Vec3Tuple>,
        to: Vec3Tuple,
    },
    Color {
        #[serde(default)]
        from: Option<ColorTuple>,
        to: ColorTuple,
    },
    BoxSize {
        #[serde(default)]
        from: Option<Vec2Tuple>,
        to: Vec2Tuple,
    },
    Rotation {
        #[serde(default)]
        from: Option<Val<f32>>,
        to: Val<f32>,
    },
    Alpha {
        #[serde(default)]
        from: Option<Val<f32>>,
        to: Val<f32>,
    },
}

// ============================================================================
// Default helpers
// ============================================================================

fn default_true() -> bool {
    true
}
