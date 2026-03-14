//! # view.rs
//!
//! View Layout schema types for `.view.ron` and `.sdf.ron` files.
//! Mirrors souprune's view_schema.rs types without Bevy dependency.
//!
//! `.view.ron` 和 `.sdf.ron` 文件的 View Layout Schema 类型。
//! 对应 souprune 的 view_schema.rs 类型，无 Bevy 依赖。

use crate::val::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Serializable Helper Types (mirrors serde_types.rs)
// ============================================================================

pub type FloatOrExpr = Val<f32>;
pub type DynamicColor = ColorTuple;
pub type SerializableVec3 = Vec3Tuple;
pub type SerializableVec2 = Vec2Tuple;
pub type SerializableColor = ColorTuple;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SerializableTransform {
    #[serde(default)]
    pub translation: Option<SerializableVec3>,
    #[serde(default)]
    pub rotation: Option<f32>,
    #[serde(default)]
    pub scale: Option<SerializableVec3>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum SerializableVal {
    Auto,
    Px(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiFlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum SerializablePositionType {
    Relative,
    Absolute,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum SerializableJustifyContent {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum SerializableAlignItems {
    Start,
    End,
    Center,
    Baseline,
    Stretch,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ViewFontDef {
    DeterminationMono,
    DeterminationSans,
    Hud,
    BattleHud,
}

/// Visual resource path (transparent string wrapper).
///
/// 视觉资源路径（透明字符串包装）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Visual(pub String);

// ============================================================================
// ViewLayout (top-level asset, mirrors ViewLayoutAsset)
// ============================================================================

/// View Layout — the top-level schema for `.view.ron` files.
///
/// 视图布局——`.view.ron` 文件的顶层 Schema。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ViewLayout {
    pub roots: Vec<ViewNodeDef>,

    #[serde(default)]
    pub global_triggers: Option<HashMap<String, Vec<GlobalTriggerRuleDef>>>,

    #[serde(default)]
    pub requires: Vec<DataRequirement>,

    #[serde(default)]
    pub facts: Option<HashMap<String, InitialFactValue>>,

    #[serde(default)]
    pub world_space: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DataRequirement {
    File(String),
    Interface {
        interface: String,
        #[serde(default)]
        expects: Vec<String>,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum InitialFactValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    StringList(Vec<String>),
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

// ============================================================================
// ViewNodeDef
// ============================================================================

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ViewNodeDef {
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub style: StyleDef,
    #[serde(default)]
    pub visible_when: Option<String>,
    #[serde(default)]
    pub background_color: Option<SerializableColor>,
    #[serde(default)]
    pub border_color: Option<SerializableColor>,
    #[serde(default)]
    pub image: Option<ImageDef>,
    #[serde(default)]
    pub sprite: Option<SpriteDef>,
    #[serde(default)]
    pub state_sprite: Option<StateSpriteConfig>,
    #[serde(default)]
    pub texts: Vec<TextDef>,
    #[serde(default)]
    #[serde(alias = "view_box", alias = "ui_box_logic")]
    pub view_box: Option<ViewBoxLogicDef>,
    #[serde(default)]
    pub children: Vec<ViewNodeDef>,
    #[serde(default)]
    pub repeat: Option<RepeatDef>,
}

// ============================================================================
// Child types
// ============================================================================

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
pub struct ImageDef {
    pub path: String,
    #[serde(default)]
    pub color: Option<SerializableColor>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpriteDef {
    pub visual: Visual,
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
    #[serde(default)]
    pub custom_shader: Option<String>,
    #[serde(default)]
    pub shader_params: Option<DynamicColor>,
    #[serde(default)]
    pub pivot: Option<SerializableVec2>,
    #[serde(default)]
    pub frame_duration: Option<f32>,
    #[serde(default)]
    pub visible_when: Option<String>,
    #[serde(default)]
    pub health_bar_source: Option<HealthBarSourceDef>,
    #[serde(default)]
    pub material: Option<MaterialDef>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum HealthBarSourceDef {
    Player,
    Enemy,
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepeatDef {
    pub source: String,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub index_var: Option<String>,
    #[serde(default)]
    pub item_var: Option<String>,
}

// ============================================================================
// State Sprite Configuration
// ============================================================================

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StateSpriteConfig {
    pub default: String,
    #[serde(default)]
    pub variants: HashMap<String, String>,
    #[serde(default)]
    pub rules: Vec<StateRuleDef>,
    #[serde(default)]
    pub transform: Option<SerializableTransform>,
    #[serde(default)]
    pub visible_when: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StateRuleDef {
    pub trigger: StateTriggerDef,
    pub state: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum StateTriggerDef {
    InteractiveLayerSelected { layer_id: String, index: usize },
}

// ============================================================================
// Material Configuration
// ============================================================================

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MaterialDef {
    pub shader: String,
    #[serde(default)]
    pub params: HashMap<String, MaterialParamValue>,
    #[serde(default)]
    pub animations: Option<MaterialAnimationsDef>,
    #[serde(default)]
    pub texture: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum MaterialParamValue {
    Static(f32),
    Expr(String),
}

impl Default for MaterialParamValue {
    fn default() -> Self {
        Self::Static(0.0)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MaterialAnimationsDef {
    #[serde(default)]
    pub lag: Option<LagAnimationDef>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LagAnimationDef {
    pub source: String,
    pub target: String,
    #[serde(default = "default_lag_delay")]
    pub delay: f32,
    #[serde(default = "default_lag_duration")]
    pub duration: f32,
    #[serde(default)]
    pub easing: EasingDef,
}

fn default_lag_delay() -> f32 {
    0.2
}

fn default_lag_duration() -> f32 {
    0.4
}

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

// ============================================================================
// SDF Structure Asset
// ============================================================================

/// SDF structure layout for `.sdf.ron` files.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SdfStructure {
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
// UI Visibility Rule (legacy)
// ============================================================================

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UIVisibilityRuleDef {
    pub rule_type: String,
    #[serde(default)]
    pub layers: Option<Vec<String>>,
}
