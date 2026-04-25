//! Plays per-character dialogue voice sounds while a typewriter is revealing text.
//!
//! 在打字机逐字显示文本时播放对应的逐字语音音效。
//!
//! Keeps a deliberately narrow scope: it watches typewriter
//! progress, compares the newly revealed character index with the previous one,
//! and triggers the configured voice sound whenever another character appears.
//! Which characters play or suppress voice is driven by [`VoiceConfig`] presets
//! loaded from `narrative/dialogue.ron`.
//!
//! 刻意把职责收得很窄：它监视打字机的推进进度，对比这次新显示的字符
//! 索引与上一次的差值，并在出现新的字符时触发配置好的语音音效。
//! 哪些字符播放或抑制语音由 [`VoiceConfig`] 预设驱动，
//! 从 `narrative/dialogue.ron` 加载。

use bevy::prelude::*;
use bevy_ecs_typewriter::{Typewriter, TypewriterState};
use bevy_fact_rule_event::LayeredFactDatabase;

use crate::core::dialogue::TypewriterVoice;
use crate::core::dialogue::voice_config::VoiceConfig;
use crate::core::fre_facts;

pub fn typewriter_voice_system(
    config: Res<VoiceConfig>,
    facts: Res<LayeredFactDatabase>,
    mut query: Query<(&Typewriter, &mut TypewriterVoice)>,
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
) {
    use bevy_kira_audio::AudioControl;

    let enabled = facts
        .get_bool(fre_facts::DIALOGUE_VOICE_ENABLED)
        .unwrap_or(true);
    if !enabled {
        // Sync indices even when disabled to prevent stale sounds on re-enable.
        // 禁用时也同步索引，防止重新启用时触发残留音效。
        for (typewriter, mut voice) in query.iter_mut() {
            voice.last_char_index = typewriter.current_char_index;
        }
        return;
    }

    let preset_name = facts.get_string(fre_facts::DIALOGUE_VOICE_PRESET);

    for (typewriter, mut voice) in query.iter_mut() {
        if typewriter.state != TypewriterState::Playing {
            // Keep index in sync so resuming doesn't trigger a stale sound.
            // 保持索引同步，避免恢复时触发多余音效。
            voice.last_char_index = typewriter.current_char_index;
            continue;
        }

        if typewriter.current_char_index > voice.last_char_index {
            let should_play = typewriter
                .source_text
                .chars()
                .nth(typewriter.current_char_index.saturating_sub(1))
                .is_some_and(|ch| config.should_play(preset_name, &ch.to_string()));

            if should_play {
                let handle: Handle<bevy_kira_audio::AudioSource> =
                    asset_server.load(&voice.sound_path);
                audio.play(handle);
            }
            voice.last_char_index = typewriter.current_char_index;
        } else if typewriter.current_char_index < voice.last_char_index {
            voice.last_char_index = typewriter.current_char_index;
        }
    }
}
