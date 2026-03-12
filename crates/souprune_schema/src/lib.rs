//! # souprune_schema
//!
//! Pure data schema types for SoupRune RON files.
//! No Bevy dependency — designed for use by both the main game engine
//! and standalone tools (linters, editors, CI validators).
//!
//! SoupRune RON 文件的纯数据 Schema 类型。
//! 无 Bevy 依赖——同时供主游戏引擎和独立工具使用。

pub mod val;
pub mod view;

pub use val::{ColorTuple, Val, Vec2Tuple, Vec3Tuple};

/// Parse a RON string into a typed schema.
///
/// 将 RON 字符串解析为类型化的 Schema。
pub fn from_ron_str<T: serde::de::DeserializeOwned>(
    s: &str,
) -> Result<T, ron::error::SpannedError> {
    ron::from_str(s)
}

/// All supported RON file types with their schema targets.
///
/// 所有支持的 RON 文件类型及其 Schema 目标。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RonFileKind {
    View,
    SdfStructure,
}

impl RonFileKind {
    /// Detect file kind from file extension.
    ///
    /// 从文件扩展名检测文件类型。
    pub fn from_path(path: &str) -> Option<Self> {
        if path.ends_with(".view.ron") {
            Some(Self::View)
        } else if path.ends_with(".sdf.ron") {
            Some(Self::SdfStructure)
        } else {
            None
        }
    }

    /// All recognized extensions.
    pub fn all_extensions() -> &'static [&'static str] {
        &[".view.ron", ".sdf.ron"]
    }
}
