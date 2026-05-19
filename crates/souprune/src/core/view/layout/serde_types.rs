//! # serde_types.rs
//!
//! # serde_types.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Helper types for Serde serialization/deserialization used in View schemas.
//! This module handles the conversion between simple RON data types (like `SerializableColor`)
//! and Bevy's runtime types (like `Color`, `Val`).
//!
//! 用于 View Schema 的 Serde 序列化/反序列化辅助类型。
//! 本模块处理简单 RON 数据类型（如 `SerializableColor`）与 Bevy 运行时类型（如 `Color`, `Val`）之间的转换。

use bevy::prelude::*;
#[cfg(feature = "debug")]
use bevy::reflect::Reflect;
use bevy::ui::Val as BevyVal;
use bevy::ui::{
    AlignItems, AlignSelf, Display as BevyDisplay, FlexDirection, JustifyContent, PositionType,
};
use bevy_bitmap_text::{TextAlign, TextAnchor};
use serde::{Deserialize, Serialize};

pub use crate::core::sequencer::chapter_schema::{ColorTuple, Value, Vec2Tuple, Vec3Tuple};

pub type FloatOrExpr = Value<f32>;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "debug", derive(Reflect))]
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

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "debug", derive(Reflect))]
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

/// Four-sided layout value rectangle.
///
/// 四边布局值矩形。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct SerializableRect {
    /// Left side value.
    ///
    /// 左侧值。
    pub left: SerializableVal,
    /// Right side value.
    ///
    /// 右侧值。
    pub right: SerializableVal,
    /// Top side value.
    ///
    /// 上侧值。
    pub top: SerializableVal,
    /// Bottom side value.
    ///
    /// 下侧值。
    pub bottom: SerializableVal,
}

/// Two-axis layout gap values.
///
/// 双轴布局间距值。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct StyleGap {
    /// Row-axis gap value.
    ///
    /// 行轴间距值。
    pub row: SerializableVal,
    /// Column-axis gap value.
    ///
    /// 列轴间距值。
    pub column: SerializableVal,
}

/// Per-axis View sizing mode.
///
/// 单轴 View 尺寸模式。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum ViewSizeAxisDef {
    /// Use measured content size.
    ///
    /// 使用内容测量尺寸。
    Fit,
    /// Fill available parent space on the flex axis.
    ///
    /// 在 Flex 轴上填充父级可用空间。
    Fill,
    /// Use a fixed serialized length value.
    ///
    /// 使用固定的序列化长度值。
    Fixed(SerializableVal),
}

/// Author-facing View sizing shorthand.
///
/// 面向作者的 View 尺寸简写。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum ViewSizingDef {
    /// Both axes use measured content size.
    ///
    /// 两个轴都使用内容测量尺寸。
    Fit,
    /// Both axes try to fill available space.
    ///
    /// 两个轴都尝试填充可用空间。
    Fill,
    /// Both axes use fixed values.
    ///
    /// 两个轴都使用固定值。
    Fixed {
        /// Fixed width.
        ///
        /// 固定宽度。
        width: SerializableVal,
        /// Fixed height.
        ///
        /// 固定高度。
        height: SerializableVal,
    },
    /// Configure each axis separately.
    ///
    /// 分别配置每个轴。
    Axes {
        /// Width sizing mode.
        ///
        /// 宽度尺寸模式。
        width: ViewSizeAxisDef,
        /// Height sizing mode.
        ///
        /// 高度尺寸模式。
        height: ViewSizeAxisDef,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Reflect))]
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

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Reflect))]
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

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Reflect))]
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

/// Per-node alignment override inside the parent layout.
///
/// 父布局内的单节点对齐覆盖。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub enum SerializableAlignSelf {
    /// Use the parent alignment.
    ///
    /// 使用父级对齐方式。
    Auto,
    /// Align to the start edge.
    ///
    /// 对齐到起始边。
    Start,
    /// Align to the end edge.
    ///
    /// 对齐到结束边。
    End,
    /// Center on the cross axis.
    ///
    /// 在交叉轴居中。
    Center,
    /// Align text baselines.
    ///
    /// 对齐文本基线。
    Baseline,
    /// Stretch to fill the available cross-axis space.
    ///
    /// 拉伸以填充可用交叉轴空间。
    Stretch,
}

impl From<SerializableAlignSelf> for AlignSelf {
    fn from(val: SerializableAlignSelf) -> Self {
        match val {
            SerializableAlignSelf::Auto => AlignSelf::Auto,
            SerializableAlignSelf::Start => AlignSelf::Start,
            SerializableAlignSelf::End => AlignSelf::End,
            SerializableAlignSelf::Center => AlignSelf::Center,
            SerializableAlignSelf::Baseline => AlignSelf::Baseline,
            SerializableAlignSelf::Stretch => AlignSelf::Stretch,
        }
    }
}

/// Layout display mode.
///
/// 布局显示模式。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub enum SerializableDisplay {
    /// Participate in flex layout.
    ///
    /// 参与 Flex 布局。
    Flex,
    /// Do not display this node.
    ///
    /// 不显示此节点。
    None,
}

impl From<SerializableDisplay> for BevyDisplay {
    fn from(val: SerializableDisplay) -> Self {
        match val {
            SerializableDisplay::Flex => BevyDisplay::Flex,
            SerializableDisplay::None => BevyDisplay::None,
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
            Value::Static(v) => *v,
            Value::Expr(_) => 0.0,
        },
        match &color.1 {
            Value::Static(v) => *v,
            Value::Expr(_) => 0.0,
        },
        match &color.2 {
            Value::Static(v) => *v,
            Value::Expr(_) => 0.0,
        },
        match &color.3 {
            Value::Static(v) => *v,
            Value::Expr(_) => 0.0,
        },
    )
}

pub fn serializable_vec2_to_vec2(vec: &SerializableVec2) -> Vec2 {
    Vec2::new(
        match &vec.0 {
            Value::Static(v) => *v,
            Value::Expr(_) => 0.0,
        },
        match &vec.1 {
            Value::Static(v) => *v,
            Value::Expr(_) => 0.0,
        },
    )
}

pub fn color_tuple_to_static(color: &ColorTuple) -> (f32, f32, f32, f32) {
    (
        match &color.0 {
            Value::Static(v) => *v,
            Value::Expr(_) => 1.0,
        },
        match &color.1 {
            Value::Static(v) => *v,
            Value::Expr(_) => 1.0,
        },
        match &color.2 {
            Value::Static(v) => *v,
            Value::Expr(_) => 1.0,
        },
        match &color.3 {
            Value::Static(v) => *v,
            Value::Expr(_) => 1.0,
        },
    )
}

pub fn vec2_tuple_to_static(vec: &Vec2Tuple) -> (f32, f32) {
    (
        match &vec.0 {
            Value::Static(v) => *v,
            Value::Expr(_) => 0.5,
        },
        match &vec.1 {
            Value::Static(v) => *v,
            Value::Expr(_) => 0.5,
        },
    )
}

pub fn dynamic_color_to_static(color: &DynamicColor) -> Color {
    Color::srgba(
        match &color.0 {
            Value::Static(v) => *v,
            Value::Expr(_) => 0.0,
        },
        match &color.1 {
            Value::Static(v) => *v,
            Value::Expr(_) => 0.0,
        },
        match &color.2 {
            Value::Static(v) => *v,
            Value::Expr(_) => 0.0,
        },
        match &color.3 {
            Value::Static(v) => *v,
            Value::Expr(_) => 0.0,
        },
    )
}

pub fn is_dynamic_color(color: &DynamicColor) -> bool {
    color.0.is_expr() || color.1.is_expr() || color.2.is_expr() || color.3.is_expr()
}

pub fn serializable_vec3_to_static(vec: &SerializableVec3) -> Vec3 {
    Vec3::new(
        match &vec.0 {
            Value::Static(v) => *v,
            Value::Expr(_) => 0.0,
        },
        match &vec.1 {
            Value::Static(v) => *v,
            Value::Expr(_) => 0.0,
        },
        match &vec.2 {
            Value::Static(v) => *v,
            Value::Expr(_) => 0.0,
        },
    )
}

pub fn is_dynamic_vec3(vec: &SerializableVec3) -> bool {
    vec.0.is_expr() || vec.1.is_expr() || vec.2.is_expr()
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SerializableTransform {
    #[serde(default)]
    pub translation: Option<SerializableVec3>,
    #[serde(default)]
    pub rotation: Option<FloatOrExpr>,
    #[serde(default)]
    pub scale: Option<SerializableVec3>,
}

/// Re-export from schema — font identifier is now a plain String.
pub type ViewFontDef = String;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlignDef {
    #[default]
    Left,
    Center,
    Right,
}

impl From<TextAlignDef> for TextAlign {
    fn from(value: TextAlignDef) -> Self {
        match value {
            TextAlignDef::Left => TextAlign::Left,
            TextAlignDef::Center => TextAlign::Center,
            TextAlignDef::Right => TextAlign::Right,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAnchorDef {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    #[default]
    BottomRight,
}

impl From<TextAnchorDef> for TextAnchor {
    fn from(value: TextAnchorDef) -> Self {
        match value {
            TextAnchorDef::TopLeft => TextAnchor::TOP_LEFT,
            TextAnchorDef::TopCenter => TextAnchor::TOP_CENTER,
            TextAnchorDef::TopRight => TextAnchor::TOP_RIGHT,
            TextAnchorDef::CenterLeft => TextAnchor::CENTER_LEFT,
            TextAnchorDef::Center => TextAnchor::CENTER,
            TextAnchorDef::CenterRight => TextAnchor::CENTER_RIGHT,
            TextAnchorDef::BottomLeft => TextAnchor::BOTTOM_LEFT,
            TextAnchorDef::BottomCenter => TextAnchor::BOTTOM_CENTER,
            TextAnchorDef::BottomRight => TextAnchor::BOTTOM_RIGHT,
        }
    }
}

impl From<ViewFontDef> for crate::core::view::components::ViewFont {
    fn from(val: ViewFontDef) -> Self {
        crate::core::view::components::ViewFont(val)
    }
}
