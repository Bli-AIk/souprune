//! Executes fact-driven branching chapters such as `Conditional` and `FactSwitch`.
//!
//! 执行基于 facts 决定分支的章节，例如 `Conditional` 与 `FactSwitch`。
//!
//! This file is the branching evaluator for the sequencer's fact-oriented
//! chapter set. It reads the current layered fact database, decides which branch
//! should run, and then spawns the chosen child chapter back into the generic
//! chapter lifecycle managed by the flow systems.
//!
//! 这个文件是 sequencer 中 fact 分支章节的求值器。它读取当前的 layered fact
//! 数据库，决定应当执行哪个分支，然后把选中的子章节重新交回给 flow 生命周期
//! 系统去继续推进。

use super::super::chapter_schema::{Chapter, FactCondition, FactValueMatch};
use super::super::context::{ActiveChapter, ChapterFinished, ParallelTracker};
use super::super::flow::spawn_chapter;
use bevy::prelude::*;
use bevy_fact_rule_event::{FactReader, FactValue, LayeredFactDatabase};

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
                spawn_chapter(&mut commands, (**then_branch).clone(), Some(entity));
            } else if let Some(else_chapter) = else_branch {
                spawn_chapter(&mut commands, (**else_chapter).clone(), Some(entity));
            }

            if result || else_branch.is_some() {
                commands
                    .entity(entity)
                    .insert(ParallelTracker { pending_count: 1 });
            } else {
                commands.entity(entity).insert(ChapterFinished);
            }
        }
    }
}

pub fn process_fact_switch_chapter_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), Without<ChapterFinished>>,
    layered_db: Res<LayeredFactDatabase>,
) {
    for (entity, active) in query.iter() {
        let Chapter::FactSwitch {
            fact_key,
            cases,
            default,
        } = &active.chapter
        else {
            continue;
        };

        let fact_value = layered_db.get_by_str(fact_key);

        let mut matched_chapter = None;
        for (case_value, chapter) in cases {
            if matches_value(fact_value, case_value) {
                matched_chapter = Some(chapter.clone());
                info!("FactSwitch Chapter: matched case for key '{}'", fact_key);
                break;
            }
        }

        let chapter_to_spawn = matched_chapter.or_else(|| {
            let d = default.as_ref()?;
            info!(
                "FactSwitch Chapter: no match for key '{}', using default",
                fact_key
            );
            Some((**d).clone())
        });

        if let Some(chapter) = chapter_to_spawn {
            spawn_chapter(&mut commands, chapter, Some(entity));
            commands
                .entity(entity)
                .insert(ParallelTracker { pending_count: 1 });
        } else {
            info!(
                "FactSwitch Chapter: no match and no default for key '{}'",
                fact_key
            );
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}
