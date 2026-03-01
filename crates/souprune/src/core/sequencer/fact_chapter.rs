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

use super::chapter_schema::{
    AggregateRule, Chapter, FactCondition, FactModificationDef, FactValueMatch,
};
use super::context::{ActiveChapter, ChapterFinished};
use super::flow::spawn_chapter;
use bevy::prelude::*;
use bevy_fact_rule_event::{FactEvent, FactReader, FactValue, FreAsset, LayeredFactDatabase};

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
/// For Expr values, it first checks View's local_facts, then falls back to LayeredFactDatabase.
///
/// 处理 ModifyFact 章节的系统。
/// 将指定的修改应用于 LayeredFactDatabase 并立即完成。
/// 对于 Expr 值，首先检查 View 的 local_facts，然后回退到 LayeredFactDatabase。
pub fn process_modify_fact_chapter_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), Without<ChapterFinished>>,
    mut layered_db: ResMut<LayeredFactDatabase>,
    view_root_query: Query<&crate::core::view::ViewRoot>,
) {
    for (entity, active) in query.iter() {
        if let Chapter::ModifyFact { modifications } = &active.chapter {
            for modification in modifications {
                match modification {
                    FactModificationDef::Set { key, value } => {
                        let fact_value: Option<FactValue> = match value {
                            FactValueMatch::Int(v) => Some(FactValue::Int(*v)),
                            FactValueMatch::Float(v) => Some(FactValue::Float(*v)),
                            FactValueMatch::Bool(v) => Some(FactValue::Bool(*v)),
                            FactValueMatch::String(v) => Some(FactValue::String(v.clone())),
                            FactValueMatch::Expr(expr) => {
                                // For simple $key references, read the fact directly
                                // First check View's local_facts, then LayeredFactDatabase
                                // This supports string facts unlike evaluate_expr_to_fact
                                if let Some(fact_key) = expr.strip_prefix('$') {
                                    // Try View's local_facts first
                                    let view_root = view_root_query.iter().next();
                                    info!(
                                        "ModifyFact Expr: looking for '{}', has ViewRoot={}",
                                        fact_key,
                                        view_root.is_some()
                                    );
                                    let from_view = view_root.and_then(|vr| {
                                        vr.local_facts.get_by_str(fact_key).cloned()
                                    });
                                    let from_db = layered_db.get_by_str(fact_key).cloned();
                                    info!(
                                        "ModifyFact Expr: '{}' -> view={:?}, db={:?}",
                                        fact_key, from_view, from_db
                                    );
                                    from_view.or(from_db)
                                } else {
                                    // For complex expressions, use numeric evaluation
                                    bevy_fact_rule_event::expr::evaluate_expr_to_fact(
                                        expr,
                                        &layered_db,
                                    )
                                }
                            }
                        };
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

            commands.entity(entity).insert(ChapterFinished);
        }
    }
}

// =============================================================================
// LoadFre Chapter Processing
// LoadFre 章节处理
// =============================================================================

/// Component to track LoadFre chapter state.
/// Holds handles to the FRE assets being loaded.
///
/// 跟踪 LoadFre 章节状态的组件。
/// 持有正在加载的 FRE 资产句柄。
#[derive(Component)]
pub struct LoadFreState {
    /// Handles to the FRE files being loaded
    pub handles: Vec<Handle<FreAsset>>,
    /// Aggregation rules to apply after loading
    pub aggregate: std::collections::HashMap<String, AggregateRule>,
    /// Whether all assets have been processed
    pub processed: bool,
}

/// System to initiate LoadFre chapter loading.
/// Starts loading FRE files and attaches LoadFreState component.
///
/// 启动 LoadFre 章节加载的系统。
/// 开始加载 FRE 文件并附加 LoadFreState 组件。
pub fn process_load_fre_chapter_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<ChapterFinished>, Without<LoadFreState>)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, active) in query.iter() {
        if let Chapter::LoadFre { files, aggregate } = &active.chapter {
            let handles: Vec<Handle<FreAsset>> = files
                .iter()
                .map(|path| {
                    info!("LoadFre Chapter: Loading FRE file '{}'", path);
                    asset_server.load::<FreAsset>(path.clone())
                })
                .collect();

            commands.entity(entity).insert(LoadFreState {
                handles,
                aggregate: aggregate.clone(),
                processed: false,
            });
        }
    }
}

/// System to complete LoadFre chapter after assets are loaded.
/// Processes loaded FRE files, applies aggregation, and finishes.
///
/// 资产加载后完成 LoadFre 章节的系统。
/// 处理已加载的 FRE 文件，应用聚合，然后完成。
pub fn complete_load_fre_chapter_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LoadFreState), Without<ChapterFinished>>,
    mut layered_db: ResMut<LayeredFactDatabase>,
    fre_assets: Res<Assets<FreAsset>>,
) {
    for (entity, mut state) in query.iter_mut() {
        if state.processed {
            continue;
        }

        // Check if all assets are loaded
        let all_loaded = state.handles.iter().all(|h| fre_assets.contains(h));
        if !all_loaded {
            continue;
        }

        // Collect all facts from loaded FRE files
        let mut all_facts: std::collections::HashMap<String, FactValue> =
            std::collections::HashMap::new();

        for handle in &state.handles {
            if let Some(fre_asset) = fre_assets.get(handle) {
                for (key, value_def) in fre_asset.get_facts() {
                    let fact_value: FactValue = value_def.clone().into();
                    all_facts.insert(key.clone(), fact_value.clone());
                    layered_db.set(key.as_str(), fact_value);
                }
                info!(
                    "LoadFre Chapter: Loaded {} facts from FRE file",
                    fre_asset.get_facts().len()
                );
            }
        }

        // Apply aggregation rules
        for (array_name, rule) in &state.aggregate {
            match rule {
                AggregateRule::Collect(pattern) => {
                    let values = collect_matching_values(&all_facts, pattern);
                    if !values.is_empty() {
                        apply_collected_values(array_name, &values, &mut layered_db);
                        info!(
                            "LoadFre Chapter: Aggregated {} values into '{}'",
                            values.len(),
                            array_name
                        );
                    }
                }
                AggregateRule::CollectKeys(pattern) => {
                    let keys = collect_matching_keys(&all_facts, pattern);
                    if !keys.is_empty() {
                        layered_db.set(array_name.as_str(), keys.clone());
                        info!(
                            "LoadFre Chapter: Collected {} keys into '{}'",
                            keys.len(),
                            array_name
                        );
                    }
                }
            }
        }

        state.processed = true;
        commands.entity(entity).insert(ChapterFinished);
        info!("LoadFre Chapter: Completed");
    }
}

/// Apply collected values to the database based on their type.
/// 根据值类型将收集的值应用到数据库。
fn apply_collected_values(
    array_name: &str,
    values: &[FactValue],
    layered_db: &mut ResMut<LayeredFactDatabase>,
) {
    let Some(first) = values.first() else {
        return;
    };

    match first {
        FactValue::Int(_) => {
            let int_values: Vec<i64> = values
                .iter()
                .filter_map(|v| {
                    if let FactValue::Int(i) = v {
                        Some(*i)
                    } else {
                        None
                    }
                })
                .collect();
            layered_db.set(array_name, int_values);
        }
        FactValue::String(_) => {
            let string_values: Vec<String> = values
                .iter()
                .filter_map(|v| {
                    if let FactValue::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .collect();
            layered_db.set(array_name, string_values);
        }
        _ => {
            warn!(
                "LoadFre Chapter: Unsupported value type for aggregation '{}' (only Int and String supported)",
                array_name
            );
        }
    }
}

/// Collect values from facts matching a glob pattern.
/// Pattern uses `*` as wildcard. Example: "*.hp" matches "dummy.hp", "sans.hp".
///
/// 收集匹配 glob 模式的 facts 值。
/// 模式使用 `*` 作为通配符。示例："*.hp" 匹配 "dummy.hp"、"sans.hp"。
fn collect_matching_values(
    facts: &std::collections::HashMap<String, FactValue>,
    pattern: &str,
) -> Vec<FactValue> {
    let mut values = Vec::new();

    // Simple glob matching: convert "*.hp" to regex "^[^.]+\.hp$"
    let regex_pattern = pattern.replace(".", r"\.").replace("*", "[^.]+");
    let regex = regex::Regex::new(&format!("^{}$", regex_pattern)).ok();

    if let Some(re) = regex {
        for (key, value) in facts {
            if re.is_match(key) {
                values.push(value.clone());
            }
        }
    }

    values
}

/// Collect fact keys matching a glob pattern.
///
/// 收集匹配 glob 模式的 fact 键。
fn collect_matching_keys(
    facts: &std::collections::HashMap<String, FactValue>,
    pattern: &str,
) -> Vec<String> {
    let mut keys = Vec::new();

    let regex_pattern = pattern.replace(".", r"\.").replace("*", "[^.]+");
    let regex = regex::Regex::new(&format!("^{}$", regex_pattern)).ok();

    if let Some(re) = regex {
        for key in facts.keys() {
            if re.is_match(key) {
                keys.push(key.clone());
            }
        }
    }

    keys
}
