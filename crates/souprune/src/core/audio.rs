//! # audio.rs
//!
//! # audio.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module provides audio playback system using bevy_kira_audio.
//!
//! 本模块使用 bevy_kira_audio 提供音频播放系统。
//!
//! ## Path Resolution
//!
//! ## 路径解析
//!
//! Sound names are resolved using the audios directory from mod.toml:
//! - Simple name: `"choice.wav"` → searches in entire `{audios}` directory recursively
//! - Extension optional: `"choice"` or `"choice.wav"` both work
//!
//! 音效名称使用 mod.toml 中的 audios 目录解析：
//! - 简单名称：`"choice.wav"` → 在整个 `{audios}` 目录中递归搜索
//! - 扩展名可选：`"choice"` 或 `"choice.wav"` 都可以

use crate::config::load_config;
use crate::core::resource_resolver;
use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_kira_audio::AudioSource as KiraAudioSource;
use bevy_kira_audio::prelude::*;
use std::collections::HashMap;
use tracing::{trace, warn};

/// Supported audio extensions for automatic detection.
///
/// 自动检测支持的音频扩展名。
const AUDIO_EXTENSIONS: &[&str] = &["wav", "ogg", "mp3", "flac"];

/// Resolve a sound name to its asset path by searching in the audios directory.
/// Searches main mod first, then dependency mods.
///
/// 通过搜索 audios 目录将音效名称解析为资源路径。
/// 先搜索主 mod，再搜索依赖 mod。
fn resolve_sound_path(sound_name: &str) -> Option<String> {
    let config = load_config();
    let category_dir = &config.resources.audios;

    let resolved = resource_resolver::resolve_resource(category_dir, sound_name, AUDIO_EXTENSIONS)?;
    Some(resource_resolver::to_relative_asset_path(&resolved))
}

// ============================================================================
// Public API
// ============================================================================

/// Audio plugin that sets up the audio system.
///
/// 设置音频系统的插件。
pub(crate) struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        let settings = audio_settings();
        trace!(
            "backend_settings sound_capacity={} android_cpal_probe={}",
            settings.sound_capacity,
            cfg!(target_os = "android")
        );
        app.insert_resource(settings);
        app.init_resource::<AudioSourceCache>();
        app.add_plugins(bevy_kira_audio::prelude::AudioPlugin);
    }
}

fn audio_settings() -> AudioSettings {
    AudioSettings {
        sound_capacity: 256,
    }
}

/// Strong handle cache for short sound effects.
///
/// 短音效的强引用句柄缓存。
#[derive(Resource, Default)]
pub struct AudioSourceCache {
    handles: HashMap<String, Handle<KiraAudioSource>>,
}

impl AudioSourceCache {
    pub(crate) fn cached_audio_source_handle(
        &mut self,
        path: &str,
        load: impl FnOnce(&str) -> Handle<KiraAudioSource>,
    ) -> (Handle<KiraAudioSource>, bool) {
        use std::collections::hash_map::Entry;

        match self.handles.entry(path.to_string()) {
            Entry::Occupied(entry) => (entry.get().clone(), false),
            Entry::Vacant(entry) => {
                let handle = load(path);
                entry.insert(handle.clone());
                (handle, true)
            }
        }
    }
}

pub(crate) fn load_state_label(load_state: LoadState) -> &'static str {
    match load_state {
        LoadState::NotLoaded => "not_loaded",
        LoadState::Loading => "loading",
        LoadState::Loaded => "loaded",
        LoadState::Failed(_) => "failed",
    }
}

fn audio_source_probe(
    path: &str,
    handle: &Handle<KiraAudioSource>,
    asset_server: &AssetServer,
) -> String {
    let asset_id = handle.id();
    let load_state = load_state_label(asset_server.load_state(asset_id));
    let known_path = asset_server
        .get_path(asset_id.untyped())
        .map(|path| path.to_string())
        .unwrap_or_else(|| "<unknown>".to_string());
    format!(
        "path='{}' known_path='{}' id={} load_state={}",
        path, known_path, asset_id, load_state
    )
}

/// Play a sound effect by searching for it in the audios directory.
///
/// 通过在 audios 目录中搜索来播放音效。
pub fn play_sound(
    audio: &Audio,
    asset_server: &AssetServer,
    audio_cache: &mut AudioSourceCache,
    sound_name: &str,
) {
    if let Some(path) = resolve_sound_path(sound_name) {
        let (sound_handle, inserted) = audio_cache
            .cached_audio_source_handle(&path, |path| asset_server.load(path.to_string()));
        let mut command = audio.play(sound_handle.clone());
        let instance_handle = command.handle();
        trace!(
            "request source=play_sound sound_name='{}' cache={} {} instance_id={}",
            sound_name,
            if inserted { "miss" } else { "hit" },
            audio_source_probe(&path, &sound_handle, asset_server),
            instance_handle.id()
        );
    }
}

/// Play a sound effect using a full asset path (no prefix added).
///
/// 使用完整资源路径播放音效（不添加前缀）。
pub fn play_sound_full_path(
    audio: &Audio,
    asset_server: &AssetServer,
    audio_cache: &mut AudioSourceCache,
    sound_path: &str,
) {
    let (sound_handle, inserted) = audio_cache
        .cached_audio_source_handle(sound_path, |path| asset_server.load(path.to_string()));
    let mut command = audio.play(sound_handle.clone());
    let instance_handle = command.handle();
    trace!(
        "request source=play_sound_full_path cache={} {} instance_id={}",
        if inserted { "miss" } else { "hit" },
        audio_source_probe(sound_path, &sound_handle, asset_server),
        instance_handle.id()
    );
}

/// Play background music with looping.
///
/// 播放循环的背景音乐。
pub fn play_bgm(
    audio: &Audio,
    asset_server: &AssetServer,
    music_path: &str,
) -> Handle<AudioInstance> {
    let resolved_path = resolve_sound_path(music_path).unwrap_or_else(|| {
        warn!("BGM file not found: {}, using path as-is", music_path);
        music_path.to_string()
    });
    let music_handle = asset_server.load(resolved_path);
    audio.play(music_handle).looped().handle()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_settings_allow_dense_sound_effect_playback() {
        assert_eq!(audio_settings().sound_capacity, 256);
    }

    #[test]
    fn audio_source_cache_loads_each_path_once() {
        let mut cache = AudioSourceCache::default();
        let mut load_count = 0;

        let (_, inserted) = cache.cached_audio_source_handle("voice.wav", |_| {
            load_count += 1;
            Handle::<KiraAudioSource>::default()
        });
        assert!(inserted);

        let (_, inserted) = cache.cached_audio_source_handle("voice.wav", |_| {
            load_count += 1;
            Handle::<KiraAudioSource>::default()
        });
        assert!(!inserted);
        assert_eq!(load_count, 1);
    }
}
