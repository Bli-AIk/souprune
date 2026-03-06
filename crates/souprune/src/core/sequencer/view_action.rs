//! # sequencer/view_action.rs
//!
//! ## Module Overview
//!
//! View control systems for the battle sequencer.
//! All view spawning is unified through SpawnViewRequest messages.
//!
//! 战斗序列管理器的视图控制系统。
//! 所有视图生成通过 SpawnViewRequest 消息统一处理。

use super::chapter_schema::{Chapter, FactValueMatch};
use super::context::*;
use crate::core::view::components::ViewRoot;
use bevy::prelude::*;
use bevy_fact_rule_event::FactValue;

/// System to process view actions.
///
/// 处理视图动作的系统。
pub fn process_view_action_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    spawn_writer: Option<MessageWriter<crate::core::view::SpawnViewRequest>>,
) {
    let Some(mut spawn_writer) = spawn_writer else {
        return;
    };
    for (entity, active_chapter) in query.iter() {
        if let Chapter::SetUI(action) = &active_chapter.chapter {
            match action {
                super::chapter_schema::UIAction::LoadLayout(path) => {
                    spawn_writer.write(crate::core::view::SpawnViewRequest {
                        path: path.clone(),
                        mode_scope: Some("battle".to_string()),
                        bindings: None,
                    });
                }
                _ => {
                    warn!("View action {:?} not fully implemented yet", action);
                }
            }
            commands.entity(entity).insert(ChapterFinished);
        } else if let Chapter::SpawnView {
            view_layout,
            bindings,
        } = &active_chapter.chapter
        {
            info!("[Battle] Loading view layout for battle: {}", view_layout);

            spawn_writer.write(crate::core::view::SpawnViewRequest {
                path: view_layout.clone(),
                mode_scope: Some("battle".to_string()),
                bindings: if bindings.is_empty() {
                    None
                } else {
                    Some(bindings.clone())
                },
            });

            commands.entity(entity).insert(ChapterFinished);
        }
    }
}

/// System to process SetViewFact chapters.
///
/// 处理 SetViewFact 章节的系统。
pub fn process_set_view_fact_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    mut view_root_query: Query<&mut ViewRoot>,
) {
    for (chapter_entity, active_chapter) in query.iter() {
        if let Chapter::SetViewFact { key, value } = &active_chapter.chapter {
            // Find the active ViewRoot and set the fact
            if let Some(mut view_root) = view_root_query.iter_mut().next() {
                let fact_value = match value {
                    FactValueMatch::Bool(b) => Some(FactValue::Bool(*b)),
                    FactValueMatch::Int(i) => Some(FactValue::Int(*i)),
                    FactValueMatch::Float(f) => Some(FactValue::Float(*f)),
                    FactValueMatch::String(s) => Some(FactValue::String(s.clone())),
                    FactValueMatch::Expr(_expr) => {
                        warn!("[Battle] SetViewFact: Expr not supported yet for '{}'", key);
                        None
                    }
                };

                if let Some(fv) = fact_value {
                    view_root.local_facts.set(key.as_str(), fv.clone());
                    info!("[Battle] SetViewFact: Set '{}' = {:?}", key, fv);
                }
            } else {
                warn!("[Battle] SetViewFact: No ViewRoot found!");
            }

            commands.entity(chapter_entity).insert(ChapterFinished);
        }
    }
}
