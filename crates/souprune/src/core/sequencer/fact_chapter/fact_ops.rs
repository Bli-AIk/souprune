//! Executes chapters that emit fact events or directly mutate the layered fact database.
//!
//! 执行那些会发出 fact 事件或直接修改分层 fact 数据库的章节。
//!
//! This file is the imperative fact-operations layer of the sequencer. It turns
//! schema-level `EmitFactEvent` and `ModifyFact` chapters into concrete runtime
//! side effects, including expression resolution that can read either view-local
//! facts or the shared layered database.
//!
//! 这个文件是 sequencer 里偏命令式的 fact 操作层。它把 schema 里的
//! `EmitFactEvent` 和 `ModifyFact` 章节落地成实际运行时副作用，其中表达式求值
//! 既可以读取 view 局部 facts，也可以读取共享的 layered fact 数据库。

use super::super::chapter_schema::{Chapter, FactModificationDef, FactValueMatch};
use super::super::context::{ActiveChapter, ChapterFinished};
use bevy::prelude::*;
use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase};

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

pub fn process_modify_fact_chapter_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), Without<ChapterFinished>>,
    mut layered_db: ResMut<LayeredFactDatabase>,
    view_root_query: Query<&crate::core::view::ViewRoot>,
) {
    for (entity, active) in query.iter() {
        let Chapter::ModifyFact { modifications } = &active.chapter else {
            continue;
        };

        for modification in modifications {
            apply_fact_modification(modification, &mut layered_db, &view_root_query);
        }

        commands.entity(entity).insert(ChapterFinished);
    }
}

fn apply_fact_modification(
    modification: &FactModificationDef,
    layered_db: &mut ResMut<LayeredFactDatabase>,
    view_root_query: &Query<&crate::core::view::ViewRoot>,
) {
    match modification {
        FactModificationDef::Set { key, value } => {
            let fact_value = resolve_set_value(value, layered_db, view_root_query);
            if let Some(fv) = fact_value {
                layered_db.set(key.as_str(), fv.clone());
                info!("ModifyFact Chapter: Set '{}' to {:?}", key, fv);
            } else {
                warn!(
                    "ModifyFact Chapter: Failed to evaluate expression for '{}'",
                    key
                );
            }
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

fn resolve_set_value(
    value: &FactValueMatch,
    layered_db: &LayeredFactDatabase,
    view_root_query: &Query<&crate::core::view::ViewRoot>,
) -> Option<FactValue> {
    match value {
        FactValueMatch::Int(v) => Some(FactValue::Int(*v)),
        FactValueMatch::Float(v) => Some(FactValue::Float(*v)),
        FactValueMatch::Bool(v) => Some(FactValue::Bool(*v)),
        FactValueMatch::String(v) => Some(FactValue::String(v.clone())),
        FactValueMatch::Expr(expr) => resolve_expr_value(expr, layered_db, view_root_query),
    }
}

fn resolve_expr_value(
    expr: &str,
    layered_db: &LayeredFactDatabase,
    view_root_query: &Query<&crate::core::view::ViewRoot>,
) -> Option<FactValue> {
    let Some(fact_key) = expr.strip_prefix('$') else {
        return bevy_fact_rule_event::expr::evaluate_expr_to_fact(expr, layered_db);
    };

    let view_root = view_root_query.iter().next();
    info!(
        "ModifyFact Expr: looking for '{}', has ViewRoot={}",
        fact_key,
        view_root.is_some()
    );
    let from_view = view_root.and_then(|vr| vr.local_facts.get_by_str(fact_key).cloned());
    let from_db = layered_db.get_by_str(fact_key).cloned();
    info!(
        "ModifyFact Expr: '{}' -> view={:?}, db={:?}",
        fact_key, from_view, from_db
    );
    from_view.or(from_db)
}
