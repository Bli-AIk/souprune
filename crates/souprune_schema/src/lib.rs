//! # souprune_schema
//!
//! Pure data schema types for SoupRune RON files.
//! No Bevy dependency — designed for use by both the main framework
//! and standalone tools (linters, editors, CI validators).
//!
//! SoupRune RON 文件的纯数据 Schema 类型。
//! 无 Bevy 依赖——同时供主框架和独立工具使用。

pub mod battle;
pub mod bevy_types;
pub mod character;
pub mod config;
pub mod danmaku;
pub mod dialogue;
pub mod enemy;
pub mod fre;
pub mod item;
pub mod ordered_map;
pub mod overworld;
pub mod sequence;
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
    /// `.view.ron`
    View,
    /// `.sdf.ron`
    SdfStructure,
    /// `.performance.ron`
    Performance,
    /// `.sequence.ron`
    Sequence,
    /// `.enemy.ron`
    Enemy,
    /// `.items.ron`
    Items,
    /// `.fre.ron`
    Fre,
    /// `dialogue.ron` (exact filename)
    Dialogue,
    /// `input.ron` (exact filename)
    Input,
    /// `flow.ron` (exact filename)
    Flow,
    /// `touch_layout.ron` (exact filename)
    TouchLayout,
    /// `alight_motion_config.ron` (exact filename)
    AlightMotionConfig,
    /// `.character.ron`
    Character,
    /// `.animation_config.ron`
    AnimationConfig,
    /// `player_behavior.ron` (exact filename)
    PlayerBehavior,
    /// `chase_config.ron` (exact filename)
    ChaseConfig,
}

impl RonFileKind {
    /// Detect file kind from path (extension or exact filename).
    ///
    /// 从路径检测文件类型（扩展名或精确文件名）。
    pub fn from_path(path: &str) -> Option<Self> {
        // Extension-based matching (order: longer suffixes first)
        if path.ends_with(".view.ron") {
            return Some(Self::View);
        }
        if path.ends_with(".sdf.ron") {
            return Some(Self::SdfStructure);
        }
        if path.ends_with(".performance.ron") {
            return Some(Self::Performance);
        }
        if path.ends_with(".sequence.ron") {
            return Some(Self::Sequence);
        }
        if path.ends_with(".enemy.ron") {
            return Some(Self::Enemy);
        }
        if path.ends_with(".items.ron") {
            return Some(Self::Items);
        }
        if path.ends_with(".fre.ron") {
            return Some(Self::Fre);
        }
        if path.ends_with(".animation_config.ron") {
            return Some(Self::AnimationConfig);
        }
        if path.ends_with(".character.ron") {
            return Some(Self::Character);
        }

        // Exact filename matching
        let filename = path.rsplit(['/', '\\']).next().unwrap_or(path);
        match filename {
            "dialogue.ron" => Some(Self::Dialogue),
            "input.ron" => Some(Self::Input),
            "flow.ron" => Some(Self::Flow),
            "touch_layout.ron" => Some(Self::TouchLayout),
            "alight_motion_config.ron" => Some(Self::AlightMotionConfig),
            "player_behavior.ron" => Some(Self::PlayerBehavior),
            "chase_config.ron" => Some(Self::ChaseConfig),
            _ => None,
        }
    }

    /// All recognized extensions and filenames.
    pub fn all_extensions() -> &'static [&'static str] {
        &[
            ".view.ron",
            ".sdf.ron",
            ".performance.ron",
            ".sequence.ron",
            ".enemy.ron",
            ".items.ron",
            ".fre.ron",
            ".animation_config.ron",
            ".character.ron",
            "dialogue.ron",
            "input.ron",
            "flow.ron",
            "touch_layout.ron",
            "alight_motion_config.ron",
            "player_behavior.ron",
            "chase_config.ron",
        ]
    }

    /// Returns true if the path ends with `.ron`.
    pub fn is_ron_file(path: &str) -> bool {
        path.ends_with(".ron")
    }
}
