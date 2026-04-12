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
    AlightMotionPerformance {
        amproj_path: String,
        #[serde(default)]
        alight_motion_config: Option<String>,
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

/// Axis along which to split a battle box.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum SplitAxis {
    Vertical,
    #[default]
    Horizontal,
}

/// Policy for how gap affects split/merge geometry.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum GapPolicy {
    #[default]
    Expands,
    Includes,
}

/// Log level for Log chapter.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum LogLevel {
    #[default]
    Info,
    Debug,
    Warn,
    Error,
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
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum EaseKindRepr {
    #[default]
    Linear,
    #[serde(alias = "InQuad")]
    QuadIn,
    #[serde(alias = "OutQuad")]
    QuadOut,
    #[serde(alias = "InOutQuad")]
    QuadInOut,
    #[serde(alias = "InCubic")]
    CubicIn,
    #[serde(alias = "OutCubic")]
    CubicOut,
    #[serde(alias = "InOutCubic")]
    CubicInOut,
    #[serde(alias = "InSine")]
    SineIn,
    #[serde(alias = "OutSine")]
    SineOut,
    #[serde(alias = "InOutSine")]
    SineInOut,
    #[serde(alias = "InCirc")]
    CircularIn,
    #[serde(alias = "OutCirc")]
    CircularOut,
    #[serde(alias = "InOutCirc")]
    CircularInOut,
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
        /// Anchor point that stays fixed during resize (normalized, center = (0,0)).
        ///
        /// 缩放时保持不动的锚点（归一化坐标，中心 = (0,0)）。
        /// `(0, -1)` = bottom, `(0, 1)` = top, `(-1, 0)` = left, `(1, 0)` = right.
        #[serde(default)]
        anchor: Option<(f32, f32)>,
        /// Fact key to store the final offset position when anchor is used.
        /// The tween system writes the computed end position so that
        /// reconciliation can derive the correct desired Transform.
        ///
        /// 使用锚点时存储最终偏移位置的 fact 键名。
        /// Tween 系统写入计算出的终点位置，使协调系统能推导出正确的期望 Transform。
        #[serde(default)]
        anchor_fact: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_split_battle_box_with_legacy_easing_alias() {
        let ron = r#"SplitBattleBox(
            source: "main",
            result: ("left", "right"),
            axis: Vertical,
            position: 0.0,
            gap: 20.0,
            gap_policy: Expands,
            duration: 0.3,
            easing: OutCubic,
        )"#;

        let chapter: Chapter = ron::from_str(ron).expect("legacy easing alias should parse");
        match chapter {
            Chapter::SplitBattleBox { easing, .. } => {
                assert_eq!(easing, EaseKindRepr::CubicOut);
            }
            other => panic!("unexpected chapter parsed: {other:?}"),
        }
    }

    #[test]
    fn parses_split_battle_box_with_canonical_easing_name() {
        let ron = r#"SplitBattleBox(
            source: "main",
            result: ("left", "right"),
            axis: Vertical,
            position: 0.0,
            gap: 20.0,
            gap_policy: Expands,
            duration: 0.3,
            easing: CubicOut,
        )"#;

        let chapter: Chapter = ron::from_str(ron).expect("SplitBattleBox should parse");
        match chapter {
            Chapter::SplitBattleBox {
                source,
                result,
                axis,
                gap_policy,
                easing,
                ..
            } => {
                assert_eq!(source, "main");
                assert_eq!(result, ("left".to_string(), "right".to_string()));
                assert_eq!(axis, SplitAxis::Vertical);
                assert_eq!(gap_policy, GapPolicy::Expands);
                assert_eq!(easing, EaseKindRepr::CubicOut);
            }
            other => panic!("unexpected chapter parsed: {other:?}"),
        }
    }

    #[test]
    fn parses_merge_battle_boxes_and_log_chapters() {
        let merge = r#"MergeBattleBoxes(
            sources: ("left", "right"),
            result: "main",
            gap_policy: Includes,
            duration: 0.5,
            easing: CubicOut,
        )"#;
        let log = r#"Log(
            text: "hello",
            level: Warn,
        )"#;

        let merge_chapter: Chapter = ron::from_str(merge).expect("MergeBattleBoxes should parse");
        let log_chapter: Chapter = ron::from_str(log).expect("Log should parse");

        match merge_chapter {
            Chapter::MergeBattleBoxes {
                sources,
                result,
                gap_policy,
                easing,
                ..
            } => {
                assert_eq!(sources, ("left".to_string(), "right".to_string()));
                assert_eq!(result, "main");
                assert_eq!(gap_policy, GapPolicy::Includes);
                assert_eq!(easing, EaseKindRepr::CubicOut);
            }
            other => panic!("unexpected chapter parsed: {other:?}"),
        }

        match log_chapter {
            Chapter::Log { text, level } => {
                assert_eq!(text, "hello");
                assert_eq!(level, LogLevel::Warn);
            }
            other => panic!("unexpected chapter parsed: {other:?}"),
        }
    }
}
