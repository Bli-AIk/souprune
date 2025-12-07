//! # RON UI Layout Documentation / RON UI 布局文档
//!
//! This module defines the structure of the RON UI layout files used in SoupRune.
//!
//! 本模块定义了 SoupRune 中使用的 RON UI 布局文件的结构。
//!
//! ---
//!
//! ## File Structure / 文件结构
//!
//! ```ron
//! (
//!     version: 1,
//!     roots: [
//!         (
//!             name: "MyNode",
//!             // Node properties... / 节点属性...
//!         )
//!     ]
//! )
//! ```
//!
//! ---
//!
//! ## Node Properties / 节点属性
//!
//! ### Text Transform / 文本变换
//!
//! Texts use a `transform` field to define position, rotation, and scale.
//!
//! 文本使用 `transform` 字段定义位置、旋转和缩放。
//!
//! ```ron
//! transform: (
//!     translation: (x: 10.0, y: 20.0, z: 1.0), // Required / 必须
//!     rotation: Some(45.0),                    // Optional (degrees) / 可选（角度）
//!     scale: Some((x: 1.5, y: 1.5, z: 1.0)),   // Optional / 可选
//! )
//! ```
//!
//! ### Cursor Configuration / 光标配置
//!
//! Cursors can be positioned dynamically using `default_translation` or statically using `transform`.
//!
//! 光标可以使用 `default_translation` 进行动态定位，或使用 `transform` 进行静态定位。
//!
//! ```ron
//! cursor: Some((
//!     sprite_path: "common/heartsmall",
//!
//!     // Mode 1: Dynamic Logic / 模式 1：动态逻辑
//!     // Used for menus where the cursor moves between options.
//!     // 用于光标在选项之间移动的菜单。
//!     default_translation: Some(Linear(
//!         origin: (x: 0.0, y: 0.0, z: 0.0),
//!         step: (x: 0.0, y: -20.0, z: 0.0),
//!     )),
//!
//!     // Mode 2: Static Transform / 模式 2：静态变换
//!     transform: Some((
//!         // Used if default_translation is missing.
//!         // 当 default_translation 缺失时使用。
//!         translation: Some((x: 10.0, y: 10.0, z: 2.0)),
//!
//!         rotation: Some(0.0),
//!         scale: Some((x: 1.0, y: 1.0, z: 1.0)),
//!     )),
//! ))
//! ```
//!
//! ### Conditional Styles / 条件样式
//!
//! Change text color based on game state.
//!
//! 根据游戏状态改变文本颜色。
//!
//! ```ron
//! conditional_style: Some((
//!     condition: "player.inventory.is_empty",
//!     color: (r: 0.5, g: 0.5, b: 0.5, a: 1.0),
//! ))
//! ```
//!
//! ### Variable Substitution / 变量替换
//!
//! - `{{key}}`: Look up in Mortar string table. / 在 Mortar 字符串表中查找。
//! - `{@path}`: Look up in PlayerData. / 在 PlayerData 中查找 (e.g., `{@player.hp}`).

use bevy::prelude::*;
use bevy::ui::{AlignItems, FlexDirection, JustifyContent, PositionType, Val};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum UiFlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl From<UiFlexDirection> for FlexDirection {
    fn from(val: UiFlexDirection) -> Self {
        match val {
            UiFlexDirection::Row => FlexDirection::Row,
            UiFlexDirection::Column => FlexDirection::Column,
            UiFlexDirection::RowReverse => FlexDirection::RowReverse,
            UiFlexDirection::ColumnReverse => FlexDirection::ColumnReverse,
        }
    }
}


#[derive(Asset, TypePath, Debug, Deserialize, Clone)]
pub struct UILayoutAsset {
    pub version: u32,
    pub roots: Vec<UINodeDef>,
    #[serde(default)]
    pub navigation: Option<HashMap<String, NavigationRuleDef>>,
    #[serde(default)]
    pub transitions: Option<HashMap<String, LayerTransitionsDef>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UINodeDef {
    pub name: String,
    #[serde(default)]
    pub style: StyleDef,
    #[serde(default)]
    pub visibility_rule: Option<UIVisibilityRuleDef>,
    #[serde(default)]
    pub background_color: Option<SerializableColor>,
    #[serde(default)]
    pub border_color: Option<SerializableColor>,
    #[serde(default)]
    pub image: Option<ImageDef>,
    #[serde(default)]
    pub texts: Vec<TextDef>,
    #[serde(default)]
    pub cursor: Option<CursorDef>,
    #[serde(default)]
    pub ui_box_logic: Option<UIBoxLogicDef>,
    #[serde(default)]
    pub children: Vec<UINodeDef>,
}

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
pub enum SerializableVal {
    Auto,
    Px(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
}

impl From<SerializableVal> for Val {
    fn from(val: SerializableVal) -> Self {
        match val {
            SerializableVal::Auto => Val::Auto,
            SerializableVal::Px(v) => Val::Px(v),
            SerializableVal::Percent(v) => Val::Percent(v),
            SerializableVal::Vw(v) => Val::Vw(v),
            SerializableVal::Vh(v) => Val::Vh(v),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub enum SerializablePositionType {
    Relative,
    Absolute,
}

impl From<SerializablePositionType> for PositionType {
    fn from(val: SerializablePositionType) -> Self {
        match val {
            SerializablePositionType::Relative => PositionType::Relative,
            SerializablePositionType::Absolute => PositionType::Absolute,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub enum SerializableJustifyContent {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl From<SerializableJustifyContent> for JustifyContent {
    fn from(val: SerializableJustifyContent) -> Self {
        match val {
            SerializableJustifyContent::Start => JustifyContent::Start,
            SerializableJustifyContent::End => JustifyContent::End,
            SerializableJustifyContent::Center => JustifyContent::Center,
            SerializableJustifyContent::SpaceBetween => JustifyContent::SpaceBetween,
            SerializableJustifyContent::SpaceAround => JustifyContent::SpaceAround,
            SerializableJustifyContent::SpaceEvenly => JustifyContent::SpaceEvenly,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub enum SerializableAlignItems {
    Start,
    End,
    Center,
    Baseline,
    Stretch,
}

impl From<SerializableAlignItems> for AlignItems {
    fn from(val: SerializableAlignItems) -> Self {
        match val {
            SerializableAlignItems::Start => AlignItems::Start,
            SerializableAlignItems::End => AlignItems::End,
            SerializableAlignItems::Center => AlignItems::Center,
            SerializableAlignItems::Baseline => AlignItems::Baseline,
            SerializableAlignItems::Stretch => AlignItems::Stretch,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SerializableColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<SerializableColor> for Color {
    fn from(val: SerializableColor) -> Self {
        Color::srgba(val.r, val.g, val.b, val.a)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct UIVisibilityRuleDef {
    pub rule_type: String,
    #[serde(default)]
    pub layers: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ImageDef {
    pub path: String,
    #[serde(default)]
    pub color: Option<SerializableColor>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum UIFontDef {
    DeterminationMono,
    DeterminationSans,
    Hud,
}

impl From<UIFontDef> for super::components::UIFont {
    fn from(val: UIFontDef) -> Self {
        match val {
            UIFontDef::DeterminationMono => super::components::UIFont::DeterminationMono,
            UIFontDef::DeterminationSans => super::components::UIFont::DeterminationSans,
            UIFontDef::Hud => super::components::UIFont::Hud,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SerializableVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<SerializableVec3> for Vec3 {
    fn from(val: SerializableVec3) -> Self {
        Vec3::new(val.x, val.y, val.z)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SerializableVec2 {
    pub x: f32,
    pub y: f32,
}

impl From<SerializableVec2> for Vec2 {
    fn from(val: SerializableVec2) -> Self {
        Vec2::new(val.x, val.y)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SerializableTransform {
    pub translation: SerializableVec3,
    #[serde(default)]
    pub rotation: Option<f32>,
    #[serde(default)]
    pub scale: Option<SerializableVec3>,
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

#[derive(Debug, Deserialize, Clone)]
pub struct CursorDef {
    pub sprite_path: String,
    #[serde(default)]
    pub default_translation: Option<BoxCursorPositionDef>,
    #[serde(default)]
    pub overrides: HashMap<String, BoxCursorPositionDef>,
    #[serde(default)]
    pub visibility_rule: Option<UIVisibilityRuleDef>,
    #[serde(default)]
    pub transform: Option<CursorTransformDef>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum BoxCursorPositionDef {
    Static(SerializableVec3),
    Linear {
        origin: SerializableVec3,
        step: SerializableVec3,
    },
    Custom {
        positions: Vec<SerializableVec3>,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub struct CursorTransformDef {
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
    pub border_width: f32,
    pub offset: SerializableVec3,
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
