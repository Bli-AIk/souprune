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

use bevy::prelude::*;

// ============================================================================
// Kira Audio Backend (default)
// ============================================================================
#[cfg(all(feature = "bevy_kira_audio", not(feature = "experimental")))]
mod kira_backend {
    use bevy::prelude::*;
    use bevy_kira_audio::prelude::*;

    pub(crate) struct AudioPluginImpl;

    impl Plugin for AudioPluginImpl {
        fn build(&self, app: &mut App) {
            app.add_plugins(bevy_kira_audio::AudioPlugin);
        }
    }

    pub fn play_sound_impl(audio: &Audio, asset_server: &AssetServer, sound_path: &str) {
        let sound_handle = asset_server.load(format!("audios/sfx/{}", sound_path));
        audio.play(sound_handle);
    }

    pub fn play_bgm_impl(
        audio: &Audio,
        asset_server: &AssetServer,
        music_path: &str,
    ) -> Handle<AudioInstance> {
        let music_handle = asset_server.load(format!("audios/music/{}", music_path));
        audio.play(music_handle).looped().handle()
    }
}

#[cfg(all(feature = "bevy_kira_audio", not(feature = "experimental")))]
pub use kira_backend::*;

// ============================================================================
// Seedling Audio Backend (experimental)
// ============================================================================
#[cfg(feature = "experimental")]
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

            app.add_plugins(plugin).add_systems(Startup, setup_audio_pools);
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

    pub fn play_sound_impl(
        commands: &mut Commands,
        asset_server: &AssetServer,
        sound_path: &str,
    ) {
        let sound_handle = asset_server.load(format!("audios/sfx/{}", sound_path));
        // One-shot sounds go to SFX pool with full volume
        // 一次性音效进入 SFX 池，使用全音量
        commands.spawn((
            Name::new(format!("SFX: {}", sound_path)),
            SamplePlayer::new(sound_handle)
                .with_volume(Volume::Decibels(0.0)), // Full volume (0dB = no attenuation)
            SfxPool,
            // Extend queue lifetime to prevent sounds from being skipped during resource loading
            // 延长队列生命周期，防止音效在资源加载期间被跳过
            SampleQueueLifetime(std::time::Duration::from_secs(10)),
        ));
    }

    pub fn play_bgm_impl(
        commands: &mut Commands,
        asset_server: &AssetServer,
        music_path: &str,
    ) -> Entity {
        let music_handle = asset_server.load(format!("audios/music/{}", music_path));
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

#[cfg(feature = "experimental")]
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

/// Play a sound effect from the assets/audios/sfx directory.
///
/// 从 assets/audios/sfx 目录播放音效。
///
/// # Example with Kira
/// ```ignore
/// play_sound(&audio, &asset_server, "choice.wav");
/// ```
///
/// # Example with Seedling
/// ```ignore
/// play_sound(&mut commands, &asset_server, "choice.wav");
/// ```
#[cfg(all(feature = "bevy_kira_audio", not(feature = "experimental")))]
pub fn play_sound(audio: &bevy_kira_audio::Audio, asset_server: &AssetServer, sound_path: &str) {
    play_sound_impl(audio, asset_server, sound_path);
}

#[cfg(feature = "experimental")]
pub fn play_sound(commands: &mut Commands, asset_server: &AssetServer, sound_path: &str) {
    play_sound_impl(commands, asset_server, sound_path);
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
#[cfg(all(feature = "bevy_kira_audio", not(feature = "experimental")))]
pub fn play_bgm(
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
    music_path: &str,
) -> Handle<bevy_kira_audio::AudioInstance> {
    play_bgm_impl(audio, asset_server, music_path)
}

#[cfg(feature = "experimental")]
pub fn play_bgm(commands: &mut Commands, asset_server: &AssetServer, music_path: &str) -> Entity {
    play_bgm_impl(commands, asset_server, music_path)
}
