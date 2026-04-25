//! # SDF View Schema
//!
//! SDF structure schema types for `.sdf.ron` files.
//!
//! # SDF View Schema
//!
//! `.sdf.ron` 文件的 SDF 结构 Schema 类型。

use serde::{Deserialize, Serialize};

use super::SerializableColor;

/// SDF structure layout for `.sdf.ron` files.
///
/// `.sdf.ron` 文件的 SDF 结构布局。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SdfStructure {
    pub layer_count: usize,
    pub root: SdfLayerDef,
}

pub type SdfStructureAsset = SdfStructure;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SdfLayerDef {
    pub name: String,
    pub sdf_type: SdfShapeKind,
    #[serde(default)]
    pub color_source: SdfColorSource,
    #[serde(default = "default_z_offset")]
    pub z_offset: f32,
    #[serde(default)]
    pub is_filler: bool,
    #[serde(default)]
    pub children: Vec<SdfLayerDef>,
}

impl Default for SdfLayerDef {
    fn default() -> Self {
        Self {
            name: String::new(),
            sdf_type: SdfShapeKind::Outer,
            color_source: SdfColorSource::default(),
            z_offset: default_z_offset(),
            is_filler: false,
            children: Vec::new(),
        }
    }
}

fn default_z_offset() -> f32 {
    0.1
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum SdfShapeKind {
    Outer,
    Inner,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub enum SdfColorSource {
    #[default]
    FillColor,
    White,
    Custom(SerializableColor),
    /// Toggle between two colors based on a boolean FRE fact.
    ///
    /// 根据布尔 FRE fact 在两种颜色间切换。
    FactToggle {
        key: String,
        on: SerializableColor,
        off: SerializableColor,
    },
}
