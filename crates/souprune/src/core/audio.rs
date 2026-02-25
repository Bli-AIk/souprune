//! # audio.rs
//!
//! # audio.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module provides audio playback system using bevy_kira_audio or bevy_seedling.
//!
//! 本模块使用 bevy_kira_audio 或 bevy_seedling 提供音频播放系统。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It handles playing sound effects and music with support for multiple formats.
//!
//! 处理音效和音乐播放，支持多种音频格式。
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
use std::path::Path;
use tracing::warn;

/// Supported audio extensions for automatic detection.
///
/// 自动检测支持的音频扩展名。
const AUDIO_EXTENSIONS: &[&str] = &["wav", "ogg", "mp3", "flac"];

/// Resolve a sound name to its asset path by searching in the audios directory.
///
/// 通过搜索 audios 目录将音效名称解析为资源路径。
///
/// # Path Resolution
///
/// 1. Check if the name already has an extension → search for exact match
/// 2. Try adding common audio extensions
/// 3. Search recursively in the entire audios directory
///
/// # 路径解析
///
/// 1. 检查名称是否已有扩展名 → 搜索精确匹配
/// 2. 尝试添加常见音频扩展名
/// 3. 在整个 audios 目录中递归搜索
fn resolve_sound_path(sound_name: &str) -> Option<String> {
    let config = load_config();
    let mod_name = &config.project.mod_name;
    let audios_dir = &config.resources.audios;

    // Build the audios directory path
    let audios_root = crate::config::get_projects_base_path()
        .join(mod_name)
        .join(audios_dir);

    if !audios_root.exists() {
        warn!("Audios directory does not exist: {:?}", audios_root);
        return None;
    }

    // Get the stem (filename without extension) for searching
    let stem = Path::new(sound_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(sound_name);

    // Check if the name already has an audio extension
    let has_extension = Path::new(sound_name)
        .extension()
        .map(|e| AUDIO_EXTENSIONS.contains(&e.to_str().unwrap_or("")))
        .unwrap_or(false);

    // Search for the file
    if let Some(found) =
        search_audio_recursive(&audios_root, stem, has_extension.then_some(sound_name))
    {
        // Convert to asset path relative to assets directory
        let asset_path = found
            .strip_prefix(format!("projects/{}/assets/", mod_name))
            .unwrap_or(&found);
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
            // If we have an exact name to match
            if let Some(exact) = exact_name {
                if file_name == exact {
                    return Some(path);
                }
            } else {
                // Match by stem with any audio extension
                let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let file_ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                if file_stem == stem && AUDIO_EXTENSIONS.contains(&file_ext) {
                    results.push(path.clone());
                }
            }
        } else if path.is_dir() && !file_name.starts_with('.') {
            // Recurse into subdirectories
            if let Some(found) = search_audio_recursive(&path, stem, exact_name) {
                return Some(found);
            }
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
// Kira Audio Backend (default)
// ============================================================================
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
mod kira_backend {
    use super::*;
    use bevy_kira_audio::prelude::*;

    pub(crate) struct AudioPluginImpl;

    impl Plugin for AudioPluginImpl {
        fn build(&self, app: &mut App) {
            app.add_plugins(bevy_kira_audio::prelude::AudioPlugin);
        }
    }

    pub fn play_sound_impl(audio: &Audio, asset_server: &AssetServer, sound_name: &str) {
        if let Some(path) = super::resolve_sound_path(sound_name) {
            let sound_handle = asset_server.load(path);
            audio.play(sound_handle);
        }
    }

    pub fn play_bgm_impl(
        audio: &Audio,
        asset_server: &AssetServer,
        music_path: &str,
    ) -> Handle<AudioInstance> {
        // Use the same path resolution as sound effects
        // 使用与音效相同的路径解析
        let resolved_path = super::resolve_sound_path(music_path).unwrap_or_else(|| {
            warn!("BGM file not found: {}, using path as-is", music_path);
            music_path.to_string()
        });
        let music_handle = asset_server.load(resolved_path);
        audio.play(music_handle).looped().handle()
    }
}

#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub use kira_backend::*;

// ============================================================================
// Seedling Audio Backend (firewheel)
// ============================================================================
#[cfg(feature = "firewheel")]
mod seedling_backend {
    use bevy::prelude::*;
    use bevy_seedling::prelude::*;
    use bevy_seedling::sample::SampleQueueLifetime;

    /// Pool label for background music - prevents automatic cleanup
    #[derive(PoolLabel, PartialEq, Eq, Debug, Hash, Clone, Copy)]
    pub struct BgmPool;

    /// Pool label for sound effects - with larger pool size to prevent interruption
    #[derive(PoolLabel, PartialEq, Eq, Debug, Hash, Clone, Copy)]
    pub struct SfxPool;

    pub(crate) struct AudioPluginImpl;

    impl Plugin for AudioPluginImpl {
        fn build(&self, app: &mut App) {
            // Configure FirewheelConfig with minimal/disabled declick for crisp short sound effects
            // 配置 FirewheelConfig，禁用 declick 以获得清晰的短音效
            // Default is 10ms which is WAY too long for short SFX (like choice.wav at ~35ms)
            // 默认的 10ms 对于短音效来说太长了（如 choice.wav 只有约 35ms）
            let config = FirewheelConfig {
                // Disable declick completely for crisp, unmodified sound
                // 完全禁用 declick 以获得原始清晰的声音
                // Setting to 0.0 will result in minimum 1 frame (practically disabled)
                // 设置为 0.0 会使用最小 1 帧（实际上相当于禁用）
                declick_seconds: 0.0,
                ..Default::default()
            };

            let plugin = SeedlingPlugin {
                config,
                ..Default::default()
            };

            app.add_plugins(plugin)
                .add_systems(Startup, setup_audio_pools);
        }
    }

    /// Set up audio pools for BGM and SFX with optimized configurations
    ///
    /// 为 BGM 和 SFX 设置音频池，使用优化的配置
    fn setup_audio_pools(mut commands: Commands) {
        // Create BGM pool with fixed size - BGM doesn't need many samplers
        // 创建 BGM 池，固定大小 - BGM 不需要很多采样器
        commands.spawn((
            Name::new("BGM Pool"),
            SamplerPool(BgmPool),
            PoolSize(4..=4), // Fixed pool size for BGM
        ));

        // Create SFX pool with large fixed size to prevent sound stealing/interruption
        // 创建 SFX 池，使用大的固定大小以防止音效被抢占/中断
        // Pool size is fixed (64..=64) to ensure enough samplers for rapid-fire sounds
        // 池大小固定为 64..=64，确保有足够的采样器处理快速连续的音效
        // This prevents audio truncation when playing multiple sounds rapidly
        // 这可以防止快速播放多个音效时出现截断
        commands.spawn((
            Name::new("SFX Pool"),
            SamplerPool(SfxPool),
            PoolSize(64..=64), // Larger fixed pool size to handle rapid-fire sounds
            SamplerConfig {
                // Disable declickers for SFX pool to preserve original sound
                // 为 SFX 池禁用 declicker 以保留原始声音
                num_declickers: 0,
                ..Default::default()
            },
        ));
    }

    pub fn play_sound_impl(commands: &mut Commands, asset_server: &AssetServer, sound_name: &str) {
        if let Some(path) = super::resolve_sound_path(sound_name) {
            let sound_handle = asset_server.load(path);
            // One-shot sounds go to SFX pool with full volume
            // 一次性音效进入 SFX 池，使用全音量
            commands.spawn((
                Name::new(format!("SFX: {}", sound_name)),
                SamplePlayer::new(sound_handle).with_volume(Volume::Decibels(0.0)), // Full volume (0dB = no attenuation)
                SfxPool,
                // Extend queue lifetime to prevent sounds from being skipped during resource loading
                // 延长队列生命周期，防止音效在资源加载期间被跳过
                SampleQueueLifetime(std::time::Duration::from_secs(10)),
            ));
        }
    }

    pub fn play_bgm_impl(
        commands: &mut Commands,
        asset_server: &AssetServer,
        music_path: &str,
    ) -> Entity {
        // Use the same path resolution as sound effects
        // 使用与音效相同的路径解析
        let resolved_path = super::resolve_sound_path(music_path).unwrap_or_else(|| {
            warn!("BGM file not found: {}, using path as-is", music_path);
            music_path.to_string()
        });
        let music_handle = asset_server.load(resolved_path);
        // BGM with looping enabled, goes to BGM pool
        // 带循环的 BGM，进入 BGM 池
        commands
            .spawn((
                Name::new(format!("BGM: {}", music_path)),
                SamplePlayer::new(music_handle)
                    .looping()
                    .with_volume(Volume::Decibels(0.0)), // Full volume (0dB = no attenuation)
                BgmPool,
            ))
            .id()
    }
}

#[cfg(feature = "firewheel")]
pub use seedling_backend::*;

// ============================================================================
// Public API
// ============================================================================

/// Audio plugin that sets up the audio system.
///
/// 设置音频系统的插件。
pub(crate) struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPluginImpl);
    }
}

/// Play a sound effect by searching for it in the audios directory.
///
/// 通过在 audios 目录中搜索来播放音效。
///
/// Sound names are resolved using the audios path from mod.toml.
/// The file is searched recursively in the entire `{audios}` directory.
/// Extension is optional - "choice" or "choice.wav" both work.
///
/// 音效名称使用 mod.toml 中的 audios 路径解析。
/// 文件在整个 `{audios}` 目录中递归搜索。
/// 扩展名可选 - "choice" 或 "choice.wav" 都可以。
///
/// # Example with Kira
/// ```ignore
/// play_sound(&audio, &asset_server, "choice");
/// play_sound(&audio, &asset_server, "choice.wav");
/// ```
///
/// # Example with Seedling
/// ```ignore
/// play_sound(&mut commands, &asset_server, "choice");
/// play_sound(&mut commands, &asset_server, "choice.wav");
/// ```
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub fn play_sound(audio: &bevy_kira_audio::Audio, asset_server: &AssetServer, sound_name: &str) {
    play_sound_impl(audio, asset_server, sound_name);
}

#[cfg(feature = "firewheel")]
pub fn play_sound(commands: &mut Commands, asset_server: &AssetServer, sound_name: &str) {
    play_sound_impl(commands, asset_server, sound_name);
}

/// Play a sound effect using a full asset path (no prefix added).
///
/// 使用完整资源路径播放音效（不添加前缀）。
///
/// Use this for configuration-driven sounds where the full path is specified.
///
/// 用于配置驱动的音效，其中已指定完整路径。
///
/// # Example with Kira
/// ```ignore
/// play_sound_full_path(&audio, &asset_server, "assets/audios/sfx/confirm.wav");
/// ```
///
/// # Example with Seedling
/// ```ignore
/// play_sound_full_path(&mut commands, &asset_server, "assets/audios/sfx/confirm.wav");
/// ```
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub fn play_sound_full_path(
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
    sound_path: &str,
) {
    use bevy_kira_audio::prelude::*;
    let sound_handle = asset_server.load(sound_path.to_string());
    audio.play(sound_handle);
}

#[cfg(feature = "firewheel")]
pub fn play_sound_full_path(commands: &mut Commands, asset_server: &AssetServer, sound_path: &str) {
    use bevy_seedling::prelude::*;
    use bevy_seedling::sample::SampleQueueLifetime;

    let sound_handle = asset_server.load(sound_path.to_string());
    commands.spawn((
        Name::new(format!("SFX: {}", sound_path)),
        SamplePlayer::new(sound_handle).with_volume(Volume::Decibels(0.0)),
        SfxPool,
        SampleQueueLifetime(std::time::Duration::from_secs(10)),
    ));
}

/// Play background music from the assets/audios/music directory with looping.
///
/// 从 assets/audios/music 目录播放循环的背景音乐。
///
/// # Example with Kira
/// ```ignore
/// let handle = play_bgm(&audio, &asset_server, "mus_ruins.ogg");
/// ```
///
/// # Example with Seedling
/// ```ignore
/// let entity = play_bgm(&mut commands, &asset_server, "mus_ruins.ogg");
/// ```
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub fn play_bgm(
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
    music_path: &str,
) -> Handle<bevy_kira_audio::AudioInstance> {
    play_bgm_impl(audio, asset_server, music_path)
}

#[cfg(feature = "firewheel")]
pub fn play_bgm(commands: &mut Commands, asset_server: &AssetServer, music_path: &str) -> Entity {
    play_bgm_impl(commands, asset_server, music_path)
}
