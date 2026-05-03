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

/// Rectangle definition for ghost text spawn area.
///
/// 幽灵文本生成区域的矩形定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RectDef {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// How text characters appear on screen during typewriter playback.
///
/// 打字机播放期间文本字符在屏幕上的显示方式。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextDisplayDef {
    /// Standard left-to-right layout within a text block.
    /// 标准从左到右排列。
    Normal,
    /// Ghost text — characters appear at random positions and fade out.
    /// 幽灵文本 — 字符出现在随机位置并渐隐。
    Ghost {
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
    /// When set, glyphs orbit their base position with `char_index * angle` phase offset,
    /// matching Undertale's `shake > 38` / `shake == 43` orbiting draw effect.
    ///
    /// 设置后，字形围绕基位置以 `char_index * angle` 相位偏移进行轨道运动，
    /// 匹配 Undertale 的 `shake > 38` / `shake == 43` 轨道绘制效果。
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
