//! # Sequencer Bridge
//!
//! # Sequencer 桥接层
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Bridge layer between the editor and Sequencer.
//! Responsible for loading/cleaning sequence context when switching between Play/Edit modes.
//!
//! 编辑器与 Sequencer 之间的桥接层。
//! 负责在 Play/Edit 模式切换时加载/清理序列上下文。

use bevy::prelude::*;
use souprune::editor_api::{app, game_action, sequencer};

use crate::panels::playback::PlaybackState;
use crate::panels::sequence_timeline::EditorSequenceState;

/// 进入 Play 模式时，将编辑器中的序列加载到 SequenceContext。
/// 若无编辑器序列，则从 SoupruneConfig 加载默认序列。
/// 支持 `PlaybackState::start_from` 从指定章节开始播放。
pub fn on_enter_play(
    mut commands: Commands,
    editor_seq: Res<EditorSequenceState>,
    mut context: ResMut<sequencer::SequenceContext>,
    mut playback: ResMut<PlaybackState>,
    mut sequence_mode: ResMut<app::SequenceMode>,
    config: Option<Res<souprune::config::SoupruneConfig>>,
    asset_server: Res<AssetServer>,
) {
    if let Some(seq) = &editor_seq.current {
        let start_idx = playback.start_from.take().unwrap_or(0);
        let runtime_asset = match sequencer::runtime_asset_from_schema(&seq.to_asset()) {
            Ok(asset) => asset,
            Err(err) => {
                error!("[editor] entering play failed: {err}");
                context.chapters.clear();
                context.state = sequencer::SequenceExecutionState::Idle;
                return;
            }
        };
        let chapters: Vec<_> = runtime_asset.chapters.into_iter().skip(start_idx).collect();
        info!(
            "[editor] entering play — from chapter {start_idx}, loading {} chapters",
            chapters.len()
        );
        context.chapters = chapters;
        context.state = sequencer::SequenceExecutionState::Idle;
        // 同步 SequenceMode（与 sync_battle_flow_system 一致）
        if let Some(mode) = &seq.mode {
            sequence_mode.0 = Some(mode.clone());
        }
        // 加载序列级 FRE 规则
        if let Some(rules_path) = &seq.rules_file {
            let rules_handle = asset_server.load::<game_action::GameFreAsset>(rules_path.clone());
            commands.insert_resource(sequencer::SequenceRulesHandle {
                handle: Some(rules_handle),
                registered: false,
            });
        }
    } else if let Some(cfg) = config {
        // 从 SoupruneConfig 加载默认序列
        if let Some(path) = cfg.game.initial_sequence_path.clone() {
            info!("[编辑器] 进入播放 — 加载默认序列: {}", path);
            let handle = asset_server.load::<sequencer::SequenceAsset>(path);
            commands.insert_resource(sequencer::CurrentSequenceFlow(handle));
        } else {
            warn!("[编辑器] 进入播放但无默认序列路径");
        }
    } else {
        warn!("[编辑器] 进入播放但无可用序列");
    }
}

/// 退出 Play 模式时，由 souprune::reset_game_state 执行全局清理。
/// 此函数仅处理编辑器侧状态（PlaybackState 等）。
pub fn on_exit_play(mut playback: ResMut<PlaybackState>) {
    playback.start_from = None;
    info!("[编辑器] 退出播放");
}
