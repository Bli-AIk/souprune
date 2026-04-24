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
    fn rejects_split_battle_box_with_legacy_easing_alias() {
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

        let error = ron::from_str::<Chapter>(ron).expect_err("legacy easing alias should fail");
        assert!(
            error.to_string().contains("OutCubic"),
            "error should mention rejected legacy alias: {error}",
        );
    }

    #[test]
    fn parses_split_battle_box_with_canonical_easing_name() {
        let ron = r#"SplitBattleBox(
            source: "main",
            result: ("left", "right"),
            axis: Vertical,
            position: 0.0,
            gap: 20.0,
            gap_policy: Expands,
            duration: 0.3,
            easing: CubicOut,
        )"#;

        let chapter: Chapter = ron::from_str(ron).expect("SplitBattleBox should parse");
        match chapter {
            Chapter::SplitBattleBox {
                source,
                result,
                axis,
                gap_policy,
                easing,
                ..
            } => {
                assert_eq!(source, "main");
                assert_eq!(result, ("left".to_string(), "right".to_string()));
                assert_eq!(axis, SplitAxis::Vertical);
                assert_eq!(gap_policy, GapPolicy::Expands);
                assert_eq!(easing, EaseKindRepr::CubicOut);
            }
            other => panic!("unexpected chapter parsed: {other:?}"),
        }
    }

    #[test]
    fn parses_merge_battle_boxes_and_log_chapters() {
        let merge = r#"MergeBattleBoxes(
            sources: ("left", "right"),
            result: "main",
            gap_policy: Includes,
            duration: 0.5,
            easing: CubicOut,
        )"#;
        let log = r#"Log(
            text: "hello",
            level: Warn,
        )"#;

        let merge_chapter: Chapter = ron::from_str(merge).expect("MergeBattleBoxes should parse");
        let log_chapter: Chapter = ron::from_str(log).expect("Log should parse");

        match merge_chapter {
            Chapter::MergeBattleBoxes {
                sources,
                result,
                gap_policy,
                easing,
                ..
            } => {
                assert_eq!(sources, ("left".to_string(), "right".to_string()));
                assert_eq!(result, "main");
                assert_eq!(gap_policy, GapPolicy::Includes);
                assert_eq!(easing, EaseKindRepr::CubicOut);
            }
            other => panic!("unexpected chapter parsed: {other:?}"),
        }

        match log_chapter {
            Chapter::Log { text, level } => {
                assert_eq!(text, "hello");
                assert_eq!(level, LogLevel::Warn);
            }
            other => panic!("unexpected chapter parsed: {other:?}"),
        }
    }
}
