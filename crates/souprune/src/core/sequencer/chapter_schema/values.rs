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
