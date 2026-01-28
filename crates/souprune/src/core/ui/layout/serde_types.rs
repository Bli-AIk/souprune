//! # serde_types.rs
//!
//! # serde_types.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Helper types for Serde serialization/deserialization used in UI schemas.
//! This module handles the conversion between simple RON data types (like `SerializableColor`)
//! and Bevy's runtime types (like `Color`, `Val`).
//!
//! 用于 UI Schema 的 Serde 序列化/反序列化辅助类型。
//! 本模块处理简单 RON 数据类型（如 `SerializableColor`）与 Bevy 运行时类型（如 `Color`, `Val`）之间的转换。

use bevy::prelude::*;
use bevy::ui::{AlignItems, FlexDirection, JustifyContent, PositionType, Val};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
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
#[serde(untagged)]
pub enum FloatOrExpr {
    Static(f32),
    Dynamic(String),
}

impl FloatOrExpr {
    pub fn as_float(&self) -> f32 {
        match self {
            FloatOrExpr::Static(v) => *v,
            FloatOrExpr::Dynamic(_) => 0.0,
        }
    }

    pub fn as_expr(&self) -> Option<&String> {
        match self {
            FloatOrExpr::Dynamic(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_dynamic(&self) -> bool {
        matches!(self, FloatOrExpr::Dynamic(_))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DynamicColor {
    pub r: FloatOrExpr,
    pub g: FloatOrExpr,
    pub b: FloatOrExpr,
    pub a: FloatOrExpr,
}

impl DynamicColor {
    pub fn to_static_color(&self) -> Color {
        Color::srgba(
            self.r.as_float(),
            self.g.as_float(),
            self.b.as_float(),
            self.a.as_float(),
        )
    }

    pub fn is_dynamic(&self) -> bool {
        self.r.is_dynamic() || self.g.is_dynamic() || self.b.is_dynamic() || self.a.is_dynamic()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SerializableVec3 {
    pub x: FloatOrExpr,
    pub y: FloatOrExpr,
    pub z: FloatOrExpr,
}

impl SerializableVec3 {
    pub fn to_static_vec3(&self) -> Vec3 {
        Vec3::new(self.x.as_float(), self.y.as_float(), self.z.as_float())
    }

    pub fn is_dynamic(&self) -> bool {
        self.x.is_dynamic() || self.y.is_dynamic() || self.z.is_dynamic()
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
pub enum UIFontDef {
    DeterminationMono,
    DeterminationSans,
    Hud,
    BattleHud,
}

impl From<UIFontDef> for crate::core::ui::components::UIFont {
    fn from(val: UIFontDef) -> Self {
        match val {
            UIFontDef::DeterminationMono => crate::core::ui::components::UIFont::DeterminationMono,
            UIFontDef::DeterminationSans => crate::core::ui::components::UIFont::DeterminationSans,
            UIFontDef::Hud => crate::core::ui::components::UIFont::Hud,
            UIFontDef::BattleHud => crate::core::ui::components::UIFont::BattleHud,
        }
    }
}
