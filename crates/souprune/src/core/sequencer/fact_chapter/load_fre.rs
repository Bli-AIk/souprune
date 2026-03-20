use super::super::chapter_schema::{AggregateRule, Chapter};
use super::super::context::{ActiveChapter, ChapterFinished};
use bevy::prelude::*;
use bevy_fact_rule_event::{EnumRegistry, FactValue, LayeredFactDatabase};

use crate::core::game_action::GameFreAsset;

#[derive(Component)]
pub struct LoadFreState {
    pub handles: Vec<Handle<GameFreAsset>>,
    pub aggregate: std::collections::HashMap<String, AggregateRule>,
    pub processed: bool,
}

pub fn process_load_fre_chapter_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<ChapterFinished>, Without<LoadFreState>)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, active) in query.iter() {
        if let Chapter::LoadFre { files, aggregate } = &active.chapter {
            let handles: Vec<Handle<GameFreAsset>> = files
                .iter()
                .map(|path| {
                    info!("LoadFre Chapter: Loading FRE file '{}'", path);
                    asset_server.load::<GameFreAsset>(path.clone())
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

pub fn complete_load_fre_chapter_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LoadFreState), Without<ChapterFinished>>,
    mut layered_db: ResMut<LayeredFactDatabase>,
    fre_assets: Res<Assets<GameFreAsset>>,
    mut enum_registry: ResMut<EnumRegistry>,
) {
    for (entity, mut state) in query.iter_mut() {
        if state.processed {
            continue;
        }

        let all_loaded = state.handles.iter().all(|h| fre_assets.contains(h));
        if !all_loaded {
            continue;
        }

        let mut all_facts = std::collections::HashMap::new();

        for handle in &state.handles {
            let Some(fre_asset) = fre_assets.get(handle) else {
                continue;
            };
            enum_registry.register_from_asset(fre_asset);

            for (key, fact_value) in fre_asset.resolve_facts(&enum_registry) {
                all_facts.insert(key.clone(), fact_value.clone());
                layered_db.set(key.as_str(), fact_value);
            }
            info!(
                "LoadFre Chapter: Loaded {} facts from FRE file",
                fre_asset.get_facts().len()
            );
        }

        for (array_name, rule) in &state.aggregate {
            apply_aggregate_rule(array_name, rule, &all_facts, &mut layered_db);
        }

        state.processed = true;
        commands.entity(entity).insert(ChapterFinished);
        info!("LoadFre Chapter: Completed");
    }
}

fn apply_aggregate_rule(
    array_name: &str,
    rule: &AggregateRule,
    all_facts: &std::collections::HashMap<String, FactValue>,
    layered_db: &mut ResMut<LayeredFactDatabase>,
) {
    match rule {
        AggregateRule::Collect(pattern) => {
            let values = collect_matching_values(all_facts, pattern);
            if !values.is_empty() {
                apply_collected_values(array_name, &values, layered_db);
                info!(
                    "LoadFre Chapter: Aggregated {} values into '{}'",
                    values.len(),
                    array_name
                );
            }
        }
        AggregateRule::CollectKeys(pattern) => {
            let keys = collect_matching_keys(all_facts, pattern);
            if !keys.is_empty() {
                layered_db.set(array_name, keys.clone());
                info!(
                    "LoadFre Chapter: Collected {} keys into '{}'",
                    keys.len(),
                    array_name
                );
            }
        }
    }
}

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

fn collect_matching_values(
    facts: &std::collections::HashMap<String, FactValue>,
    pattern: &str,
) -> Vec<FactValue> {
    let mut values = Vec::new();
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
