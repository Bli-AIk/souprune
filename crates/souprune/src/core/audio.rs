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
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use tracing::warn;

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
        app.add_plugins(bevy_kira_audio::prelude::AudioPlugin);
    }
}

/// Play a sound effect by searching for it in the audios directory.
///
/// 通过在 audios 目录中搜索来播放音效。
pub fn play_sound(audio: &Audio, asset_server: &AssetServer, sound_name: &str) {
    if let Some(path) = resolve_sound_path(sound_name) {
        let sound_handle = asset_server.load(path);
        audio.play(sound_handle);
    }
}

/// Play a sound effect using a full asset path (no prefix added).
///
/// 使用完整资源路径播放音效（不添加前缀）。
pub fn play_sound_full_path(audio: &Audio, asset_server: &AssetServer, sound_path: &str) {
    let sound_handle = asset_server.load(sound_path.to_string());
    audio.play(sound_handle);
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
