//! # sequencer/run_sequence.rs
//!
//! ## Module Overview
//!
//! Handles RunSequence chapter types.
//! Enables loading and executing external sequence files with parameter passing.
//!
//! 处理 RunSequence 章节类型。
//! 支持加载和执行外部序列文件，并传递参数。

use super::SequenceAsset;
use super::chapter_schema::{Chapter, FactValueMatch};
use super::context::*;
use crate::core::view::ViewRoot;
use bevy::prelude::*;
use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};
use std::collections::HashMap;

/// Prefix for injected sequence parameters in local_facts.
/// Parameters are stored as `_param_{name}` to avoid conflicts.
///
/// 注入到 local_facts 的序列参数前缀。
/// 参数存储为 `_param_{name}` 以避免冲突。
const PARAM_PREFIX: &str = "_param_";

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
    view_roots: Query<&ViewRoot>,
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

        // Resolve the path
        let resolved_path = if let Some(p) = path {
            Some(p.clone())
        } else if let Some(fact_key) = path_fact {
            // Try local facts first, then global
            let local_path = view_roots.iter().next().and_then(|vr| {
                vr.local_facts
                    .get_string(fact_key)
                    .map(|s: &str| s.to_string())
            });

            local_path.or_else(|| fact_db.get_string(fact_key).map(|s: &str| s.to_string()))
        } else {
            None
        };

        if let Some(seq_path) = resolved_path {
            // Skip empty paths (placeholder for unimplemented actions)
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
            warn!("RunSequence: Could not resolve path");
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
    mut view_root_query: Query<&mut ViewRoot>,
    layered_db: Res<LayeredFactDatabase>,
) {
    for (entity, run_seq, _active) in query.iter_mut() {
        let Some(asset) = assets.get(&run_seq.handle) else {
            continue;
        };

        // Inject parameters into ViewRoot's local_facts
        // Parameters are prefixed with "_param_" to avoid conflicts
        if !run_seq.params.is_empty() {
            if let Some(mut view_root) = view_root_query.iter_mut().next() {
                inject_sequence_params(&mut view_root, &run_seq.params, &layered_db);
            } else {
                warn!("RunSequence: params provided but no ViewRoot found to inject into");
            }
        }

        // Insert the chapters from the loaded sequence at the front of the queue
        // This ensures they execute before any remaining chapters
        let mut new_chapters = asset.chapters.clone();
        new_chapters.append(&mut context.chapters);
        context.chapters = new_chapters;

        info!(
            "RunSequence: Loaded {} chapters from {}",
            asset.chapters.len(),
            run_seq.path
        );
        commands.entity(entity).insert(ChapterFinished);
    }
}

/// Inject sequence parameters into a ViewRoot's local_facts.
fn inject_sequence_params(
    view_root: &mut ViewRoot,
    params: &HashMap<String, FactValueMatch>,
    layered_db: &LayeredFactDatabase,
) {
    for (key, value) in params {
        let prefixed_key = format!("{}{}", PARAM_PREFIX, key);
        let Some(fv) = resolve_fact_value(value, layered_db) else {
            warn!("RunSequence: Failed to evaluate param '{}'", key);
            continue;
        };
        view_root.local_facts.set(prefixed_key, fv.clone());
        info!("RunSequence: Injected param '{}' = {:?}", key, fv);
    }
}

/// Resolve a FactValueMatch to a FactValue.
fn resolve_fact_value(
    value: &FactValueMatch,
    layered_db: &LayeredFactDatabase,
) -> Option<FactValue> {
    match value {
        FactValueMatch::Bool(b) => Some(FactValue::Bool(*b)),
        FactValueMatch::Int(i) => Some(FactValue::Int(*i)),
        FactValueMatch::Float(f) => Some(FactValue::Float(*f)),
        FactValueMatch::String(s) => Some(FactValue::String(s.clone())),
        FactValueMatch::Expr(expr) => {
            if let Some(fact_key) = expr.strip_prefix('$') {
                layered_db.get_by_str(fact_key).cloned()
            } else {
                bevy_fact_rule_event::expr::evaluate_expr_to_fact(expr, layered_db)
            }
        }
    }
}
