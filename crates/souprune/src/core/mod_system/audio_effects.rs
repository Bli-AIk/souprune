//! Applies audio side effects requested by WASM mods.
//!
//! 提交 WASM 模组请求的音频副作用。

use bevy::prelude::*;

use crate::core::audio::{self, AudioSourceCache};
use crate::core::wasm_runtime::PendingSoundEffect;

/// Audio effects waiting for Bevy audio playback.
///
/// 等待 Bevy 音频系统播放的音频副作用。
#[derive(Resource, Default)]
pub(crate) struct PendingAudioEffects {
    effects: Vec<PendingSoundEffect>,
}

impl PendingAudioEffects {
    pub(crate) fn extend(&mut self, effects: Vec<PendingSoundEffect>) {
        self.effects.extend(effects);
    }
}

pub(super) fn flush_audio_effects_system(
    audio_player: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    mut audio_cache: ResMut<AudioSourceCache>,
    mut pending: ResMut<PendingAudioEffects>,
) {
    let effects = std::mem::take(&mut pending.effects);
    for effect in effects {
        match effect {
            PendingSoundEffect::SoundKey(sound_key) => {
                audio::play_sound(&audio_player, &asset_server, &mut audio_cache, &sound_key);
            }
            PendingSoundEffect::FullPath(sound_path) => {
                audio::play_sound_full_path(
                    &audio_player,
                    &asset_server,
                    &mut audio_cache,
                    &sound_path,
                );
            }
        }
    }
}
