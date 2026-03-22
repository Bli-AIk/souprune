//! Defines shared value wrappers and helper types reused across sequence chapter schema definitions.
//!
//! 定义在各类序列章节 schema 中复用的值包装类型与辅助类型。
//!
//! This file is the low-level value vocabulary of the sequencer schema. It
//! provides the `Value<T>` wrapper for static-versus-expression fields, tuple
//! aliases used by element tweens, and the `LogLevel` type consumed by logging
//! chapters.
//!
//! 这个文件是 sequencer schema 的底层值词汇表。它提供了 `Value<T>`，用于表示
//! 静态值与表达式值的二选一；也提供元素过渡会用到的 tuple 别名，以及日志章节
//! 使用的 `LogLevel` 类型。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub type Vec3Tuple = (Value<f32>, Value<f32>, Value<f32>);
pub type Vec2Tuple = (Value<f32>, Value<f32>);
pub type ColorTuple = (Value<f32>, Value<f32>, Value<f32>, Value<f32>);

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Value<T> {
    Static(T),
    Expr(String),
}

impl<T> Value<T> {
    pub fn is_expr(&self) -> bool {
        matches!(self, Value::Expr(_))
    }

    pub fn is_dynamic(&self) -> bool {
        self.is_expr()
    }

    pub fn as_static(&self) -> Option<&T> {
        match self {
            Value::Static(v) => Some(v),
            Value::Expr(_) => None,
        }
    }

    pub fn as_expr(&self) -> Option<&str> {
        match self {
            Value::Expr(s) => Some(s),
            Value::Static(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub enum LogLevel {
    #[default]
    Info,
    Debug,
    Warn,
    Error,
}

impl LogLevel {
    pub fn action(&self, text: &str) {
        match self {
            LogLevel::Info => info!("[Chapter] {}", text),
            LogLevel::Debug => debug!("[Chapter] {}", text),
            LogLevel::Warn => warn!("[Chapter] {}", text),
            LogLevel::Error => error!("[Chapter] {}", text),
        }
    }
}
