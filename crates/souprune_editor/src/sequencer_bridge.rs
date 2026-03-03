//! # Sequencer Bridge
//!
//! 编辑器与 Sequencer 之间的桥接层。
//! 负责在 Play/Edit 切换时加载/清理序列上下文。

use bevy::prelude::*;

use crate::panels::playback::PlaybackState;
use crate::panels::sequence_timeline::EditorSequenceState;
use souprune::core::sequencer::{CurrentSequenceFlow, SequenceContext, SequenceExecutionState};

/// 进入 Play 模式时，将编辑器中的序列加载到 SequenceContext。
/// 若无编辑器序列，则从 SoupruneConfig 加载默认序列。
/// 支持 `PlaybackState::start_from` 从指定章节开始播放。
pub fn on_enter_play(
    mut commands: Commands,
    editor_seq: Res<EditorSequenceState>,
    mut context: ResMut<SequenceContext>,
    mut playback: ResMut<PlaybackState>,
    config: Option<Res<souprune::config::SoupruneConfig>>,
    asset_server: Res<AssetServer>,
) {
    if let Some(seq) = &editor_seq.current {
        let start_idx = playback.start_from.take().unwrap_or(0);
        let chapters: Vec<_> = seq.chapters.iter().skip(start_idx).cloned().collect();
        info!(
            "[编辑器] 进入播放 — 从章节 {start_idx} 开始，加载 {} 个章节",
            chapters.len()
        );
        context.chapters = chapters;
        context.state = SequenceExecutionState::Idle;
    } else if let Some(cfg) = config {
        // 从 SoupruneConfig 加载默认序列
        if let Some(path) = cfg.game.initial_sequence_path.clone() {
            info!("[编辑器] 进入播放 — 加载默认序列: {}", path);
            let handle = asset_server.load::<souprune::core::sequencer::SequenceAsset>(path);
            commands.insert_resource(CurrentSequenceFlow(handle));
        } else {
            warn!("[编辑器] 进入播放但无默认序列路径");
        }
    } else {
        warn!("[编辑器] 进入播放但无可用序列");
    }
}

/// 退出 Play 模式时，清理 SequenceContext 和活跃章节实体。
pub fn on_exit_play(
    mut commands: Commands,
    mut context: ResMut<SequenceContext>,
    active_chapters: Query<Entity, With<souprune::core::sequencer::ActiveChapter>>,
) {
    context.chapters.clear();
    context.state = SequenceExecutionState::Idle;

    for entity in &active_chapters {
        commands.entity(entity).despawn();
    }

    info!("[编辑器] 退出播放 — 已清理序列状态");
}
