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
use crate::core::view::components::InteractiveLayerDef;
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
    pub navigation: Option<HashMap<String, NavigationRuleDef>>,
    #[serde(default)]
    pub transitions: Option<HashMap<String, LayerTransitionsDef>>,
    #[serde(default)]
    pub global_triggers: Option<HashMap<String, Vec<GlobalTriggerRuleDef>>>,
    /// Interactive layer definitions for unified navigation.
    ///
    /// 用于统一导航的交互层定义。
    ///
    /// This is the new unified system that works for both OW and Battle.
    /// Format: { "layer_id": InteractiveLayerDef }
    ///
    /// 这是新的统一系统，适用于 OW 和 Battle。
    /// 格式: { "层ID": InteractiveLayerDef }
    #[serde(default)]
    pub interactive_layers: Option<HashMap<String, InteractiveLayerDef>>,
    /// The initial layer to activate when this layout is loaded.
    /// If not specified, the first key in interactive_layers will be used.
    ///
    /// 加载此布局时要激活的初始层。
    /// 如果未指定，将使用 interactive_layers 中的第一个键。
    #[serde(default)]
    pub initial_layer: Option<String>,
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
    #[serde(default)]
    pub visibility_rule: Option<UIVisibilityRuleDef>,
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
    #[serde(default)]
    pub texts: Vec<TextDef>,
    #[serde(default)]
    #[serde(alias = "cursor")]
    pub reactive_indicator: Option<ReactiveIndicatorDef>,
    #[serde(default)]
    #[serde(alias = "ui_box_logic")]
    pub ui_shape_logic: Option<UIBoxLogicDef>,
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

#[derive(Debug, Deserialize, Clone)]
pub struct SpriteDef {
    /// Path to texture or animation config file.
    pub path: String,
    /// If true, the path is treated as an animation config (.animation.ron).
    /// If false, it's treated as a static image.
    #[serde(default)]
    pub is_animation: bool,
    #[serde(default)]
    pub initial_state: Option<String>,
    #[serde(default)]
    pub color: Option<SerializableColor>,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
    #[serde(default)]
    pub transform: Option<SerializableTransform>,
    /// Optional custom shader path for sprite material.
    #[serde(default)]
    pub custom_shader: Option<String>,
    /// Shader parameters passed via uniform data.
    #[serde(default)]
    pub shader_params: Option<DynamicColor>,
    #[serde(default)]
    pub pivot: Option<SerializableVec2>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TextDef {
    pub id: String,
    #[serde(default)]
    pub content: Option<String>,
    pub font: UIFontDef,
    pub world_scale: SerializableVec2,
    pub color: SerializableColor,
    pub transform: SerializableTransform,
    #[serde(default)]
    pub line_height: Option<f32>,
    #[serde(default)]
    pub conditional_style: Option<ConditionalStyleDef>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ConditionalStyleDef {
    pub condition: String,
    pub color: SerializableColor,
}

/// Definition for a reactive indicator element in RON configuration.
/// This configures visual elements that respond to UI events such as selection changes.
///
/// RON 配置中响应式指示器元素的定义。
/// 配置响应选择变更等 UI 事件的视觉元素。
#[derive(Debug, Deserialize, Clone)]
pub struct ReactiveIndicatorDef {
    #[allow(dead_code)]
    pub sprite_path: String,
    #[serde(default)]
    pub default_translation: Option<ReactivePositionDef>,
    #[serde(default)]
    pub overrides: HashMap<String, ReactivePositionDef>,
    #[serde(default)]
    pub visibility_rule: Option<UIVisibilityRuleDef>,
    #[serde(default)]
    pub transform: Option<ReactiveTransformDef>,
}

/// Position calculation mode for reactive indicators.
///
/// 响应式指示器的位置计算模式。
#[derive(Debug, Deserialize, Clone)]
pub enum ReactivePositionDef {
    /// Fixed position regardless of selection index.
    ///
    /// 固定位置，不随选择索引变化。
    Static(SerializableVec3),

    /// Linear interpolation: origin + step * index.
    ///
    /// 线性插值：origin + step * index。
    Linear {
        origin: SerializableVec3,
        step: SerializableVec3,
    },

    /// Custom positions for each index.
    ///
    /// 每个索引的自定义位置。
    Custom { positions: Vec<SerializableVec3> },
}

/// Transform definition for reactive indicators.
///
/// 响应式指示器的变换定义。
#[derive(Debug, Deserialize, Clone)]
pub struct ReactiveTransformDef {
    #[serde(default)]
    pub translation: Option<SerializableVec3>,
    #[serde(default)]
    pub scale: Option<SerializableVec3>,
    #[serde(default)]
    pub rotation: Option<f32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UIBoxLogicDef {
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

#[derive(Debug, Deserialize, Clone)]
pub struct NavigationRuleDef {
    #[serde(default)]
    pub mappings: HashMap<String, isize>,
    #[serde(default)]
    pub looping: bool,
    #[serde(default)]
    pub min_index: Option<IndexBoundDef>,
    #[serde(default)]
    pub max_index: Option<IndexBoundDef>,
    #[serde(default)]
    pub sound_on_navigate: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum IndexBoundDef {
    Static(usize),
    Dynamic(String),
}

#[derive(Debug, Deserialize, Clone)]
pub struct LayerTransitionsDef {
    #[serde(default)]
    pub on_confirm: Option<Vec<TransitionRuleDef>>,
    #[serde(default)]
    pub on_cancel: Option<TransitionActionDef>,
    #[serde(default)]
    pub sound_on_confirm: Option<String>,
    #[serde(default)]
    pub sound_on_cancel: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TransitionRuleDef {
    #[serde(default)]
    pub condition: Option<String>,
    pub action: TransitionActionDef,
}

#[derive(Debug, Deserialize, Clone)]
pub enum TransitionActionDef {
    GotoLayer(String),
    PopState,
    PushState(String),
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
