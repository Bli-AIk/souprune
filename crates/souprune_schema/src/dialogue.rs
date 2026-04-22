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
    pub presets: HashMap<String, HashMap<String, f64>>,
}

/// Voice playback rules.
///
/// 语音播放规则。
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct VoiceConfig {
    pub default_preset: String,
    pub presets: HashMap<String, HashMap<String, bool>>,
}

/// Top-level dialogue configuration.
///
/// 顶层对话配置。
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DialogueConfig {
    pub auto_pause: AutoPauseConfig,
    pub voice: VoiceConfig,
}
