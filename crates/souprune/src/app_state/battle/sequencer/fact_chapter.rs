//! # sequencer/fact_chapter.rs
//!
//! ## Module Overview
//!
//! Processing systems for FRE-based conditional chapters.
//!
//! 基于 FRE 的条件章节处理系统。
//!
//! This module handles:
//! - Conditional chapters (if-then-else based on facts)
//! - FactSwitch chapters (switch-case based on fact values)
//! - EmitFactEvent chapters (emit FRE events from sequencer)
//! - ModifyFact chapters (modify facts from sequencer)
//!
//! 本模块处理：
//! - 条件章节（基于 facts 的 if-then-else）
//! - FactSwitch 章节（基于 fact 值的 switch-case）
//! - EmitFactEvent 章节（从 sequencer 发出 FRE 事件）
//! - ModifyFact 章节（从 sequencer 修改 facts）

use super::context::{ActiveChapter, ChapterFinished};
use super::flow::spawn_chapter;
use crate::app_state::battle::chapter_schema::{
    Chapter, FactCondition, FactModificationDef, FactValueMatch,
};
use bevy::prelude::*;
use bevy_fact_rule_event::{FactEvent, FactReader, FactValue, LayeredFactDatabase};

/// Evaluate a FactCondition against the LayeredFactDatabase.
///
/// 根据 LayeredFactDatabase 评估 FactCondition。
fn evaluate_condition(condition: &FactCondition, db: &impl FactReader) -> bool {
    match condition {
        FactCondition::Equals { key, value } => {
            let fact_value = db.get_by_str(key);
            match (fact_value, value) {
                (Some(FactValue::Int(v)), FactValueMatch::Int(expected)) => *v == *expected,
                (Some(FactValue::Float(v)), FactValueMatch::Float(expected)) => {
                    (*v - *expected).abs() < f64::EPSILON
                }
                (Some(FactValue::Bool(v)), FactValueMatch::Bool(expected)) => *v == *expected,
                (Some(FactValue::String(v)), FactValueMatch::String(expected)) => v == expected,
                _ => false,
            }
        }
        FactCondition::GreaterThan { key, value } => db.get_int(key).is_some_and(|v| v > *value),
        FactCondition::LessThan { key, value } => db.get_int(key).is_some_and(|v| v < *value),
        FactCondition::GreaterOrEqual { key, value } => {
            db.get_int(key).is_some_and(|v| v >= *value)
        }
        FactCondition::LessOrEqual { key, value } => db.get_int(key).is_some_and(|v| v <= *value),
        FactCondition::Exists(key) => db.contains(key),
        FactCondition::NotExists(key) => !db.contains(key),
        FactCondition::IsTrue(key) => db.get_bool(key) == Some(true),
        FactCondition::IsFalse(key) => db.get_bool(key) == Some(false),
        FactCondition::And(conditions) => conditions.iter().all(|c| evaluate_condition(c, db)),
        FactCondition::Or(conditions) => conditions.iter().any(|c| evaluate_condition(c, db)),
        FactCondition::Not(condition) => !evaluate_condition(condition, db),
        FactCondition::Always => true,
    }
}

/// Match a fact value against a FactValueMatch.
///
/// 将 fact 值与 FactValueMatch 匹配。
fn matches_value(fact_value: Option<&FactValue>, expected: &FactValueMatch) -> bool {
    match (fact_value, expected) {
        (Some(FactValue::Int(v)), FactValueMatch::Int(expected)) => *v == *expected,
        (Some(FactValue::Float(v)), FactValueMatch::Float(expected)) => {
            (*v - *expected).abs() < f64::EPSILON
        }
        (Some(FactValue::Bool(v)), FactValueMatch::Bool(expected)) => *v == *expected,
        (Some(FactValue::String(v)), FactValueMatch::String(expected)) => v == expected,
        _ => false,
    }
}

/// System to process Conditional chapters.
/// Evaluates the condition and spawns the appropriate branch.
///
/// 处理 Conditional 章节的系统。
/// 评估条件并生成适当的分支。
pub fn process_conditional_chapter_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), Without<ChapterFinished>>,
    layered_db: Res<LayeredFactDatabase>,
) {
    for (entity, active) in query.iter() {
        if let Chapter::Conditional {
            condition,
            then_branch,
            else_branch,
        } = &active.chapter
        {
            let result = evaluate_condition(condition, &*layered_db);

            info!("Conditional Chapter: condition evaluated to {}", result);

            if result {
                // Spawn then branch as child
                spawn_chapter(&mut commands, (**then_branch).clone(), Some(entity));
            } else if let Some(else_chapter) = else_branch {
                // Spawn else branch as child
                spawn_chapter(&mut commands, (**else_chapter).clone(), Some(entity));
            }

            // Mark parent as having one pending child (or finished if no else branch and condition false)
            if result || else_branch.is_some() {
                commands
                    .entity(entity)
                    .insert(super::context::ParallelTracker { pending_count: 1 });
            } else {
                // No else branch and condition is false - just finish
                commands.entity(entity).insert(ChapterFinished);
            }
        }
    }
}

/// System to process FactSwitch chapters.
/// Matches the fact value and spawns the appropriate case branch.
///
/// 处理 FactSwitch 章节的系统。
/// 匹配 fact 值并生成适当的 case 分支。
pub fn process_fact_switch_chapter_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), Without<ChapterFinished>>,
    layered_db: Res<LayeredFactDatabase>,
) {
    for (entity, active) in query.iter() {
        if let Chapter::FactSwitch {
            fact_key,
            cases,
            default,
        } = &active.chapter
        {
            let fact_value = layered_db.get_by_str(fact_key);

            // Find matching case
            let mut matched_chapter = None;
            for (case_value, chapter) in cases {
                if matches_value(fact_value, case_value) {
                    matched_chapter = Some(chapter.clone());
                    info!("FactSwitch Chapter: matched case for key '{}'", fact_key);
                    break;
                }
            }

            // Use default if no match
            let chapter_to_spawn = matched_chapter.or_else(|| {
                default.as_ref().map(|d| {
                    info!(
                        "FactSwitch Chapter: no match for key '{}', using default",
                        fact_key
                    );
                    (**d).clone()
                })
            });

            if let Some(chapter) = chapter_to_spawn {
                spawn_chapter(&mut commands, chapter, Some(entity));
                commands
                    .entity(entity)
                    .insert(super::context::ParallelTracker { pending_count: 1 });
            } else {
                // No match and no default - just finish
                info!(
                    "FactSwitch Chapter: no match and no default for key '{}'",
                    fact_key
                );
                commands.entity(entity).insert(ChapterFinished);
            }
        }
    }
}

/// System to process EmitFactEvent chapters.
/// Emits the specified FRE event and finishes immediately.
///
/// 处理 EmitFactEvent 章节的系统。
/// 发出指定的 FRE 事件并立即完成。
pub fn process_emit_fact_event_chapter_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), Without<ChapterFinished>>,
    mut event_writer: MessageWriter<FactEvent>,
) {
    for (entity, active) in query.iter() {
        if let Chapter::EmitFactEvent { event_id, data } = &active.chapter {
            let mut event = FactEvent::new(event_id.clone());

            for (key, value) in data {
                event = event.with_data(key, value);
            }

            event_writer.write(event);

            info!("EmitFactEvent Chapter: emitted event '{}'", event_id);

            commands.entity(entity).insert(ChapterFinished);
        }
    }
}

/// System to process ModifyFact chapters.
/// Applies the specified modifications to the LayeredFactDatabase and finishes immediately.
///
/// 处理 ModifyFact 章节的系统。
/// 将指定的修改应用于 LayeredFactDatabase 并立即完成。
pub fn process_modify_fact_chapter_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), Without<ChapterFinished>>,
    mut layered_db: ResMut<LayeredFactDatabase>,
) {
    for (entity, active) in query.iter() {
        if let Chapter::ModifyFact { modifications } = &active.chapter {
            for modification in modifications {
                match modification {
                    FactModificationDef::Set { key, value } => {
                        let fact_value: FactValue = match value {
                            FactValueMatch::Int(v) => FactValue::Int(*v),
                            FactValueMatch::Float(v) => FactValue::Float(*v),
                            FactValueMatch::Bool(v) => FactValue::Bool(*v),
                            FactValueMatch::String(v) => FactValue::String(v.clone()),
                        };
                        layered_db.set(key.as_str(), fact_value);
                        info!("ModifyFact Chapter: Set '{}' to {:?}", key, value);
                    }
                    FactModificationDef::Increment { key, amount } => {
                        layered_db.increment(key, *amount);
                        info!("ModifyFact Chapter: Increment '{}' by {}", key, amount);
                    }
                    FactModificationDef::Remove(key) => {
                        layered_db.remove(key);
                        info!("ModifyFact Chapter: Remove '{}'", key);
                    }
                    FactModificationDef::Toggle(key) => {
                        let current = layered_db.get_bool(key).unwrap_or(false);
                        layered_db.set(key.as_str(), !current);
                        info!("ModifyFact Chapter: Toggle '{}' (now {})", key, !current);
                    }
                }
            }

            commands.entity(entity).insert(ChapterFinished);
        }
    }
}
