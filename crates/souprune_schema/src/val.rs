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

/// 3D vector tuple: (x, y, z) — each component can be static or expression.
pub type Vec3Tuple = (Val<f32>, Val<f32>, Val<f32>);

/// 2D vector tuple: (x, y).
pub type Vec2Tuple = (Val<f32>, Val<f32>);

/// RGBA color tuple: (r, g, b, a).
pub type ColorTuple = (Val<f32>, Val<f32>, Val<f32>, Val<f32>);
