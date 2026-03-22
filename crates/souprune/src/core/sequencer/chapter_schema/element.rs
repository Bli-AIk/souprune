//! Defines view-element selectors, modifications, tween targets, and easing serialization for sequence chapters.
//!
//! 定义序列章节里用于操作 View 元素的选择器、修改项、过渡目标以及 easing 序列化规则。
//!
//! This file is the part of the sequence schema that talks about individual UI
//! elements. It describes how a chapter points at an element, which properties
//! may be changed or tweened, and how easing names from asset files map onto the
//! runtime tween implementation.
//!
//! 这个文件是序列 schema 里专门描述单个 UI 元素操作的一部分。它定义章节如何
//! 选中元素、可以修改或补间哪些属性，以及资产文件里的 easing 名称如何映射到
//! 运行时实际使用的 tween 实现。

use super::values::{ColorTuple, Value, Vec2Tuple, Vec3Tuple};
use bevy_tween::interpolation::EaseKind;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ElementSelector {
    FullName(String),
    LocalName(String),
    Tag(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ElementModification {
    SetTexture(String),
    SetPosition(Value<f32>, Value<f32>, Value<f32>),
    SetScale(Value<f32>, Value<f32>, Value<f32>),
    SetColor(Value<f32>, Value<f32>, Value<f32>, Value<f32>),
    SetVisibility(Value<bool>),
    SetBoxSize(Value<f32>, Value<f32>),
    Undo,
    Redo,
    Reset,
}

pub(crate) fn default_easing() -> EaseKind {
    EaseKind::Linear
}

pub(crate) mod ease_kind_serde {
    use super::*;

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    enum EaseKindRepr {
        Linear,
        #[serde(alias = "InQuad")]
        QuadIn,
        #[serde(alias = "OutQuad")]
        QuadOut,
        #[serde(alias = "InOutQuad")]
        QuadInOut,
        #[serde(alias = "InCubic")]
        CubicIn,
        #[serde(alias = "OutCubic")]
        CubicOut,
        #[serde(alias = "InOutCubic")]
        CubicInOut,
        #[serde(alias = "InSine")]
        SineIn,
        #[serde(alias = "OutSine")]
        SineOut,
        #[serde(alias = "InOutSine")]
        SineInOut,
        #[serde(alias = "InCirc")]
        CircularIn,
        #[serde(alias = "OutCirc")]
        CircularOut,
        #[serde(alias = "InOutCirc")]
        CircularInOut,
        ExpoIn,
        ExpoOut,
        ExpoInOut,
        ElasticIn,
        ElasticOut,
        ElasticInOut,
        BounceIn,
        BounceOut,
        BounceInOut,
        BackIn,
        BackOut,
        BackInOut,
    }

    impl From<EaseKindRepr> for EaseKind {
        fn from(repr: EaseKindRepr) -> Self {
            match repr {
                EaseKindRepr::Linear => EaseKind::Linear,
                EaseKindRepr::QuadIn => EaseKind::QuadraticIn,
                EaseKindRepr::QuadOut => EaseKind::QuadraticOut,
                EaseKindRepr::QuadInOut => EaseKind::QuadraticInOut,
                EaseKindRepr::CubicIn => EaseKind::CubicIn,
                EaseKindRepr::CubicOut => EaseKind::CubicOut,
                EaseKindRepr::CubicInOut => EaseKind::CubicInOut,
                EaseKindRepr::SineIn => EaseKind::SineIn,
                EaseKindRepr::SineOut => EaseKind::SineOut,
                EaseKindRepr::SineInOut => EaseKind::SineInOut,
                EaseKindRepr::CircularIn => EaseKind::CircularIn,
                EaseKindRepr::CircularOut => EaseKind::CircularOut,
                EaseKindRepr::CircularInOut => EaseKind::CircularInOut,
                EaseKindRepr::ExpoIn => EaseKind::ExponentialIn,
                EaseKindRepr::ExpoOut => EaseKind::ExponentialOut,
                EaseKindRepr::ExpoInOut => EaseKind::ExponentialInOut,
                EaseKindRepr::ElasticIn => EaseKind::ElasticIn,
                EaseKindRepr::ElasticOut => EaseKind::ElasticOut,
                EaseKindRepr::ElasticInOut => EaseKind::ElasticInOut,
                EaseKindRepr::BounceIn => EaseKind::BounceIn,
                EaseKindRepr::BounceOut => EaseKind::BounceOut,
                EaseKindRepr::BounceInOut => EaseKind::BounceInOut,
                EaseKindRepr::BackIn => EaseKind::BackIn,
                EaseKindRepr::BackOut => EaseKind::BackOut,
                EaseKindRepr::BackInOut => EaseKind::BackInOut,
            }
        }
    }

    impl From<EaseKind> for EaseKindRepr {
        fn from(kind: EaseKind) -> Self {
            match kind {
                EaseKind::Linear => EaseKindRepr::Linear,
                EaseKind::QuadraticIn => EaseKindRepr::QuadIn,
                EaseKind::QuadraticOut => EaseKindRepr::QuadOut,
                EaseKind::QuadraticInOut => EaseKindRepr::QuadInOut,
                EaseKind::CubicIn => EaseKindRepr::CubicIn,
                EaseKind::CubicOut => EaseKindRepr::CubicOut,
                EaseKind::CubicInOut => EaseKindRepr::CubicInOut,
                EaseKind::SineIn => EaseKindRepr::SineIn,
                EaseKind::SineOut => EaseKindRepr::SineOut,
                EaseKind::SineInOut => EaseKindRepr::SineInOut,
                EaseKind::CircularIn => EaseKindRepr::CircularIn,
                EaseKind::CircularOut => EaseKindRepr::CircularOut,
                EaseKind::CircularInOut => EaseKindRepr::CircularInOut,
                EaseKind::ExponentialIn => EaseKindRepr::ExpoIn,
                EaseKind::ExponentialOut => EaseKindRepr::ExpoOut,
                EaseKind::ExponentialInOut => EaseKindRepr::ExpoInOut,
                EaseKind::ElasticIn => EaseKindRepr::ElasticIn,
                EaseKind::ElasticOut => EaseKindRepr::ElasticOut,
                EaseKind::ElasticInOut => EaseKindRepr::ElasticInOut,
                EaseKind::BounceIn => EaseKindRepr::BounceIn,
                EaseKind::BounceOut => EaseKindRepr::BounceOut,
                EaseKind::BounceInOut => EaseKindRepr::BounceInOut,
                EaseKind::BackIn => EaseKindRepr::BackIn,
                EaseKind::BackOut => EaseKindRepr::BackOut,
                EaseKind::BackInOut => EaseKindRepr::BackInOut,
                _ => EaseKindRepr::Linear,
            }
        }
    }

    pub fn serialize<S>(kind: &EaseKind, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let repr: EaseKindRepr = (*kind).into();
        repr.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<EaseKind, D::Error>
    where
        D: Deserializer<'de>,
    {
        let repr = EaseKindRepr::deserialize(deserializer)?;
        Ok(repr.into())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TweenTarget {
    Position {
        #[serde(default)]
        from: Option<Vec3Tuple>,
        to: Vec3Tuple,
    },
    Scale {
        #[serde(default)]
        from: Option<Vec3Tuple>,
        to: Vec3Tuple,
    },
    Color {
        #[serde(default)]
        from: Option<ColorTuple>,
        to: ColorTuple,
    },
    BoxSize {
        #[serde(default)]
        from: Option<Vec2Tuple>,
        to: Vec2Tuple,
    },
    Rotation {
        #[serde(default)]
        from: Option<Value<f32>>,
        to: Value<f32>,
    },
    Alpha {
        #[serde(default)]
        from: Option<Value<f32>>,
        to: Value<f32>,
    },
}
