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
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It handles playing sound effects and music with support for multiple formats.
//!
//! 处理音效和音乐播放，支持多种音频格式。

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

/// Audio plugin that sets up the audio system.
///
/// 设置音频系统的插件。
pub(crate) struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_kira_audio::AudioPlugin);
    }
}

/// Play a sound effect from the assets/audios/sfx directory.
///
/// 从 assets/audios/sfx 目录播放音效。
///
/// # Example
/// ```ignore
/// play_sound(&audio, &asset_server, "choice.wav");
/// ```
pub fn play_sound(audio: &Audio, asset_server: &AssetServer, sound_path: &str) {
    let sound_handle = asset_server.load(format!("audios/sfx/{}", sound_path));
    audio.play(sound_handle);
}

/// Play background music from the assets/audios/music directory with looping.
///
/// 从 assets/audios/music 目录播放循环的背景音乐。
///
/// # Example
/// ```ignore
/// play_bgm(&audio, &asset_server, "mus_ruins.ogg");
/// ```
pub fn play_bgm(audio: &Audio, asset_server: &AssetServer, music_path: &str) {
    let music_handle = asset_server.load(format!("audios/music/{}", music_path));
    audio.play(music_handle).looped();
}
