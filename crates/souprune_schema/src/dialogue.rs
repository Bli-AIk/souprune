//! Dialogue configuration schema types for `dialogue.ron`.
//!
//! `dialogue.ron` 文件的对话配置 Schema 类型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Automatic punctuation pause rules.
///
/// 自动标点停顿规则。
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AutoPauseConfig {
    pub default_preset: String,
    #[serde(serialize_with = "crate::ordered_map::serialize_ordered_nested_map")]
    pub presets: HashMap<String, HashMap<String, f64>>,
}

/// Voice playback rules.
///
/// 语音播放规则。
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct VoiceConfig {
    pub default_preset: String,
    #[serde(serialize_with = "crate::ordered_map::serialize_ordered_nested_map")]
    pub presets: HashMap<String, HashMap<String, bool>>,
}

/// Rectangle definition for text spawn areas.
///
/// 文本生成区域的矩形定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RectDef {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// How text characters appear on screen during typewriter playback.
///
/// 打字机播放期间文本字符在屏幕上的显示方式。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextDisplayDef {
    /// Standard left-to-right layout within a text block.
    /// 标准从左到右排列。
    Normal,
    /// Floating text — characters appear at configured positions and fade out.
    /// 浮动文本 — 字符出现在配置的位置并渐隐。
    Floating {
        spawn_area: RectDef,
        linger_seconds: f32,
    },
}

/// Per-character shake configuration.
///
/// 逐字符抖动配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextShakeDef {
    pub intensity: f32,
    #[serde(default)]
    pub mode: TextShakeModeDef,
}

/// Per-character shake selection mode.
///
/// 逐字符抖动选择模式。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum TextShakeModeDef {
    /// Every visible glyph shakes continuously.
    ///
    /// 所有可见字形持续抖动。
    #[default]
    Continuous,
    /// At interval boundaries, one visible glyph may shake briefly.
    ///
    /// 每个间隔边界处，可能让一个可见字形短暂抖动。
    RandomSingle {
        /// Seconds between shake attempts.
        ///
        /// 抖动尝试之间的秒数。
        interval_seconds: f32,
        /// Probability that an interval produces a shake.
        ///
        /// 每个间隔实际产生抖动的概率。
        chance: f32,
        /// Seconds that the selected glyph keeps the shake effect.
        ///
        /// 被选中字形保留抖动效果的秒数。
        duration_seconds: f32,
    },
    /// Occasionally offsets one visible glyph for one frame.
    ///
    /// 偶发地将一个可见字形偏移一帧。
    Twitch {
        /// Average frames between twitch attempts.
        ///
        /// 抖动尝试之间的平均帧数。
        average_frames: u32,
        /// Random frame variation around the average.
        ///
        /// 平均值附近的随机帧变化量。
        frame_variation: u32,
    },
}

/// Per-character wave or orbiting distortion.
///
/// 逐字符波浪或轨道扭曲。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextWaveDef {
    pub amplitude: f32,
    pub frequency: f32,
    /// Phase angle offset per character, in degrees.
    ///
    /// 每字符相位角偏移，单位为度。
    ///
    /// When set, glyphs orbit their base position with `char_index * angle` phase offset.
    ///
    /// 设置后，字形围绕基位置以 `char_index * angle` 相位偏移进行轨道运动。
    ///
    /// When `None` (default), a traditional spatial Y-based sine wave is applied.
    ///
    /// `None`（默认）时，应用传统基于 Y 坐标的正弦波浪。
    #[serde(default)]
    pub orbit_angle_per_char_deg: Option<f32>,
}

impl Default for TextWaveDef {
    fn default() -> Self {
        Self {
            amplitude: 0.0,
            frequency: 0.0,
            orbit_angle_per_char_deg: None,
        }
    }
}

/// A complete character text style preset.
///
/// 一个完整的角色文本风格预设。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextAnimationPresetDef {
    pub display: TextDisplayDef,
    pub shake: Option<TextShakeDef>,
    pub wave: Option<TextWaveDef>,
}

/// Top-level text animation config.
///
/// 顶层文本动画配置。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextAnimationConfigDef {
    pub default_preset: String,
    #[serde(serialize_with = "crate::ordered_map::serialize_ordered_map")]
    pub presets: HashMap<String, TextAnimationPresetDef>,
}

/// Top-level dialogue configuration.
///
/// 顶层对话配置。
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DialogueConfig {
    #[serde(default)]
    pub auto_pause: AutoPauseConfig,
    #[serde(default)]
    pub voice: VoiceConfig,
    #[serde(default)]
    pub text_animation: TextAnimationConfigDef,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floating_display_serializes_full_area_field_names() {
        let display = TextDisplayDef::Floating {
            spawn_area: RectDef {
                x: 1.0,
                y: 2.0,
                width: 3.0,
                height: 4.0,
            },
            linger_seconds: 0.5,
        };

        let ron = ron::to_string(&display).unwrap();

        assert!(ron.contains("Floating"));
        assert!(ron.contains("width"));
        assert!(ron.contains("height"));
        assert!(!ron.contains("(w:"));
        assert!(!ron.contains(",w:"));
        assert!(!ron.contains("(h:"));
        assert!(!ron.contains(",h:"));
    }

    #[test]
    fn random_single_shake_mode_serializes_timing_fields() {
        let mode = TextShakeModeDef::RandomSingle {
            interval_seconds: 0.08,
            chance: 0.4,
            duration_seconds: 0.03,
        };

        let ron = ron::to_string(&mode).unwrap();

        assert!(ron.contains("RandomSingle"));
        assert!(ron.contains("interval_seconds"));
        assert!(ron.contains("chance"));
        assert!(ron.contains("duration_seconds"));
    }

    #[test]
    fn twitch_shake_mode_deserializes_frame_window() {
        let mode: TextShakeModeDef =
            ron::from_str("Twitch(average_frames: 48, frame_variation: 16)").unwrap();

        match mode {
            TextShakeModeDef::Twitch {
                average_frames,
                frame_variation,
            } => {
                assert_eq!(average_frames, 48);
                assert_eq!(frame_variation, 16);
            }
            other => panic!("expected Twitch mode, got {other:?}"),
        }
    }
}
