//! # bevy_types.rs
//!
//! Bevy type equivalents WITHOUT Bevy dependency.
//! Provides Color, Vec2, Vec3 representations for RON serialization.
//!
//! 无 Bevy 依赖的 Bevy 类型等价物。
//! 提供用于 RON 序列化的 Color、Vec2、Vec3 表示。

use serde::{Deserialize, Serialize};

/// Bevy Color equivalent.
/// In RON files, Color serializes as e.g. `Srgba((red: 1.0, green: 0.0, blue: 0.0, alpha: 1.0))`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BevyColor {
    Srgba(SrgbaColor),
    LinearRgba(SrgbaColor),
}

/// SRGBA color components (0.0–1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrgbaColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

/// Bevy Vec2 equivalent — serializes as `(x, y)` tuple in RON.
pub type BevyVec2 = (f32, f32);

/// Bevy Vec3 equivalent — serializes as `(x, y, z)` tuple in RON.
pub type BevyVec3 = (f32, f32, f32);
