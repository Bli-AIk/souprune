//! # View Material Schema
//!
//! Material-related schema types for view and SDF assets.
//!
//! # View 材质 Schema
//!
//! View 与 SDF 资源的材质相关 Schema 类型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MaterialDef {
    pub shader: String,
    #[serde(default, serialize_with = "crate::ordered_map::serialize_ordered_map")]
    pub params: HashMap<String, MaterialParamValue>,
    #[serde(default)]
    pub animations: Option<MaterialAnimationsDef>,
    #[serde(default)]
    pub texture: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum MaterialParamValue {
    Static(f32),
    Expr(String),
}

impl Default for MaterialParamValue {
    fn default() -> Self {
        Self::Static(0.0)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MaterialAnimationsDef {
    #[serde(default)]
    pub lag: Option<LagAnimationDef>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LagAnimationDef {
    pub source: String,
    pub target: String,
    #[serde(default = "default_lag_delay")]
    pub delay: f32,
    #[serde(default = "default_lag_duration")]
    pub duration: f32,
    #[serde(default)]
    pub easing: EasingDef,
}

impl Default for LagAnimationDef {
    fn default() -> Self {
        Self {
            source: String::new(),
            target: String::new(),
            delay: default_lag_delay(),
            duration: default_lag_duration(),
            easing: EasingDef::default(),
        }
    }
}

fn default_lag_delay() -> f32 {
    0.2
}

fn default_lag_duration() -> f32 {
    0.4
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub enum EasingDef {
    #[default]
    Linear,
    InQuad,
    OutQuad,
    InOutQuad,
    InCubic,
    OutCubic,
    InOutCubic,
    InCirc,
    OutCirc,
    InOutCirc,
}
