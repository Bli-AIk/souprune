//! # Sequence Supporting Types
//!
//! Shared value, selector, tween, and action schema types used by sequence chapters.
//!
//! # 序列辅助类型
//!
//! 序列章节使用的共享值、选择器、补间与动作 Schema 类型。

use crate::val::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FactCondition {
    Equals { key: String, value: FactValueMatch },
    GreaterThan { key: String, value: i64 },
    LessThan { key: String, value: i64 },
    GreaterOrEqual { key: String, value: i64 },
    LessOrEqual { key: String, value: i64 },
    Exists(String),
    NotExists(String),
    IsTrue(String),
    IsFalse(String),
    And(Vec<FactCondition>),
    Or(Vec<FactCondition>),
    Not(Box<FactCondition>),
    Always,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FactValueMatch {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Expr(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FactModificationDef {
    Set { key: String, value: FactValueMatch },
    Increment { key: String, amount: i64 },
    Remove(String),
    Toggle(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AggregateRule {
    Collect(String),
    CollectKeys(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DataBinding {
    File(String),
    Files(Vec<String>),
    LocalLayer,
    Expr(String),
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum LogLevel {
    #[default]
    Info,
    Debug,
    Warn,
    Error,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum CameraAction {
    SetPosition((f32, f32)),
    SetZoom(f32),
    Shake { duration: f32, intensity: f32 },
    FollowPlayer(bool),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum UIAction {
    LoadLayout(String),
    Show(String),
    Hide(String),
    SetText { id: String, content: String },
    SetVariable { name: String, value: String },
    PlayAnimation { id: String, clip: String },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PlayerAction {
    SetMode(Vec<String>),
    Spawn {
        config_path: String,
        #[serde(default)]
        position: Option<(f32, f32)>,
    },
    Teleport((f32, f32)),
    SetActive(bool),
    Despawn,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ElementSelector {
    FullName(String),
    LocalName(String),
    Tag(String),
}

impl ElementSelector {
    /// Select an element by its full hierarchical name.
    ///
    /// 按完整层级名称选择元素。
    pub fn full(name: impl Into<String>) -> Self {
        Self::FullName(name.into())
    }

    /// Select an element by its local name.
    ///
    /// 按本地名称选择元素。
    pub fn local(name: impl Into<String>) -> Self {
        Self::LocalName(name.into())
    }

    /// Select elements by tag.
    ///
    /// 按标签选择元素。
    pub fn tag(tag: impl Into<String>) -> Self {
        Self::Tag(tag.into())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ElementModification {
    SetTexture(String),
    SetPosition(Val<f32>, Val<f32>, Val<f32>),
    SetScale(Val<f32>, Val<f32>, Val<f32>),
    SetColor(Val<f32>, Val<f32>, Val<f32>, Val<f32>),
    SetVisibility(Val<bool>),
    SetBoxSize(Val<f32>, Val<f32>),
    Undo,
    Redo,
    Reset,
}

impl ElementModification {
    /// Set element position.
    ///
    /// 设置元素位置。
    pub fn set_position(
        x: impl Into<Val<f32>>,
        y: impl Into<Val<f32>>,
        z: impl Into<Val<f32>>,
    ) -> Self {
        Self::SetPosition(x.into(), y.into(), z.into())
    }

    /// Set element scale.
    ///
    /// 设置元素缩放。
    pub fn set_scale(
        x: impl Into<Val<f32>>,
        y: impl Into<Val<f32>>,
        z: impl Into<Val<f32>>,
    ) -> Self {
        Self::SetScale(x.into(), y.into(), z.into())
    }

    /// Set element color.
    ///
    /// 设置元素颜色。
    pub fn set_color(
        red: impl Into<Val<f32>>,
        green: impl Into<Val<f32>>,
        blue: impl Into<Val<f32>>,
        alpha: impl Into<Val<f32>>,
    ) -> Self {
        Self::SetColor(red.into(), green.into(), blue.into(), alpha.into())
    }

    /// Set element visibility.
    ///
    /// 设置元素可见性。
    pub fn set_visibility(value: impl Into<Val<bool>>) -> Self {
        Self::SetVisibility(value.into())
    }

    /// Set element ViewBox size.
    ///
    /// 设置元素 ViewBox 尺寸。
    pub fn set_box_size(width: impl Into<Val<f32>>, height: impl Into<Val<f32>>) -> Self {
        Self::SetBoxSize(width.into(), height.into())
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum EaseKindRepr {
    #[default]
    Linear,
    QuadIn,
    QuadOut,
    InOutQuad,
    CubicIn,
    CubicOut,
    InOutCubic,
    SineIn,
    SineOut,
    InOutSine,
    CircularIn,
    CircularOut,
    InOutCircular,
    ExpoIn,
    ExpoOut,
    InOutExpo,
    ElasticIn,
    ElasticOut,
    InOutElastic,
    BounceIn,
    BounceOut,
    InOutBounce,
    BackIn,
    BackOut,
    InOutBack,
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
        from: Option<Val<f32>>,
        to: Val<f32>,
    },
    Alpha {
        #[serde(default)]
        from: Option<Val<f32>>,
        to: Val<f32>,
    },
    Anchor(f32, f32),
}

impl TweenTarget {
    /// Create a position tween target without an explicit source value.
    ///
    /// 创建不带显式起始值的位置补间目标。
    pub fn position(to: impl Into<Vec3Tuple>) -> Self {
        Self::Position {
            from: None,
            to: to.into(),
        }
    }

    /// Create a position tween target with an explicit source value.
    ///
    /// 创建带显式起始值的位置补间目标。
    pub fn position_from(from: impl Into<Vec3Tuple>, to: impl Into<Vec3Tuple>) -> Self {
        Self::Position {
            from: Some(from.into()),
            to: to.into(),
        }
    }

    /// Create a scale tween target without an explicit source value.
    ///
    /// 创建不带显式起始值的缩放补间目标。
    pub fn scale(to: impl Into<Vec3Tuple>) -> Self {
        Self::Scale {
            from: None,
            to: to.into(),
        }
    }

    /// Create a scale tween target with an explicit source value.
    ///
    /// 创建带显式起始值的缩放补间目标。
    pub fn scale_from(from: impl Into<Vec3Tuple>, to: impl Into<Vec3Tuple>) -> Self {
        Self::Scale {
            from: Some(from.into()),
            to: to.into(),
        }
    }

    /// Create a color tween target without an explicit source value.
    ///
    /// 创建不带显式起始值的颜色补间目标。
    pub fn color(to: impl Into<ColorTuple>) -> Self {
        Self::Color {
            from: None,
            to: to.into(),
        }
    }

    /// Create a color tween target with an explicit source value.
    ///
    /// 创建带显式起始值的颜色补间目标。
    pub fn color_from(from: impl Into<ColorTuple>, to: impl Into<ColorTuple>) -> Self {
        Self::Color {
            from: Some(from.into()),
            to: to.into(),
        }
    }

    /// Create a ViewBox size tween target without an explicit source value.
    ///
    /// 创建不带显式起始值的 ViewBox 尺寸补间目标。
    pub fn box_size(to: impl Into<Vec2Tuple>) -> Self {
        Self::BoxSize {
            from: None,
            to: to.into(),
        }
    }

    /// Create a ViewBox size tween target with an explicit source value.
    ///
    /// 创建带显式起始值的 ViewBox 尺寸补间目标。
    pub fn box_size_from(from: impl Into<Vec2Tuple>, to: impl Into<Vec2Tuple>) -> Self {
        Self::BoxSize {
            from: Some(from.into()),
            to: to.into(),
        }
    }

    /// Create a rotation tween target without an explicit source value.
    ///
    /// 创建不带显式起始值的旋转补间目标。
    pub fn rotation(to: impl Into<Val<f32>>) -> Self {
        Self::Rotation {
            from: None,
            to: to.into(),
        }
    }

    /// Create a rotation tween target with an explicit source value.
    ///
    /// 创建带显式起始值的旋转补间目标。
    pub fn rotation_from(from: impl Into<Val<f32>>, to: impl Into<Val<f32>>) -> Self {
        Self::Rotation {
            from: Some(from.into()),
            to: to.into(),
        }
    }

    /// Create an alpha tween target without an explicit source value.
    ///
    /// 创建不带显式起始值的透明度补间目标。
    pub fn alpha(to: impl Into<Val<f32>>) -> Self {
        Self::Alpha {
            from: None,
            to: to.into(),
        }
    }

    /// Create an alpha tween target with an explicit source value.
    ///
    /// 创建带显式起始值的透明度补间目标。
    pub fn alpha_from(from: impl Into<Val<f32>>, to: impl Into<Val<f32>>) -> Self {
        Self::Alpha {
            from: Some(from.into()),
            to: to.into(),
        }
    }
}
