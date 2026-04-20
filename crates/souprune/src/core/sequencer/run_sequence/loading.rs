//! Loads nested sequence assets for `RunSequence` chapters and inserts them into the live queue.
//!
//! 为 `RunSequence` 章节加载嵌套序列资源，并把它们插入当前执行队列。
//!
//! Implements the runtime half of `RunSequence`: resolve the target
//! path, start loading the referenced sequence, wait until the asset is ready,
//! inject any requested parameters, and then prepend the loaded chapters to the
//! active sequence context.
//!
//! 实现了 `RunSequence` 的运行时一侧：解析目标路径、开始加载被引用的
//! 序列、等待资源就绪、注入请求的参数，并最终把加载出的章节前插到当前序列上下文。

use bevy::prelude::*;
use bevy_fact_rule_event::LayeredFactDatabase;
use std::collections::HashMap;

use super::params::inject_sequence_params;
use crate::core::view::{ActiveView, ViewRoot};

use super::super::SequenceAsset;
use super::super::chapter_schema::{Chapter, FactValueMatch};
use super::super::context::{ActiveChapter, ChapterFinished, SequenceContext};

/// Component marker for RunSequence chapters being processed.
///
/// 正在处理的 RunSequence 章节的组件标记。
#[derive(Component)]
pub struct RunSequenceChapter {
    pub path: String,
    pub params: HashMap<String, FactValueMatch>,
    pub handle: Handle<SequenceAsset>,
}

/// System to process RunSequence chapters.
/// Resolves the sequence path and starts loading.
///
/// 处理 RunSequence 章节的系统。
/// 解析序列路径并开始加载。
pub fn process_run_sequence_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<ChapterFinished>, Without<RunSequenceChapter>)>,
    asset_server: Res<AssetServer>,
    fact_db: Res<LayeredFactDatabase>,
    view_roots: Query<&ViewRoot, With<ActiveView>>,
) {
    for (entity, active) in query.iter() {
        let Chapter::RunSequence {
            path,
            path_fact,
            params,
        } = &active.chapter
        else {
            continue;
        };

        let resolved_path = if let Some(path) = path {
            Some(path.clone())
        } else if let Some(fact_key) = path_fact {
            let local_path = view_roots.iter().next().and_then(|view_root| {
                view_root
                    .local_facts
                    .get_string(fact_key)
                    .map(ToString::to_string)
            });

            local_path.or_else(|| fact_db.get_string(fact_key).map(ToString::to_string))
        } else {
            None
        };

        if let Some(seq_path) = resolved_path {
            if seq_path.is_empty() {
                info!("RunSequence: Skipping empty path");
                commands.entity(entity).insert(ChapterFinished);
                continue;
            }
            let handle = asset_server.load::<SequenceAsset>(&seq_path);
            commands.entity(entity).insert(RunSequenceChapter {
                path: seq_path,
                params: params.clone(),
                handle,
            });
        } else {
            // path_fact may reference an optional parameter — use debug level
            debug!("RunSequence: Could not resolve path (path_fact may be unset)");
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}

/// System to check RunSequence asset loading completion.
/// When loaded, spawns the chapters from the sequence and injects parameters.
///
/// 检查 RunSequence 资产加载完成的系统。
/// 加载完成后，生成序列中的章节并注入参数。
pub fn complete_run_sequence_system(
    mut commands: Commands,
    mut query: Query<(Entity, &RunSequenceChapter, &ActiveChapter), Without<ChapterFinished>>,
    assets: Res<Assets<SequenceAsset>>,
    mut context: ResMut<SequenceContext>,
    mut view_root_query: Query<&mut ViewRoot, With<ActiveView>>,
    layered_db: Res<LayeredFactDatabase>,
    mut debug_info: ResMut<super::super::context::SequenceDebugInfo>,
) {
    for (entity, run_seq, _active) in query.iter_mut() {
        let Some(asset) = assets.get(&run_seq.handle) else {
            continue;
        };

        if !run_seq.params.is_empty() {
            if let Some(mut view_root) = view_root_query.iter_mut().next() {
                inject_sequence_params(&mut view_root, &run_seq.params, &layered_db);
            } else {
                warn!("RunSequence: params provided but no ViewRoot found to inject into");
            }
        }

        let mut new_chapters = asset.chapters.clone();
        new_chapters.append(&mut context.chapters);
        context.chapters = new_chapters;

        debug_info.previous_path = debug_info.current_path.take();
        debug_info.current_path = Some(run_seq.path.clone());

        info!(
            "RunSequence: Loaded {} chapters from {}",
            asset.chapters.len(),
            run_seq.path
        );
        commands.entity(entity).insert(ChapterFinished);
    }
}
