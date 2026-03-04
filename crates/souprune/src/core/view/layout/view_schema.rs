//! # view_schema.rs
//!
//! # view_schema.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines the pure data schema for View Layouts in RON files.
//! Using single-file format (`.view.ron`) containing:
//! - `roots`: Visual layout nodes
//! - `requires`: FRE data dependencies (optional)
//! - `facts`: Initial fact values (optional)
//!
//! 定义视图布局在 RON 文件中的纯数据 Schema。
//! 使用单文件格式（`.view.ron`）包含：
//! - `roots`: 视觉布局节点
//! - `requires`: FRE 数据依赖（可选）
//! - `facts`: 初始 fact 值（可选）

use super::serde_types::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// View Layout Asset - represents a complete view layout configuration.
/// Loaded from `.view.ron` files.
///
/// 视图布局资产 - 表示完整的视图布局配置。
/// 从 `.view.ron` 文件加载。
#[derive(Asset, TypePath, Debug, Deserialize, Serialize, Clone)]
pub struct ViewLayoutAsset {
    /// Root view nodes
    /// 根视图节点
    pub roots: Vec<ViewNodeDef>,

    /// Global triggers for state transitions.
    /// DEPRECATED: Use FRE rules with SwitchState action instead.
    ///
    /// 状态转换的全局触发器。
    /// 已弃用：请改用带有 SwitchState action 的 FRE 规则。
    #[serde(default)]
    pub global_triggers: Option<HashMap<String, Vec<GlobalTriggerRuleDef>>>,

    /// Data requirements for this View.
    ///
    /// 此 View 的数据需求声明。
    #[serde(default)]
    pub requires: Vec<DataRequirement>,

    /// Inline facts to set when this View is loaded.
    ///
    /// 加载此 View 时要设置的内联事实。
    #[serde(default)]
    pub facts: Option<HashMap<String, InitialFactValue>>,
}

/// Data requirement declaration for Views.
/// Specifies how to load external FRE data.
///
/// View 的数据需求声明。
/// 指定如何加载外部 FRE 数据。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DataRequirement {
    /// Load facts and rules from a FRE file.
    /// Example: `File("battle/fre/enemies/dummy.fre.ron")`
    ///
    /// 从 FRE 文件加载事实和规则。
    /// 示例：`File("battle/fre/enemies/dummy.fre.ron")`
    File(String),

    /// Declare an interface that must be bound externally.
    /// The binding is provided by SpawnView's `bindings` field.
    ///
    /// 声明必须由外部绑定的接口。
    /// 绑定由 SpawnView 的 `bindings` 字段提供。
    Interface {
        /// Interface name (used as key in bindings)
        /// 接口名称（在 bindings 中作为键使用）
        interface: String,
        /// Expected facts (for validation, optional)
        /// 预期的事实（用于验证，可选）
        #[serde(default)]
        expects: Vec<String>,
    },
}

/// Value type for initial facts in View Schema.
/// Supports int, float, bool, string, and array values.
///
/// View Schema 中初始事实的值类型。
/// 支持 int、float、bool、string 和数组值。
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum InitialFactValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    /// List of strings - useful for enemy names, etc.
    /// 字符串列表 - 用于敌人名称等。
    StringList(Vec<String>),
    /// List of integers - useful for HP values, etc.
    /// 整数列表 - 用于 HP 值等。
    IntList(Vec<i64>),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GlobalTriggerRuleDef {
    pub target_state: String,
    #[serde(default)]
    pub sound: Option<String>,
    #[serde(default)]
    pub allowed_states: Option<Vec<String>>,
}

/// View Node Definition - defines a single visual element in the view layout.
///
/// 视图节点定义 - 定义视图布局中的单个可视化元素。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ViewNodeDef {
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub style: StyleDef,
    /// Expression-based visibility control.
    /// Examples: "fact('depth') == 1", "$selection == 0", "true"
    ///
    /// 基于表达式的可见性控制。
    /// 示例: "fact('depth') == 1", "$selection == 0", "true"
    #[serde(default)]
    pub visible_when: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub background_color: Option<SerializableColor>,
    #[serde(default)]
    #[allow(dead_code)]
    pub border_color: Option<SerializableColor>,
    #[serde(default)]
    #[allow(dead_code)]
    pub image: Option<ImageDef>,
    #[serde(default)]
    pub sprite: Option<SpriteDef>,
    /// Data-driven state sprite configuration.
    /// Allows sprite textures to change based on rules and triggers.
    ///
    /// 数据驱动的状态精灵配置。
    /// 允许精灵纹理根据规则和触发器变化。
    #[serde(default)]
    pub state_sprite: Option<StateSpriteConfig>,
    #[serde(default)]
    pub texts: Vec<TextDef>,
    #[serde(default)]
    #[serde(alias = "view_box", alias = "ui_box_logic")]
    pub view_box: Option<ViewBoxLogicDef>,
    #[serde(default)]
    #[allow(dead_code)]
    pub children: Vec<ViewNodeDef>,
    /// If true, this UI node will be anchored to the camera and follow its movement.
    /// This is useful for HUD elements that should stay fixed on screen.
    /// Default is true for top-level nodes with view_box.
    #[serde(default = "default_camera_anchored")]
    pub camera_anchored: bool,
    /// Repeat configuration for generating multiple instances from an array.
    /// When present, this node will be spawned multiple times based on the array.
    ///
    /// 用于从数组生成多个实例的重复配置。
    /// 存在时，此节点将根据数组被多次生成。
    #[serde(default)]
    pub repeat: Option<RepeatDef>,
}

fn default_camera_anchored() -> bool {
    true
}

// ============================================================================
// Repeat Configuration (Dynamic UI Element Generation)
// 重复配置（动态 UI 元素生成）
// ============================================================================

/// Repeat configuration for generating multiple UI elements from an array.
/// Used for things like HP bars where each enemy needs its own visual element.
///
/// 用于从数组生成多个 UI 元素的重复配置。
/// 用于如血条这样每个敌人需要独立视觉元素的场景。
///
/// Example in RON:
/// ```ron
/// (
///     name: "EnemyHpBars",
///     repeat: (
///         source: "enemy_names",
///         index_var: "i",
///     ),
///     sprite: (
///         visual: "procedural://white_pixel",
///         transform: (
///             translation: (100.0, "50.0 - @i * 32.0", 1.0),
///             scale: ("80.0 * $enemy_hps[@i] / $enemy_hp_maxs[@i]", 12.0, 1.0),
///         ),
///     ),
/// )
/// ```
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepeatDef {
    /// Source array fact name (e.g., "enemy_names").
    /// The length of this array determines how many instances are created.
    ///
    /// 源数组 fact 名称（如 "enemy_names"）。
    /// 此数组的长度决定创建多少个实例。
    pub source: String,

    /// Optional limit on number of items to generate.
    ///
    /// 生成元素数量的可选限制。
    #[serde(default)]
    pub limit: Option<usize>,

    /// Index variable name for templates (default: "i").
    /// Use @i in expressions to reference current index.
    ///
    /// 模板中的索引变量名（默认："i"）。
    /// 在表达式中使用 @i 引用当前索引。
    #[serde(default)]
    pub index_var: Option<String>,

    /// Item variable name for templates (default: "item").
    /// Use @item in expressions to reference current array element value.
    ///
    /// 模板中的元素变量名（默认："item"）。
    /// 在表达式中使用 @item 引用当前数组元素值。
    #[serde(default)]
    pub item_var: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct StyleDef {
    #[serde(default)]
    pub width: Option<SerializableVal>,
    #[serde(default)]
    pub height: Option<SerializableVal>,
    #[serde(default)]
    pub left: Option<SerializableVal>,
    #[serde(default)]
    pub right: Option<SerializableVal>,
    #[serde(default)]
    pub top: Option<SerializableVal>,
    #[serde(default)]
    pub bottom: Option<SerializableVal>,
    #[serde(default)]
    pub position_type: Option<SerializablePositionType>,
    #[serde(default)]
    pub flex_direction: Option<UiFlexDirection>,
    #[serde(default)]
    pub justify_content: Option<SerializableJustifyContent>,
    #[serde(default)]
    pub align_items: Option<SerializableAlignItems>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UIVisibilityRuleDef {
    pub rule_type: String,
    #[serde(default)]
    pub layers: Option<Vec<String>>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageDef {
    pub path: String,
    #[serde(default)]
    pub color: Option<SerializableColor>,
}

/// Sprite definition for view layouts.
///
/// 视图布局中的精灵定义。
///
/// Uses `Visual` for automatic path resolution and type detection.
/// Rendering properties (color, flip, shader, etc.) are defined here.
///
/// 使用 `Visual` 进行自动路径解析和类型检测。
/// 渲染属性（颜色、翻转、着色器等）在此定义。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpriteDef {
    /// Visual resource path (supports shorthand and auto type detection).
    ///
    /// 视觉资源路径（支持简写和自动类型检测）。
    pub visual: crate::core::visual::Visual,

    /// Initial animation state for character animations.
    ///
    /// 角色动画的初始状态。
    #[serde(default)]
    pub initial_state: Option<String>,

    /// Color tint.
    ///
    /// 颜色叠加。
    #[serde(default)]
    pub color: Option<SerializableColor>,

    /// Horizontal flip.
    ///
    /// 水平翻转。
    #[serde(default)]
    pub flip_x: bool,

    /// Vertical flip.
    ///
    /// 垂直翻转。
    #[serde(default)]
    pub flip_y: bool,

    /// Transform (translation, scale, rotation).
    ///
    /// 变换（位移、缩放、旋转）。
    #[serde(default)]
    pub transform: Option<SerializableTransform>,

    /// Optional custom shader path for sprite material.
    ///
    /// 精灵材质的可选自定义着色器路径。
    #[serde(default)]
    pub custom_shader: Option<String>,

    /// Shader parameters passed via uniform data.
    ///
    /// 通过 uniform 数据传递的着色器参数。
    #[serde(default)]
    pub shader_params: Option<DynamicColor>,

    /// Pivot point (anchor).
    ///
    /// 锚点。
    #[serde(default)]
    pub pivot: Option<SerializableVec2>,

    /// Frame duration for frame animations (seconds).
    ///
    /// 帧动画的帧持续时间（秒）。
    #[serde(default)]
    pub frame_duration: Option<f32>,

    /// Expression-based visibility control.
    /// Examples: "fact('depth') == 1", "$selection == 0"
    ///
    /// 基于表达式的可见性控制。
    /// 示例: "fact('depth') == 1", "$selection == 0"
    #[serde(default)]
    pub visible_when: Option<String>,

    /// HP bar source configuration for HP bar sprites.
    /// When set, this sprite is treated as an HP bar and its shader params
    /// will be dynamically updated based on the specified HP source.
    ///
    /// HP 条来源配置。
    /// 设置后，此精灵将被视为 HP 条，其 shader 参数
    /// 将根据指定的 HP 来源动态更新。
    #[serde(default)]
    #[deprecated(note = "Use `material` field instead for dynamic shader support")]
    pub hp_bar_source: Option<HPBarSourceDef>,

    /// Dynamic material definition for custom shaders.
    /// Replaces old custom_shader + shader_params + hp_bar_source fields.
    ///
    /// 自定义着色器的动态材质定义。
    /// 替代旧的 custom_shader + shader_params + hp_bar_source 字段。
    #[serde(default)]
    pub material: Option<MaterialDef>,
}

/// HP bar source definition for configuring where HP values come from.
/// HP 条来源定义，用于配置 HP 值的来源。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum HPBarSourceDef {
    /// Player HP source - uses player_hp and player_hp_max facts.
    /// 玩家 HP 来源 - 使用 player_hp 和 player_hp_max facts。
    Player,
    /// Enemy HP source - uses enemy_hps and enemy_hp_maxs arrays.
    /// The index is resolved from the repeat context (@i variable).
    /// 敌人 HP 来源 - 使用 enemy_hps 和 enemy_hp_maxs 数组。
    /// 索引从 repeat 上下文（@i 变量）解析。
    Enemy,
    /// Custom HP source with expressions.
    /// 自定义 HP 来源（使用表达式）。
    Custom {
        hp_expr: String,
        hp_max_expr: String,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TextDef {
    pub id: String,
    #[serde(default)]
    pub content: Option<String>,
    pub font: ViewFontDef,
    pub world_scale: SerializableVec2,
    pub color: SerializableColor,
    pub transform: SerializableTransform,
    #[serde(default)]
    pub line_height: Option<f32>,
    #[serde(default)]
    pub conditional_style: Option<ConditionalStyleDef>,
    /// Expression-based visibility control.
    /// Examples: "fact('depth') == 1", "$selection == 0"
    ///
    /// 基于表达式的可见性控制。
    /// 示例: "fact('depth') == 1", "$selection == 0"
    #[serde(default)]
    pub visible_when: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConditionalStyleDef {
    pub condition: String,
    pub color: SerializableColor,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ViewBoxLogicDef {
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub border_width: f32,
    pub offset: SerializableVec3,
    #[serde(default)]
    pub fill_shader: Option<String>,
    #[serde(default)]
    pub structure_file: Option<String>,
    #[serde(default)]
    pub fill_color: Option<SerializableColor>,
}

// ============================================================================
// SdfStructure Asset Definition
// ============================================================================

#[derive(Asset, TypePath, Debug, Deserialize, Serialize, Clone)]
pub struct SdfStructureAsset {
    pub layer_count: usize,
    pub root: SdfLayerDef,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SdfLayerDef {
    pub name: String,
    pub sdf_type: SdfShapeKind,
    #[serde(default)]
    pub color_source: SdfColorSource,
    #[serde(default = "default_z_offset")]
    pub z_offset: f32,
    #[serde(default)]
    pub is_filler: bool,
    #[serde(default)]
    pub children: Vec<SdfLayerDef>,
}

fn default_z_offset() -> f32 {
    0.1
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum SdfShapeKind {
    Outer,
    Inner,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub enum SdfColorSource {
    #[default]
    FillColor,
    White,
    Custom(SerializableColor),
}

// ============================================================================
// State Sprite Configuration (Data-Driven State Management)
// 状态精灵配置（数据驱动的状态管理）
// ============================================================================

/// State-based sprite configuration.
/// Allows sprite textures to change based on rules (e.g., selection state).
///
/// 基于状态的精灵配置。
/// 允许精灵纹理根据规则（如选中状态）变化。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StateSpriteConfig {
    /// Default texture path (used when no state rule matches).
    ///
    /// 默认纹理路径（当没有状态规则匹配时使用）。
    pub default: String,

    /// Map of state names to texture paths.
    ///
    /// 状态名称到纹理路径的映射。
    #[serde(default)]
    pub variants: HashMap<String, String>,

    /// Rules that determine when to switch states.
    /// Evaluated in order; first matching rule wins.
    ///
    /// 决定何时切换状态的规则。
    /// 按顺序评估；第一个匹配的规则生效。
    #[serde(default)]
    pub rules: Vec<StateRuleDef>,

    /// Transform configuration for the sprite.
    ///
    /// 精灵的变换配置。
    #[serde(default)]
    pub transform: Option<SerializableTransform>,

    /// Expression-based visibility control.
    /// Examples: "fact('depth') == 1", "$selection == 0"
    ///
    /// 基于表达式的可见性控制。
    /// 示例: "fact('depth') == 1", "$selection == 0"
    #[serde(default)]
    pub visible_when: Option<String>,
}

/// A rule that triggers a state change.
///
/// 触发状态变化的规则。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StateRuleDef {
    /// The trigger condition.
    ///
    /// 触发条件。
    pub trigger: StateTriggerDef,

    /// The state to switch to when triggered.
    ///
    /// 触发时要切换到的状态。
    pub state: String,
}

/// Trigger types for state changes.
///
/// 状态变化的触发器类型。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum StateTriggerDef {
    /// Triggered when this element is selected in an interactive layer.
    /// Parameters: layer_id, index in selectable_elements
    ///
    /// 当此元素在交互层中被选中时触发。
    /// 参数：层ID, selectable_elements中的索引
    InteractiveLayerSelected {
        /// The layer ID to subscribe to.
        layer_id: String,
        /// The index in the layer's selectable_elements list.
        index: usize,
    },
    // Future extensions / 未来扩展:
    // PlayerHPBelow(u32),
    // PlayerHPAbove(u32),
    // GameEvent(String),
    // FactCondition { fact: String, value: String },
}

// ============================================================================
// Material Configuration (Dynamic Shader System)
// 材质配置（动态着色器系统）
// ============================================================================

/// Material definition for dynamic shader-based sprites.
/// Replaces old custom_shader + shader_params + hp_bar_source fields.
///
/// 动态着色器精灵的材质定义。
/// 替代旧的 custom_shader + shader_params + hp_bar_source 字段。
///
/// Example in RON:
/// ```ron
/// material: (
///     shader: "shared/shaders/hp_bar_sprite.wgsl",
///     params: {
///         "hp_ratio": Expr("$player_hp / $player_hp_max"),
///         "lag_ratio": Static(1.0),
///         "half_width": Expr("40.0 + ($player_hp_max - 20) * 95.0 / 79 / 2"),
///         "alpha": Static(1.0),
///     },
///     animations: (
///         lag: (
///             source: "hp_ratio",
///             target: "lag_ratio",
///             delay: 0.2,
///             duration: 0.4,
///             easing: OutCirc,
///         ),
///     ),
/// )
/// ```
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MaterialDef {
    /// Shader resource path.
    /// Can be project-relative (e.g., "shared/shaders/hp_bar.wgsl")
    /// or mod-relative (e.g., "mod://my_mod/shaders/effect.wgsl").
    ///
    /// 着色器资源路径。
    /// 可以是项目相对路径（如 "shared/shaders/hp_bar.wgsl"）
    /// 或 mod 相对路径（如 "mod://my_mod/shaders/effect.wgsl"）。
    pub shader: String,

    /// Shader parameters: name -> expression/static value.
    /// Maximum 8 parameters (params vec4 + extra_params vec4).
    ///
    /// 着色器参数：名称 -> 表达式/静态值。
    /// 最多 8 个参数（params vec4 + extra_params vec4）。
    #[serde(default)]
    pub params: HashMap<String, MaterialParamValue>,

    /// Animation configurations (optional).
    ///
    /// 动画配置（可选）。
    #[serde(default)]
    pub animations: Option<MaterialAnimationsDef>,

    /// Base texture path (optional).
    /// If not specified, uses procedural://white_pixel.
    ///
    /// 基础纹理路径（可选）。
    /// 如果未指定，使用 procedural://white_pixel。
    #[serde(default)]
    pub texture: Option<String>,
}

/// Material parameter value - can be static or expression-based.
///
/// 材质参数值 - 可以是静态值或基于表达式。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum MaterialParamValue {
    /// Static float value.
    ///
    /// 静态浮点值。
    Static(f32),

    /// Expression string that will be evaluated at runtime.
    /// Supports $fact_name variables and standard math operations.
    ///
    /// 在运行时评估的表达式字符串。
    /// 支持 $fact_name 变量和标准数学运算。
    Expr(String),
}

impl Default for MaterialParamValue {
    fn default() -> Self {
        Self::Static(0.0)
    }
}

/// Animation configurations for material parameters.
///
/// 材质参数的动画配置。
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MaterialAnimationsDef {
    /// Lag animation configuration.
    /// Creates a delayed, smoothed version of a source parameter.
    ///
    /// 延迟动画配置。
    /// 创建源参数的延迟、平滑版本。
    #[serde(default)]
    pub lag: Option<LagAnimationDef>,
}

/// Lag animation definition.
/// Smoothly follows a source parameter with delay.
///
/// 延迟动画定义。
/// 带延迟地平滑跟随源参数。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LagAnimationDef {
    /// Source parameter name to track.
    ///
    /// 要跟踪的源参数名。
    pub source: String,

    /// Target parameter name to write to.
    ///
    /// 要写入的目标参数名。
    pub target: String,

    /// Delay in seconds before animation starts.
    /// Default: 0.2
    ///
    /// 动画开始前的延迟（秒）。
    /// 默认：0.2
    #[serde(default = "default_lag_delay")]
    pub delay: f32,

    /// Animation duration in seconds.
    /// Default: 0.4
    ///
    /// 动画时长（秒）。
    /// 默认：0.4
    #[serde(default = "default_lag_duration")]
    pub duration: f32,

    /// Easing function.
    ///
    /// 缓动函数。
    #[serde(default)]
    pub easing: EasingDef,
}

fn default_lag_delay() -> f32 {
    0.2
}

fn default_lag_duration() -> f32 {
    0.4
}

/// Easing function definition for animations.
///
/// 动画的缓动函数定义。
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub enum EasingDef {
    #[default]
    Linear,
    InQuad,
    OutQuad,
    InOutQuad,
    InCubic,
    OutCubic,
    InOutCubic,
    InCirc,
    OutCirc,
    InOutCirc,
}
