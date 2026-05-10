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
use bevy_kira_audio::AudioControl;
use tracing::trace;

use crate::core::audio::{AudioSourceCache, load_state_label};
use crate::core::dialogue::voice_config::VoiceConfig;
use crate::core::dialogue::{DialogueChannel, TypewriterVoice};
use crate::core::fre_facts;

struct VoicePlayContext<'a> {
    audio: &'a bevy_kira_audio::Audio,
    asset_server: &'a AssetServer,
    audio_cache: &'a mut AudioSourceCache,
    time: &'a Time,
    channel_name: &'a str,
    last_char_index: usize,
    current_char_index: usize,
    advanced: usize,
    revealed_index: usize,
    revealed_char: &'a Option<String>,
    preset_name: Option<&'a str>,
}

pub fn typewriter_voice_system(
    config: Res<VoiceConfig>,
    facts: Res<LayeredFactDatabase>,
    mut query: Query<(&Typewriter, &mut TypewriterVoice, Option<&DialogueChannel>)>,
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    mut audio_cache: ResMut<AudioSourceCache>,
    time: Res<Time>,
) {
    for (typewriter, mut voice, channel) in query.iter_mut() {
        let channel_name = channel.map(|c| c.name.as_str()).unwrap_or("<global>");
        let enabled = channel
            .and_then(|c| {
                facts.get_bool(&fre_facts::dialogue_channel_key(&c.name, "voice_enabled"))
            })
            .or_else(|| facts.get_bool(fre_facts::DIALOGUE_VOICE_ENABLED))
            .unwrap_or(true);
        if !enabled {
            // Sync indices even when disabled to prevent stale sounds on re-enable.
            // 禁用时也同步索引，防止重新启用时触发残留音效。
            if voice.last_char_index != typewriter.current_char_index {
                trace!(
                    "voice_sync_disabled t={:.3} channel='{}' last={} current={}",
                    time.elapsed_secs_f64(),
                    channel_name,
                    voice.last_char_index,
                    typewriter.current_char_index
                );
            }
            voice.last_char_index = typewriter.current_char_index;
            continue;
        }

        let preset_name = channel
            .and_then(|c| {
                facts.get_string(&fre_facts::dialogue_channel_key(&c.name, "voice_preset"))
            })
            .or_else(|| facts.get_string(fre_facts::DIALOGUE_VOICE_PRESET));

        if typewriter.state != TypewriterState::Playing {
            // Keep index in sync so resuming doesn't trigger a stale sound.
            // 保持索引同步，避免恢复时触发多余音效。
            if voice.last_char_index != typewriter.current_char_index {
                trace!(
                    "voice_sync_not_playing t={:.3} channel='{}' state={:?} last={} current={}",
                    time.elapsed_secs_f64(),
                    channel_name,
                    typewriter.state,
                    voice.last_char_index,
                    typewriter.current_char_index
                );
            }
            voice.last_char_index = typewriter.current_char_index;
            continue;
        }

        if typewriter.current_char_index > voice.last_char_index {
            let advanced = typewriter.current_char_index - voice.last_char_index;
            let revealed_index = typewriter.current_char_index.saturating_sub(1);
            let revealed_char = typewriter
                .source_text
                .chars()
                .nth(revealed_index)
                .map(|ch| ch.to_string());
            let should_play = revealed_char
                .as_ref()
                .is_some_and(|ch| config.should_play(preset_name, ch));

            if should_play {
                play_voice_sound(
                    &voice.sound_path,
                    VoicePlayContext {
                        audio: &audio,
                        asset_server: &asset_server,
                        audio_cache: &mut audio_cache,
                        time: &time,
                        channel_name,
                        last_char_index: voice.last_char_index,
                        current_char_index: typewriter.current_char_index,
                        advanced,
                        revealed_index,
                        revealed_char: &revealed_char,
                        preset_name,
                    },
                );
            } else {
                trace!(
                    "voice_skip t={:.3} channel='{}' last={} current={} advanced={} revealed_index={} char={:?} preset={:?} reason=filtered_or_missing_char",
                    time.elapsed_secs_f64(),
                    channel_name,
                    voice.last_char_index,
                    typewriter.current_char_index,
                    advanced,
                    revealed_index,
                    revealed_char,
                    preset_name
                );
            }
            voice.last_char_index = typewriter.current_char_index;
        } else if typewriter.current_char_index < voice.last_char_index {
            trace!(
                "voice_reset t={:.3} channel='{}' last={} current={} path='{}'",
                time.elapsed_secs_f64(),
                channel_name,
                voice.last_char_index,
                typewriter.current_char_index,
                voice.sound_path
            );
            voice.last_char_index = typewriter.current_char_index;
        }
    }
}

fn play_voice_sound(sound_path: &str, ctx: VoicePlayContext<'_>) {
    let (handle, inserted) = ctx
        .audio_cache
        .cached_audio_source_handle(sound_path, |path| ctx.asset_server.load(path.to_string()));
    let asset_id = handle.id();
    let load_state = load_state_label(ctx.asset_server.load_state(asset_id));
    let known_path = ctx
        .asset_server
        .get_path(asset_id.untyped())
        .map(|path| path.to_string())
        .unwrap_or_else(|| "<unknown>".to_string());
    let cache_status = if inserted { "miss" } else { "hit" };
    let mut command = ctx.audio.play(handle);
    let instance_handle = command.handle();

    trace!(
        "voice_play t={:.3} channel='{}' last={} current={} advanced={} revealed_index={} char={:?} preset={:?} path='{}' cache={} known_path='{}' asset_id={} load_state={} instance_id={}",
        ctx.time.elapsed_secs_f64(),
        ctx.channel_name,
        ctx.last_char_index,
        ctx.current_char_index,
        ctx.advanced,
        ctx.revealed_index,
        ctx.revealed_char,
        ctx.preset_name,
        sound_path,
        cache_status,
        known_path,
        asset_id,
        load_state,
        instance_handle.id()
    );
}
