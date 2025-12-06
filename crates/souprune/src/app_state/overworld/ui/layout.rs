use bevy::prelude::*;
use bevy::ui::{AlignItems, FlexDirection, JustifyContent, PositionType, Val};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum UiFlexDirection {
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

impl Default for UiFlexDirection {
    fn default() -> Self {
        UiFlexDirection::Row
    }
}

#[derive(Asset, TypePath, Debug, Deserialize, Clone)]
pub struct UILayoutAsset {
    pub version: u32,
    pub roots: Vec<UINodeDef>,
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
pub struct TextDef {
    pub id: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub localization_key: Option<String>,
    pub font: UIFontDef,
    pub world_scale: SerializableVec2,
    pub color: SerializableColor,
    pub transform: SerializableVec3,
    #[serde(default)]
    pub line_height: Option<f32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CursorDef {
    pub sprite_path: String,
    pub default_position: BoxCursorPositionDef,
    #[serde(default)]
    pub overrides: HashMap<String, BoxCursorPositionDef>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum BoxCursorPositionDef {
    Static(SerializableVec3),
    Linear { origin: SerializableVec3, step: SerializableVec3 },
    Custom { positions: Vec<SerializableVec3> },
}

#[derive(Debug, Deserialize, Clone)]
pub struct UIBoxLogicDef {
    pub width: f32,
    pub height: f32,
    pub border_width: f32,
    pub offset: SerializableVec3,
}
