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
use bevy::ui::Val as BevyVal;
use bevy::ui::{AlignItems, FlexDirection, JustifyContent, PositionType};
use serde::Deserialize;

pub use crate::core::sequencer::chapter_schema::{ColorTuple, Val, Vec2Tuple, Vec3Tuple};

pub type FloatOrExpr = Val<f32>;

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

impl From<SerializableVal> for BevyVal {
    fn from(val: SerializableVal) -> Self {
        match val {
            SerializableVal::Auto => BevyVal::Auto,
            SerializableVal::Px(v) => BevyVal::Px(v),
            SerializableVal::Percent(v) => BevyVal::Percent(v),
            SerializableVal::Vw(v) => BevyVal::Vw(v),
            SerializableVal::Vh(v) => BevyVal::Vh(v),
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

pub type DynamicColor = ColorTuple;
pub type SerializableVec3 = Vec3Tuple;
pub type SerializableVec2 = Vec2Tuple;
pub type SerializableColor = ColorTuple;

pub fn serializable_color_to_color(color: &SerializableColor) -> Color {
    Color::srgba(
        match &color.0 {
            Val::Static(v) => *v,
            Val::Expr(_) => 0.0,
        },
        match &color.1 {
            Val::Static(v) => *v,
            Val::Expr(_) => 0.0,
        },
        match &color.2 {
            Val::Static(v) => *v,
            Val::Expr(_) => 0.0,
        },
        match &color.3 {
            Val::Static(v) => *v,
            Val::Expr(_) => 0.0,
        },
    )
}

pub fn serializable_vec2_to_vec2(vec: &SerializableVec2) -> Vec2 {
    Vec2::new(
        match &vec.0 {
            Val::Static(v) => *v,
            Val::Expr(_) => 0.0,
        },
        match &vec.1 {
            Val::Static(v) => *v,
            Val::Expr(_) => 0.0,
        },
    )
}

pub fn color_tuple_to_static(color: &ColorTuple) -> (f32, f32, f32, f32) {
    (
        match &color.0 {
            Val::Static(v) => *v,
            Val::Expr(_) => 1.0,
        },
        match &color.1 {
            Val::Static(v) => *v,
            Val::Expr(_) => 1.0,
        },
        match &color.2 {
            Val::Static(v) => *v,
            Val::Expr(_) => 1.0,
        },
        match &color.3 {
            Val::Static(v) => *v,
            Val::Expr(_) => 1.0,
        },
    )
}

pub fn vec2_tuple_to_static(vec: &Vec2Tuple) -> (f32, f32) {
    (
        match &vec.0 {
            Val::Static(v) => *v,
            Val::Expr(_) => 0.5,
        },
        match &vec.1 {
            Val::Static(v) => *v,
            Val::Expr(_) => 0.5,
        },
    )
}

pub fn dynamic_color_to_static(color: &DynamicColor) -> Color {
    Color::srgba(
        match &color.0 {
            Val::Static(v) => *v,
            Val::Expr(_) => 0.0,
        },
        match &color.1 {
            Val::Static(v) => *v,
            Val::Expr(_) => 0.0,
        },
        match &color.2 {
            Val::Static(v) => *v,
            Val::Expr(_) => 0.0,
        },
        match &color.3 {
            Val::Static(v) => *v,
            Val::Expr(_) => 0.0,
        },
    )
}

pub fn is_dynamic_color(color: &DynamicColor) -> bool {
    color.0.is_expr() || color.1.is_expr() || color.2.is_expr() || color.3.is_expr()
}

pub fn serializable_vec3_to_static(vec: &SerializableVec3) -> Vec3 {
    Vec3::new(
        match &vec.0 {
            Val::Static(v) => *v,
            Val::Expr(_) => 0.0,
        },
        match &vec.1 {
            Val::Static(v) => *v,
            Val::Expr(_) => 0.0,
        },
        match &vec.2 {
            Val::Static(v) => *v,
            Val::Expr(_) => 0.0,
        },
    )
}

pub fn is_dynamic_vec3(vec: &SerializableVec3) -> bool {
    vec.0.is_expr() || vec.1.is_expr() || vec.2.is_expr()
}

#[derive(Debug, Deserialize, Clone)]
pub struct SerializableTransform {
    #[serde(default)]
    pub translation: Option<SerializableVec3>,
    #[serde(default)]
    pub rotation: Option<f32>,
    #[serde(default)]
    pub scale: Option<SerializableVec3>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum ViewFontDef {
    DeterminationMono,
    DeterminationSans,
    Hud,
    BattleHud,
}

impl From<ViewFontDef> for crate::core::view::components::ViewFont {
    fn from(val: ViewFontDef) -> Self {
        match val {
            ViewFontDef::DeterminationMono => {
                crate::core::view::components::ViewFont::DeterminationMono
            }
            ViewFontDef::DeterminationSans => {
                crate::core::view::components::ViewFont::DeterminationSans
            }
            ViewFontDef::Hud => crate::core::view::components::ViewFont::Hud,
            ViewFontDef::BattleHud => crate::core::view::components::ViewFont::BattleHud,
        }
    }
}
