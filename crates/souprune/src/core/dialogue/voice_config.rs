//! Configurable voice playback rules for typewriter text.
//!
//! 打字机文本的可配置语音播放规则。
//!
//! Controls which characters trigger or suppress voice sound effects.
//! Loaded from the `voice` section of `narrative/dialogue.ron`.
//! Named presets allow per-character or per-scene overrides.
//!
//! 控制哪些字符触发或抑制语音音效。
//! 从 `narrative/dialogue.ron` 的 `voice` 段加载。
//! 命名预设允许按角色或按场景覆盖。

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use souprune_schema::dialogue::VoiceConfig as SchemaVoiceConfig;

/// Configuration resource for voice playback rules.
///
/// 语音播放规则的配置资源。
///
/// Loaded from the `voice` section of `narrative/dialogue.ron` via
/// [`resolve_path`](crate::config::resolve_path).
/// Contains named presets that map characters to playback rules.
/// Characters in a preset with value `false` will suppress voice.
/// Characters not in the preset will play voice normally.
///
/// 从 `narrative/dialogue.ron` 的 `voice` 段加载（通过
/// [`resolve_path`](crate::config::resolve_path)）。
/// 包含命名预设，将字符映射到播放规则。
/// 预设中值为 `false` 的字符将抑制语音。
/// 不在预设中的字符将正常播放语音。
#[derive(Resource, Debug, Clone, Default)]
pub struct VoiceConfig(pub SchemaVoiceConfig);

impl Deref for VoiceConfig {
    type Target = SchemaVoiceConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VoiceConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl VoiceConfig {
    /// Returns the rules for the given preset name, falling back to `default_preset`.
    ///
    /// 返回指定预设名称的规则，回退到 `default_preset`。
    pub fn active_rules(&self, preset_name: Option<&str>) -> Option<&HashMap<String, bool>> {
        let name = preset_name.unwrap_or(&self.default_preset);
        self.presets.get(name)
    }

    /// Whether voice should play for the given character under the active preset.
    ///
    /// 在当前预设下，给定字符是否应播放语音。
    ///
    /// Returns `false` if the character is mapped to `false` in the preset.
    /// Returns `true` for characters not in the preset (default: play voice).
    ///
    /// 若字符在预设中被映射为 `false` 则返回 `false`。
    /// 不在预设中的字符返回 `true`（默认：播放语音）。
    pub fn should_play(&self, preset_name: Option<&str>, ch: &str) -> bool {
        let Some(rules) = self.active_rules(preset_name) else {
            return true;
        };
        !matches!(rules.get(ch), Some(false))
    }
}
