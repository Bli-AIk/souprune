//! # val.rs
//!
//! Core value types used across all SoupRune schemas.
//! Provides static-or-expression values for data-driven configuration.
//!
//! 所有 SoupRune Schema 通用的核心值类型。
//! 提供静态值或表达式值，用于数据驱动配置。

use serde::{Deserialize, Serialize};

/// Generic value: static or computed from an expression at runtime.
///
/// 泛型值：静态值或运行时从表达式计算。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Val<T> {
    Static(T),
    Expr(String),
}

impl<T> Val<T> {
    pub fn is_expr(&self) -> bool {
        matches!(self, Val::Expr(_))
    }

    pub fn is_dynamic(&self) -> bool {
        self.is_expr()
    }

    pub fn as_static(&self) -> Option<&T> {
        match self {
            Val::Static(v) => Some(v),
            Val::Expr(_) => None,
        }
    }
}

impl<T: Default> Default for Val<T> {
    fn default() -> Self {
        Val::Static(T::default())
    }
}

/// 3D vector: supports both positional `(1.0, 2.0, 3.0)` and named `(x: 1.0, y: 2.0, z: 3.0)`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Vec3Tuple {
    Named {
        x: Val<f32>,
        y: Val<f32>,
        z: Val<f32>,
    },
    Positional(Val<f32>, Val<f32>, Val<f32>),
}

/// 2D vector: supports both positional and named syntax.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Vec2Tuple {
    Named { x: Val<f32>, y: Val<f32> },
    Positional(Val<f32>, Val<f32>),
}

/// RGBA color: supports both positional and named `(r:, g:, b:, a:)` syntax.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ColorTuple {
    Named {
        r: Val<f32>,
        g: Val<f32>,
        b: Val<f32>,
        a: Val<f32>,
    },
    Positional(Val<f32>, Val<f32>, Val<f32>, Val<f32>),
}
