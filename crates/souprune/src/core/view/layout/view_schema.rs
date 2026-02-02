//! # view_schema.rs
//!
//! # view_schema.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines the pure data schema for View Layouts in RON files (`.view_layout.ron`).
//! This module contains `ViewLayoutAsset` and related definitions that map directly to the configuration files.
//! It relies on `serde_types` for type conversions.
//!
//! 定义视图布局在 RON 文件 (`.view_layout.ron`) 中的纯数据 Schema。
//! 本模块包含 `ViewLayoutAsset` 及相关定义，直接映射到配置文件。
//! 它依赖 `serde_types` 进行类型转换。

use super::serde_types::*;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// View Layout Asset - represents a complete view layout configuration.
///
/// 视图布局资产 - 表示完整的视图布局配置。
#[derive(Asset, TypePath, Debug, Deserialize, Clone)]
pub struct ViewLayoutAsset {
    #[allow(dead_code)]
    pub version: u32,
    /// Root view nodes
    /// 根视图节点
    pub roots: Vec<ViewNodeDef>,
    #[serde(default)]
    pub global_triggers: Option<HashMap<String, Vec<GlobalTriggerRuleDef>>>,
    /// Initial facts to set when this View is loaded.
    /// These facts are stored in the ViewRoot's local_facts database.
    /// Format: { "key": value } where value can be int, float, bool, or string.
    ///
    /// 加载此 View 时要设置的初始事实。
    /// 这些事实存储在 ViewRoot 的 local_facts 数据库中。
    /// 格式: { "key": value }，其中 value 可以是 int、float、bool 或 string。
    #[serde(default)]
    pub initial_facts: Option<HashMap<String, InitialFactValue>>,
}

/// Value type for initial facts in View Schema.
/// Supports int, float, bool, and string values.
///
/// View Schema 中初始事实的值类型。
/// 支持 int、float、bool 和 string 值。
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum InitialFactValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Deserialize, Clone)]
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
#[derive(Debug, Deserialize, Clone)]
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
    #[serde(alias = "ui_box_logic")]
    pub ui_shape_logic: Option<ViewBoxLogicDef>,
    #[serde(default)]
    #[allow(dead_code)]
    pub children: Vec<ViewNodeDef>,
    /// If true, this UI node will be anchored to the camera and follow its movement.
    /// This is useful for HUD elements that should stay fixed on screen.
    /// Default is true for top-level nodes with ui_shape_logic.
    #[serde(default = "default_camera_anchored")]
    pub camera_anchored: bool,
}

fn default_camera_anchored() -> bool {
    true
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone, Default)]
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

#[derive(Debug, Deserialize, Clone)]
pub struct UIVisibilityRuleDef {
    pub rule_type: String,
    #[serde(default)]
    pub layers: Option<Vec<String>>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
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
#[derive(Debug, Deserialize, Clone)]
pub struct SpriteDef {
    /// Visual resource path (supports shorthand and auto type detection).
    /// Replaces the old `path` + `is_animation` pattern.
    ///
    /// 视觉资源路径（支持简写和自动类型检测）。
    /// 替代旧的 `path` + `is_animation` 模式。
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
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
pub struct ConditionalStyleDef {
    pub condition: String,
    pub color: SerializableColor,
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Asset, TypePath, Debug, Deserialize, Clone)]
pub struct SdfStructureAsset {
    pub layer_count: usize,
    pub root: SdfLayerDef,
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
pub enum SdfShapeKind {
    Outer,
    Inner,
}

#[derive(Debug, Deserialize, Clone, Default)]
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
#[derive(Debug, Deserialize, Clone)]
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
#[derive(Debug, Deserialize, Clone)]
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
#[derive(Debug, Deserialize, Clone)]
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
