//! # Sequence Schema
//!
//! SequenceAsset schema types for `.sequence.ron` files.
//! Mirrors `souprune::core::sequencer::chapter_schema` without Bevy dependency.
//!
//! # 序列 Schema
//!
//! `.sequence.ron` 文件的序列资源 Schema 类型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod chapter;
mod types;

pub use chapter::*;
pub use types::*;

/// Sequence configuration asset — top-level `.sequence.ron` schema.
///
/// 序列配置资源 — `.sequence.ron` 的顶层 Schema。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SequenceAsset {
    /// Identifier for the execution mode.
    ///
    /// 执行模式的标识符。
    #[serde(default)]
    pub mode: Option<String>,
    /// Path to a `.fre.ron` file containing local rules for this sequence.
    ///
    /// 包含此序列本地规则的 `.fre.ron` 文件路径。
    #[serde(default)]
    pub rules_file: Option<String>,
    /// Exit mappings (e.g., `"success": "next_level"`).
    ///
    /// 退出映射（如 `"success": "next_level"）。
    #[serde(default, serialize_with = "crate::ordered_map::serialize_ordered_map")]
    pub exits: HashMap<String, String>,
    /// List of chapters making up the sequence.
    ///
    /// 构成序列的章节列表。
    pub chapters: Vec<Chapter>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_battle_box_chapters_in_generic_sequence_schema() {
        let ron = r#"SplitBattleBox(
            source: "main",
            result: ("left", "right"),
            axis: Vertical,
            position: 0.0,
            gap: 20.0,
            gap_policy: Expands,
            duration: 0.3,
            easing: OutCubic,
        )"#;

        let error = ron::from_str::<Chapter>(ron).expect_err("battle-specific chapter should fail");
        assert!(
            error.to_string().contains("SplitBattleBox"),
            "error should mention rejected chapter name: {error}",
        );
    }

    #[test]
    fn parses_log_chapter() {
        let log = r#"Log(
            text: "hello",
            level: Warn,
        )"#;

        let log_chapter: Chapter = ron::from_str(log).expect("Log should parse");

        match log_chapter {
            Chapter::Log { text, level } => {
                assert_eq!(text, "hello");
                assert_eq!(level, LogLevel::Warn);
            }
            other => panic!("unexpected chapter parsed: {other:?}"),
        }
    }

    #[test]
    fn parses_battle_speech_bubble_chapter() {
        let ron = r#"BattleSpeechBubble((
            mortar_path: "battle/enemies/mad_dummy.mortar",
            mortar_node: "enemy_speech_manual_intro",
            frame: MadDummyWide,
            advance: Manual,
        ))"#;

        let chapter: Chapter = ron::from_str(ron).expect("BattleSpeechBubble should parse");

        match chapter {
            Chapter::BattleSpeechBubble(request) => {
                assert_eq!(request.channel, "battle_enemy_speech");
                assert_eq!(request.mortar_node, "enemy_speech_manual_intro");
            }
            other => panic!("unexpected chapter parsed: {other:?}"),
        }
    }
}
