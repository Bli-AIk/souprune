//! # sequencer/bgm.rs
//!
//! ## Module Overview
//!
//! SetBgm chapter processing system.
//! Plays, switches, or stops background music.
//!
//! SetBgm 章节处理系统。
//! 播放、切换或停止背景音乐。

use super::chapter_schema::Chapter;
use super::context::*;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

/// Resource tracking the current BGM instance spawned by the sequencer.
///
/// 跟踪由 sequencer 生成的当前 BGM 实例的资源。
#[derive(Resource, Default)]
pub struct SequencerBgm {
    pub handle: Option<Handle<AudioInstance>>,
    pub path: Option<String>,
}

/// System to process SetBgm chapters.
///
/// 处理 SetBgm 章节的系统。
pub fn process_set_bgm_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    audio: Option<Res<Audio>>,
    asset_server: Res<AssetServer>,
    mut bgm: ResMut<SequencerBgm>,
    audio_instances: Option<ResMut<Assets<AudioInstance>>>,
) {
    let (Some(audio), Some(mut audio_instances)) = (audio, audio_instances) else {
        // 音频不可用时，仅标记章节完成
        for (entity, active_chapter) in query.iter() {
            if matches!(&active_chapter.chapter, Chapter::SetBgm { .. }) {
                commands.entity(entity).insert(ChapterFinished);
            }
        }
        return;
    };
    for (entity, active_chapter) in query.iter() {
        if let Chapter::SetBgm { path, fade_in } = &active_chapter.chapter {
            // Stop current BGM if playing
            if let Some(handle) = &bgm.handle {
                if let Some(instance) = audio_instances.get_mut(handle) {
                    let tween = if let Some(duration) = fade_in {
                        AudioTween::linear(std::time::Duration::from_secs_f32(*duration))
                    } else {
                        AudioTween::default()
                    };
                    instance.stop(tween);
                }
                bgm.handle = None;
                bgm.path = None;
            }

            // Start new BGM if path is specified
            if let Some(bgm_path) = path {
                info!("[Sequencer] SetBgm: playing '{}'", bgm_path);
                let handle = crate::core::audio::play_bgm(&audio, &asset_server, bgm_path);
                bgm.handle = Some(handle);
                bgm.path = Some(bgm_path.clone());
            } else {
                info!("[Sequencer] SetBgm: stopping BGM");
            }

            commands.entity(entity).insert(ChapterFinished);
        }
    }
}
