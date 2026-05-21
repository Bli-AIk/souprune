//! Visual child definitions for runtime View schema assets.
//!
//! 运行时 View schema 资源的视觉子定义。

use super::super::serde_types::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// Toggle between two colors based on a boolean FRE fact.
    /// `on` is used when the fact is truthy; `off` is used otherwise.
    ///
    /// 根据布尔 FRE fact 在两种颜色间切换。
    /// fact 为真值时使用 `on`；否则使用 `off`。
    FactToggle {
        key: String,
        on: SerializableColor,
        off: SerializableColor,
    },
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
/// Replaces old custom_shader + shader_params fields.
///
/// 动态着色器精灵的材质定义。
/// 替代旧的 custom_shader + shader_params 字段。
///
/// Example in RON:
/// ```ron
/// material: (
///     shader: "assets/shaders/hp_bar_sprite.wgsl",
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
    /// Can be project-relative (e.g., "assets/shaders/health_bar.wgsl")
    /// or mod-relative (e.g., "mod://my_mod/shaders/effect.wgsl").
    ///
    /// 着色器资源路径。
    /// 可以是项目相对路径（如 "assets/shaders/health_bar.wgsl"）
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
