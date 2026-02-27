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
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::path::Path;
use tracing::warn;

/// Supported audio extensions for automatic detection.
///
/// 自动检测支持的音频扩展名。
const AUDIO_EXTENSIONS: &[&str] = &["wav", "ogg", "mp3", "flac"];

/// Resolve a sound name to its asset path by searching in the audios directory.
///
/// 通过搜索 audios 目录将音效名称解析为资源路径。
fn resolve_sound_path(sound_name: &str) -> Option<String> {
    let config = load_config();
    let mod_name = &config.project.mod_name;
    let audios_dir = &config.resources.audios;

    let audios_root = crate::config::get_projects_base_path()
        .join(mod_name)
        .join(audios_dir);

    if !audios_root.exists() {
        warn!("Audios directory does not exist: {:?}", audios_root);
        return None;
    }

    let stem = Path::new(sound_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(sound_name);

    let has_extension = Path::new(sound_name)
        .extension()
        .map(|e| AUDIO_EXTENSIONS.contains(&e.to_str().unwrap_or("")))
        .unwrap_or(false);

    if let Some(found) =
        search_audio_recursive(&audios_root, stem, has_extension.then_some(sound_name))
    {
        // Strip the assets root prefix to get a path relative to asset source roots
        let assets_prefix = crate::config::get_projects_base_path()
            .join(mod_name)
            .join("assets");
        let asset_path = found.strip_prefix(&assets_prefix).unwrap_or(&found);
        return Some(asset_path.to_string_lossy().to_string());
    }

    warn!(
        "Sound file not found: {} (searched in {:?})",
        sound_name, audios_root
    );
    None
}

/// Recursively search for an audio file in a directory.
///
/// 在目录中递归搜索音频文件。
fn search_audio_recursive(
    dir: &Path,
    stem: &str,
    exact_name: Option<&str>,
) -> Option<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return None;
    };

    let mut results = Vec::new();

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if path.is_file() {
            if let Some(exact) = exact_name {
                if file_name == exact {
                    return Some(path);
                }
            } else {
                let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let file_ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                if file_stem == stem && AUDIO_EXTENSIONS.contains(&file_ext) {
                    results.push(path.clone());
                }
            }
        } else if path.is_dir()
            && !file_name.starts_with('.')
            && let Some(found) = search_audio_recursive(&path, stem, exact_name)
        {
            return Some(found);
        }
    }

    if results.len() > 1 {
        warn!(
            "Multiple audio files found for '{}': {:?}. Using first match.",
            stem, results
        );
    }

    results.into_iter().next()
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
