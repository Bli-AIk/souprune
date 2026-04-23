//! # sequence.rs
//!
//! SequenceAsset schema types for `.sequence.ron` files.
//! Mirrors `souprune::core::sequencer::chapter_schema` without Bevy dependency.
//!
//! `.sequence.ron` 文件的序列资源 Schema 类型。

use crate::val::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Top-level Asset
// ============================================================================

/// Sequence configuration asset — top-level `.sequence.ron` schema.
///
/// 序列配置资源 — `.sequence.ron` 的顶层 Schema。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SequenceAsset {
    /// Identifier for the execution mode.
    ///
    /// 执行模式的标识符。
    #[serde(default)]
    pub mode: Option<String>,
    /// Path to a `.fre.ron` file containing local rules for this sequence.
    ///
    /// 包含此序列本地规则的 `.fre.ron` 文件路径。
    #[serde(default)]
    pub rules_file: Option<String>,
    /// Exit mappings (e.g., `"success": "next_level"`).
    ///
    /// 退出映射（如 `"success": "next_level"`）。
    #[serde(default)]
    pub exits: HashMap<String, String>,
    /// List of chapters making up the sequence.
    ///
    /// 构成序列的章节列表。
    pub chapters: Vec<Chapter>,
}

// ============================================================================
// Chapter Enum
// ============================================================================

/// Chapter — minimal unit of a linear sequence.
///
/// 章节 — 线性序列的最小单元。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Chapter {
    /// Spawns a UI view layout.
    ///
    /// 生成一个 UI 视图布局。
    SpawnView {
        /// Path to the `.view.ron` file.
        ///
        /// `.view.ron` 文件的路径。
        view_layout: String,
        /// Initial fact bindings for the view.
        ///
        /// 视图的初始 fact 绑定。
        #[serde(default)]
        bindings: HashMap<String, DataBinding>,
    },
    /// Blocks the sequence until a FRE condition is met.
    ///
    /// 阻塞序列，直到满足特定的 FRE 条件。
    AwaitFact {
        /// FRE condition expression.
        ///
        /// FRE 条件表达式。
        condition: String,
        /// Whether to check local facts (default: true).
        ///
        /// 是否检查本地 fact（默认：true）。
        #[serde(default = "default_true")]
        local: bool,
    },
    /// Updates a fact in the active view's local fact database.
    ///
    /// 更新当前活动视图本地 fact 数据库中的值。
    SetViewFact {
        /// Fact key to update.
        ///
        /// 要更新的 fact 键。
        key: String,
        /// New value for the fact.
        ///
        /// fact 的新值。
        value: FactValueMatch,
    },
    /// Triggers a danmaku performance.
    ///
    /// 触发一场弹幕演出。
    DanmakuPerformance {
        /// Path to the `.performance.ron` file.
        ///
        /// `.performance.ron` 文件的路径。
        performance: String,
        /// World-space translation offset for the performance.
        ///
        /// 演出的世界空间位移偏移。
        #[serde(default)]
        translation: Option<(f32, f32)>,
    },
    /// Triggers an Alight Motion animation performance.
    ///
    /// 触发一场 Alight Motion 动画演出。
    AlightMotionPerformance {
        /// Path to the exported Alight Motion project file.
        ///
        /// 导出的 Alight Motion 项目文件路径。
        amproj_path: String,
        /// Optional configuration override path.
        ///
        /// 可选的配置覆盖路径。
        #[serde(default)]
        alight_motion_config: Option<String>,
        /// Whether to block the sequence until the animation finishes.
        ///
        /// 是否阻塞序列直至动画完成。
        #[serde(default = "default_true")]
        wait_for_completion: bool,
    },
    /// Animates properties of a view element using tweens.
    ///
    /// 使用补间动画更改视图元素的属性。
    SetViewElement {
        /// Selector for the target view element.
        ///
        /// 目标视图元素的解析器。
        selector: ElementSelector,
        /// Property to animate.
        ///
        /// 要播放动画的属性。
        target: TweenTarget,
        /// Duration of the tween. If omitted, applies instantly.
        ///
        /// 补间时长。如果省略，则立即应用。
        #[serde(default)]
        duration: Option<f32>,
        /// Easing function to use.
        ///
        /// 使用的缓动函数。
        #[serde(default)]
        easing: EaseKindRepr,
        /// Whether to block the sequence until the tween finishes.
        ///
        /// 是否阻塞序列直至补间完成。
        #[serde(default)]
        wait_for_completion: bool,
    },
    /// Blocks the sequence for a fixed duration.
    ///
    /// 将序列阻塞固定时长。
    Wait(f32),
    /// Nested linear sequence.
    ///
    /// 嵌套的线性序列。
    Sequence(Vec<Chapter>),
    /// Runs multiple chapters in parallel.
    ///
    /// 并行运行多个章节。
    Parallel(Vec<Chapter>),
    /// Controls player state or behavior.
    ///
    /// 控制玩家状态或行为。
    SetPlayer(PlayerAction),
    /// Controls global UI state.
    ///
    /// 控制全局 UI 状态。
    SetUI(UIAction),
    /// Modifies a view element's state or content.
    ///
    /// 修改视图元素的状态或内容。
    ModifyViewElement {
        /// Selector for the target view element.
        ///
        /// 目标视图元素的解析器。
        selector: ElementSelector,
        /// Modification to apply.
        ///
        /// 要应用的修改。
        modification: ElementModification,
    },
    /// Controls camera behavior or position.
    ///
    /// 控制相机行为或位置。
    SetCamera(CameraAction),
    /// Branching logic based on a FRE condition.
    ///
    /// 基于 FRE 条件的分支逻辑。
    Conditional {
        /// FRE condition to evaluate.
        ///
        /// 要评估的 FRE 条件。
        condition: FactCondition,
        /// Chapter to run if the condition is true.
        ///
        /// 条件为真时运行的章节。
        then_branch: Box<Chapter>,
        /// Optional chapter to run if the condition is false.
        ///
        /// 条件为假时运行的可选章节。
        #[serde(default)]
        else_branch: Option<Box<Chapter>>,
    },
    /// Multi-way branch based on a fact's value.
    ///
    /// 基于 fact 值的多路分支。
    FactSwitch {
        /// Fact key to check.
        ///
        /// 要检查的 fact 键。
        fact_key: String,
        /// List of cases matching values to chapters.
        ///
        /// 值与章节匹配的 case 列表。
        cases: Vec<(FactValueMatch, Chapter)>,
        /// Optional default chapter if no cases match.
        ///
        /// 无匹配 case 时运行的可选默认章节。
        #[serde(default)]
        default: Option<Box<Chapter>>,
    },
    /// Emits a FRE event.
    ///
    /// 触发一个 FRE 事件。
    EmitFactEvent {
        /// Event identifier.
        ///
        /// 事件标识符。
        event_id: String,
        /// Key-value data attached to the event.
        ///
        /// 附加到事件的键值数据。
        #[serde(default)]
        data: HashMap<String, String>,
    },
    /// Directly modifies one or more facts.
    ///
    /// 直接修改一个或多个 fact。
    ModifyFact {
        /// List of fact modifications.
        ///
        /// fact 修改操作列表。
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
    /// Spawn a headless entity with a WASM behavior attached.
    SpawnBehavior {
        behavior_id: String,
        #[serde(default)]
        context: Option<String>,
    },
    /// Repeat a sequence of chapters until a `Break` is encountered.
    Loop {
        body: Vec<Chapter>,
        #[serde(default)]
        max_iterations: Option<u32>,
    },
    /// Exit the innermost `Loop`.
    Break,
    /// Randomly select one or more chapters from a candidate list and execute them.
    RandomPick {
        candidates: Vec<Chapter>,
        #[serde(default = "default_one")]
        count: usize,
        #[serde(default)]
        allow_repeat: bool,
    },
    /// Select the next turn for an enemy from a named `turn_group`.
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

impl ElementSelector {
    /// Select an element by its full hierarchical name.
    ///
    /// 按完整层级名称选择元素。
    pub fn full(name: impl Into<String>) -> Self {
        Self::FullName(name.into())
    }

    /// Select an element by its local name.
    ///
    /// 按本地名称选择元素。
    pub fn local(name: impl Into<String>) -> Self {
        Self::LocalName(name.into())
    }

    /// Select elements by tag.
    ///
    /// 按标签选择元素。
    pub fn tag(tag: impl Into<String>) -> Self {
        Self::Tag(tag.into())
    }
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

impl ElementModification {
    /// Set element position.
    ///
    /// 设置元素位置。
    pub fn set_position(
        x: impl Into<Val<f32>>,
        y: impl Into<Val<f32>>,
        z: impl Into<Val<f32>>,
    ) -> Self {
        Self::SetPosition(x.into(), y.into(), z.into())
    }

    /// Set element scale.
    ///
    /// 设置元素缩放。
    pub fn set_scale(
        x: impl Into<Val<f32>>,
        y: impl Into<Val<f32>>,
        z: impl Into<Val<f32>>,
    ) -> Self {
        Self::SetScale(x.into(), y.into(), z.into())
    }

    /// Set element color.
    ///
    /// 设置元素颜色。
    pub fn set_color(
        red: impl Into<Val<f32>>,
        green: impl Into<Val<f32>>,
        blue: impl Into<Val<f32>>,
        alpha: impl Into<Val<f32>>,
    ) -> Self {
        Self::SetColor(red.into(), green.into(), blue.into(), alpha.into())
    }

    /// Set element visibility.
    ///
    /// 设置元素可见性。
    pub fn set_visibility(value: impl Into<Val<bool>>) -> Self {
        Self::SetVisibility(value.into())
    }

    /// Set element ViewBox size.
    ///
    /// 设置元素 ViewBox 尺寸。
    pub fn set_box_size(width: impl Into<Val<f32>>, height: impl Into<Val<f32>>) -> Self {
        Self::SetBoxSize(width.into(), height.into())
    }
}

/// Easing function representation (PascalCase, matches bevy_tween EaseKind).
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum EaseKindRepr {
    #[default]
    Linear,
    QuadIn,
    QuadOut,
    InOutQuad,
    CubicIn,
    CubicOut,
    InOutCubic,
    SineIn,
    SineOut,
    InOutSine,
    CircularIn,
    CircularOut,
    InOutCircular,
    ExpoIn,
    ExpoOut,
    InOutExpo,
    ElasticIn,
    ElasticOut,
    InOutElastic,
    BounceIn,
    BounceOut,
    InOutBounce,
    BackIn,
    BackOut,
    InOutBack,
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
    /// Set the ViewBox anchor for size-aware positioning.
    /// `(0, -1)` = bottom fixed, `(0, 1)` = top fixed, `(0, 0)` = centered (default).
    Anchor(f32, f32),
}

impl TweenTarget {
    /// Create a position tween target without an explicit source value.
    ///
    /// 创建不带显式起始值的位置补间目标。
    pub fn position(to: Vec3Tuple) -> Self {
        Self::Position { from: None, to }
    }

    /// Create a position tween target with an explicit source value.
    ///
    /// 创建带显式起始值的位置补间目标。
    pub fn position_from(from: Vec3Tuple, to: Vec3Tuple) -> Self {
        Self::Position {
            from: Some(from),
            to,
        }
    }

    /// Create a scale tween target without an explicit source value.
    ///
    /// 创建不带显式起始值的缩放补间目标。
    pub fn scale(to: Vec3Tuple) -> Self {
        Self::Scale { from: None, to }
    }

    /// Create a scale tween target with an explicit source value.
    ///
    /// 创建带显式起始值的缩放补间目标。
    pub fn scale_from(from: Vec3Tuple, to: Vec3Tuple) -> Self {
        Self::Scale {
            from: Some(from),
            to,
        }
    }

    /// Create a color tween target without an explicit source value.
    ///
    /// 创建不带显式起始值的颜色补间目标。
    pub fn color(to: ColorTuple) -> Self {
        Self::Color { from: None, to }
    }

    /// Create a color tween target with an explicit source value.
    ///
    /// 创建带显式起始值的颜色补间目标。
    pub fn color_from(from: ColorTuple, to: ColorTuple) -> Self {
        Self::Color {
            from: Some(from),
            to,
        }
    }

    /// Create a ViewBox size tween target without an explicit source value.
    ///
    /// 创建不带显式起始值的 ViewBox 尺寸补间目标。
    pub fn box_size(to: Vec2Tuple) -> Self {
        Self::BoxSize { from: None, to }
    }

    /// Create a ViewBox size tween target with an explicit source value.
    ///
    /// 创建带显式起始值的 ViewBox 尺寸补间目标。
    pub fn box_size_from(from: Vec2Tuple, to: Vec2Tuple) -> Self {
        Self::BoxSize {
            from: Some(from),
            to,
        }
    }

    /// Create a rotation tween target without an explicit source value.
    ///
    /// 创建不带显式起始值的旋转补间目标。
    pub fn rotation(to: impl Into<Val<f32>>) -> Self {
        Self::Rotation {
            from: None,
            to: to.into(),
        }
    }

    /// Create a rotation tween target with an explicit source value.
    ///
    /// 创建带显式起始值的旋转补间目标。
    pub fn rotation_from(from: impl Into<Val<f32>>, to: impl Into<Val<f32>>) -> Self {
        Self::Rotation {
            from: Some(from.into()),
            to: to.into(),
        }
    }

    /// Create an alpha tween target without an explicit source value.
    ///
    /// 创建不带显式起始值的透明度补间目标。
    pub fn alpha(to: impl Into<Val<f32>>) -> Self {
        Self::Alpha {
            from: None,
            to: to.into(),
        }
    }

    /// Create an alpha tween target with an explicit source value.
    ///
    /// 创建带显式起始值的透明度补间目标。
    pub fn alpha_from(from: impl Into<Val<f32>>, to: impl Into<Val<f32>>) -> Self {
        Self::Alpha {
            from: Some(from.into()),
            to: to.into(),
        }
    }
}

// ============================================================================
// Default helpers
// ============================================================================

fn default_true() -> bool {
    true
}

fn default_one() -> usize {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_split_battle_box_with_legacy_easing_alias() {
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

        let error = ron::from_str::<Chapter>(ron).expect_err("legacy easing alias should fail");
        assert!(
            error.to_string().contains("OutCubic"),
            "error should mention rejected legacy alias: {error}",
        );
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
