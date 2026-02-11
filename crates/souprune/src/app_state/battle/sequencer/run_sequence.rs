//! # sequencer/run_sequence.rs
//!
//! ## Module Overview
//!
//! Handles RunSequence and ExecuteActBehavior chapter types.
//! Enables loading and executing external sequence files with parameter passing.
//!
//! 处理 RunSequence 和 ExecuteActBehavior 章节类型。
//! 支持加载和执行外部序列文件，并传递参数。

use super::context::*;
use super::flow::spawn_chapter;
use crate::app_state::battle::SequenceAsset;
use crate::app_state::battle::chapter_schema::{Chapter, FactValueMatch};
use crate::core::view::ViewRoot;
use bevy::prelude::*;
use bevy_fact_rule_event::{FactDatabase, FactReader, FactValue};
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

/// Component marker for ExecuteActBehavior chapters being processed.
///
/// 正在处理的 ExecuteActBehavior 章节的组件标记。
#[derive(Component)]
pub struct ExecuteActBehaviorChapter {
    pub behaviors_fact: String,
    pub index_fact: String,
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
    fact_db: Res<FactDatabase>,
    view_roots: Query<&ViewRoot>,
) {
    for (entity, active) in query.iter() {
        if let Chapter::RunSequence {
            path,
            path_fact,
            params,
        } = &active.chapter
        {
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
    mut context: ResMut<BattleContext>,
    mut view_root_query: Query<&mut ViewRoot>,
) {
    for (entity, run_seq, _active) in query.iter_mut() {
        if let Some(asset) = assets.get(&run_seq.handle) {
            // Inject parameters into ViewRoot's local_facts
            // Parameters are prefixed with "_param_" to avoid conflicts
            if !run_seq.params.is_empty() {
                if let Some(mut view_root) = view_root_query.iter_mut().next() {
                    for (key, value) in &run_seq.params {
                        let prefixed_key = format!("{}{}", PARAM_PREFIX, key);
                        let fact_value = match value {
                            FactValueMatch::Bool(b) => FactValue::Bool(*b),
                            FactValueMatch::Int(i) => FactValue::Int(*i),
                            FactValueMatch::Float(f) => FactValue::Float(*f),
                            FactValueMatch::String(s) => FactValue::String(s.clone()),
                        };
                        view_root.local_facts.set(prefixed_key, fact_value);
                        info!("RunSequence: Injected param '{}' = {:?}", key, value);
                    }
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
}

/// System to process ExecuteActBehavior chapters.
/// Parses the behavior string and executes the appropriate action.
///
/// 处理 ExecuteActBehavior 章节的系统。
/// 解析行为字符串并执行相应操作。
pub fn process_execute_act_behavior_system(
    mut commands: Commands,
    query: Query<
        (Entity, &ActiveChapter),
        (Without<ChapterFinished>, Without<ExecuteActBehaviorChapter>),
    >,
    fact_db: Res<FactDatabase>,
    view_roots: Query<&ViewRoot>,
    mut context: ResMut<BattleContext>,
) {
    for (entity, active) in query.iter() {
        if let Chapter::ExecuteActBehavior {
            behaviors_fact,
            index_fact,
        } = &active.chapter
        {
            // Get the index
            let index: usize = view_roots
                .iter()
                .next()
                .and_then(|vr| vr.local_facts.get_int(index_fact))
                .or_else(|| fact_db.get_int(index_fact))
                .unwrap_or(0) as usize;

            // Get behavior from local facts first
            let mut behavior_opt: Option<String> = None;
            if let Some(vr) = view_roots.iter().next() {
                if let Some(list) = FactReader::get_string_list(&vr.local_facts, behaviors_fact) {
                    if index < list.len() {
                        behavior_opt = Some(list[index].clone());
                    }
                }
            }

            // Fallback to global facts
            if behavior_opt.is_none() {
                if let Some(list) = FactReader::get_string_list(&*fact_db, behaviors_fact) {
                    if index < list.len() {
                        behavior_opt = Some(list[index].clone());
                    }
                }
            }

            if let Some(behavior) = behavior_opt {
                let generated_chapters = parse_behavior_string(&behavior);

                // Insert generated chapters at the front of the queue
                let mut new_queue = generated_chapters;
                new_queue.append(&mut context.chapters);
                context.chapters = new_queue;

                info!(
                    "ExecuteActBehavior: Executing behavior '{}' at index {}",
                    behavior, index
                );
            } else {
                warn!(
                    "ExecuteActBehavior: Could not find behaviors at index {} in fact '{}'",
                    index, behaviors_fact
                );
            }

            commands.entity(entity).insert(ChapterFinished);
        }
    }
}

/// Parse a behavior string and generate the corresponding chapters.
///
/// Behavior formats:
/// - `"narration:NodeName"` → ShowNarration + AwaitConfirm + EndViewInteraction
/// - `"sequence:path"` → RunSequence(path)
/// - `"end"` → EndViewInteraction
///
/// 解析行为字符串并生成对应的章节。
fn parse_behavior_string(behavior: &str) -> Vec<Chapter> {
    if behavior.starts_with("narration:") {
        let node_name = behavior.strip_prefix("narration:").unwrap();

        // TODO(mortar): Currently only reads the first text field from the Mortar node.
        // Future implementation needs:
        // - NodeManager: manage Mortar node loading and querying
        // - DialogueScheduler: multi-step dialogue scheduling and progression
        // - EventDispatcher: handle Mortar events (play_sound, set_flag, etc.)
        // - TextRenderer: integrate with typewriter effect

        vec![
            // Set narration key for display
            Chapter::SetViewFact {
                key: "narration_key".to_string(),
                value: FactValueMatch::String(node_name.to_string()),
            },
            Chapter::SetViewFact {
                key: "narration_visible".to_string(),
                value: FactValueMatch::Bool(true),
            },
            // Wait for player to press confirm
            Chapter::AwaitFact {
                condition: "$confirm_pressed == true".to_string(),
                local: true,
            },
            Chapter::SetViewFact {
                key: "confirm_pressed".to_string(),
                value: FactValueMatch::Bool(false),
            },
            // End interaction
            Chapter::SetViewFact {
                key: "interactable".to_string(),
                value: FactValueMatch::Bool(false),
            },
            Chapter::SetViewFact {
                key: "narration_visible".to_string(),
                value: FactValueMatch::Bool(false),
            },
        ]
    } else if behavior.starts_with("sequence:") {
        let path = behavior.strip_prefix("sequence:").unwrap();
        vec![Chapter::RunSequence {
            path: Some(path.to_string()),
            path_fact: None,
            params: HashMap::new(),
        }]
    } else if behavior == "end" {
        // Just end the interaction
        vec![Chapter::SetViewFact {
            key: "interactable".to_string(),
            value: FactValueMatch::Bool(false),
        }]
    } else {
        warn!("Unknown behavior format: {}", behavior);
        vec![]
    }
}
