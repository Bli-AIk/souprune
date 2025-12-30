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

    pub(crate) struct AudioPluginImpl;

    impl Plugin for AudioPluginImpl {
        fn build(&self, app: &mut App) {
            app.add_plugins(SeedlingPlugin::default());
        }
    }

    /// Resource for tracking BGM entity
    ///
    /// 用于跟踪 BGM 实体的资源
    #[derive(Resource, Default)]
    pub struct BgmEntity(pub Option<Entity>);

    pub fn play_sound_impl(
        commands: &mut Commands,
        asset_server: &AssetServer,
        sound_path: &str,
    ) {
        let sound_handle = asset_server.load(format!("audios/sfx/{}", sound_path));
        commands.spawn((
            SamplePlayer::new(sound_handle),
            sample_effects![VolumeNode {
                volume: Volume::from_percent(0.7),
                ..default()
            }],
        ));
    }

    pub fn play_bgm_impl(
        commands: &mut Commands,
        asset_server: &AssetServer,
        music_path: &str,
    ) -> Entity {
        let music_handle = asset_server.load(format!("audios/music/{}", music_path));
        commands
            .spawn((
                SamplePlayer::new(music_handle).looping(),
                sample_effects![VolumeNode {
                    volume: Volume::from_percent(0.7),
                    ..default()
                }],
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

        #[cfg(feature = "experimental")]
        app.init_resource::<BgmEntity>();
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
