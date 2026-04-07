//! # enemy.rs
//!
//! # enemy.rs 文件
//!
//! The `enemy` module defines typed data structures for enemy definitions,
//! loads them from `.enemy.ron` files, and maintains an `EnemyRegistry` resource.
//! Enemy data is projected into the FRE fact database at battle start for
//! View dynamic resolution (e.g. `$$enemy_id.action_labels`).
//!
//! `enemy` 模块定义了敌人的类型化数据结构，从 `.enemy.ron` 文件加载，
//! 并维护 `EnemyRegistry` 资源。战斗开始时，敌人数据被投影到 FRE fact 数据库中，
//! 供 View 动态解析（如 `$$enemy_id.action_labels`）。

use bevy::asset::LoadedFolder;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
pub use souprune_schema::enemy::ActionOption;
use souprune_schema::enemy::EnemyDef as SchemaEnemyDef;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemyRegistry>()
            .init_asset::<EnemyDef>()
            .register_asset_loader(crate::core::ron_loader::RonAssetLoader::<EnemyDef>::new(&[
                "enemy.ron",
            ]))
            .add_systems(Startup, load_enemies_system)
            .add_systems(Update, sync_enemies_system);
    }
}

/// Runtime Bevy asset wrapper for `.enemy.ron`.
///
/// 共享 schema crate 负责实际文件结构；运行时层只保留资产包装和加载逻辑。
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EnemyDef(pub SchemaEnemyDef);

impl Deref for EnemyDef {
    type Target = SchemaEnemyDef;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EnemyDef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// --- Registry ---

/// Central registry of all loaded enemy definitions.
///
/// 所有已加载敌人定义的中央注册表。
#[derive(Resource, Default)]
pub struct EnemyRegistry(pub HashMap<String, EnemyDef>);

impl EnemyRegistry {
    pub fn get(&self, id: &str) -> Option<&EnemyDef> {
        self.0.get(id)
    }
}

// --- Fact Projection ---

/// Project enemy data into a fact database for View dynamic resolution.
/// This enables View expressions like `$$enemy_id.action_labels`.
///
/// Note: Only enemies need fact projection because View rules reference enemy
/// data dynamically (HP bars, ACT labels, mercy state). Items are queried
/// directly via ItemRegistry and don't participate in FRE rule evaluation.
///
/// 将敌人数据投影到 fact 数据库，供 View 动态解析使用。
/// 使 View 表达式如 `$$enemy_id.action_labels` 能正常工作。
///
/// 注意：只有敌人需要 fact 投影，因为 View 规则会动态引用敌人数据
/// （HP 条、ACT 标签、mercy 状态）。物品通过 ItemRegistry 直接查询，
/// 不参与 FRE 规则评估。
pub fn project_enemy_facts(enemy: &EnemyDef, db: &mut bevy_fact_rule_event::FactDatabase) {
    use bevy_fact_rule_event::FactValue;
    let id = &enemy.id;
    db.set(
        format!("{id}.name"),
        FactValue::String(enemy.locale.name.clone()),
    );
    db.set(format!("{id}.hp"), FactValue::Int(enemy.stats.hp));
    db.set(format!("{id}.hp_max"), FactValue::Int(enemy.stats.hp));
    db.set(format!("{id}.attack"), FactValue::Int(enemy.stats.attack));
    db.set(format!("{id}.defense"), FactValue::Int(enemy.stats.defense));
    db.set(
        format!("{id}.description"),
        FactValue::String(enemy.description.clone()),
    );
    db.set(
        format!("{id}.mortar_path"),
        FactValue::String(enemy.mortar_path.clone()),
    );
    db.set(
        format!("{id}.act_count"),
        FactValue::Int(enemy.acts.len() as i64),
    );
    db.set(
        format!("{id}.action_labels"),
        FactValue::StringList(enemy.acts.iter().map(|a| a.label.clone()).collect()),
    );
    db.set(
        format!("{id}.action_sequences"),
        FactValue::StringList(enemy.acts.iter().map(|a| a.sequence.clone()).collect()),
    );
    db.set(
        format!("{id}.action_params"),
        FactValue::StringList(enemy.acts.iter().map(|a| a.param.clone()).collect()),
    );
    db.set(
        format!("{id}.mercy_count"),
        FactValue::Int(enemy.mercies.len() as i64),
    );
    db.set(
        format!("{id}.mercy_labels"),
        FactValue::StringList(enemy.mercies.iter().map(|m| m.label.clone()).collect()),
    );
    db.set(
        format!("{id}.mercy_sequences"),
        FactValue::StringList(enemy.mercies.iter().map(|m| m.sequence.clone()).collect()),
    );
    db.set(
        format!("{id}.mercy_params"),
        FactValue::StringList(enemy.mercies.iter().map(|m| m.param.clone()).collect()),
    );
}

// --- Loading ---

#[derive(Resource)]
struct EnemyFolderHandle(Handle<LoadedFolder>);

fn load_enemies_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Starting to load enemies from folder 'data/enemies'...");
    let handle = asset_server.load_folder("data/enemies");
    commands.insert_resource(EnemyFolderHandle(handle));
}

fn sync_enemies_system(
    mut events: MessageReader<AssetEvent<LoadedFolder>>,
    enemy_folder: Option<Res<EnemyFolderHandle>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    enemy_assets: Res<Assets<EnemyDef>>,
    mut registry: ResMut<EnemyRegistry>,
) {
    let Some(folder_handle) = enemy_folder else {
        return;
    };

    for event in events.read() {
        let AssetEvent::LoadedWithDependencies { id } = event else {
            continue;
        };
        if *id != folder_handle.0.id() {
            continue;
        }

        info!("Enemy folder loaded. Indexing enemies from .enemy.ron files...");

        let Some(folder) = loaded_folders.get(&folder_handle.0) else {
            continue;
        };
        for handle in &folder.handles {
            let id = handle.id().typed::<EnemyDef>();
            if let Some(enemy) = enemy_assets.get(id) {
                info!("Registered Enemy: [{}] {}", enemy.id, enemy.locale.name);
                registry.0.insert(enemy.id.clone(), enemy.clone());
            }
        }
        info!(
            "Enemy Registry initialization complete. Total enemies: {}",
            registry.0.len()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fact_rule_event::FactValue;

    #[test]
    fn parses_enemy_asset_from_shared_schema() {
        let ron = r#"(
            id: "dummy",
            locale: (
                name: "enemy.dummy",
                file: "enemies",
            ),
            stats: (
                hp: 30,
                attack: 5,
                defense: 2,
            ),
            description: "A schema-backed enemy.",
            mortar_path: "battle/dummy.mortar",
            acts: [
                (
                    label: "enemy.act.check",
                    sequence: "battle/check.sequence.ron",
                    param: "check",
                ),
            ],
            mercies: [
                (
                    label: "enemy.mercy.spare",
                    sequence: "battle/spare.sequence.ron",
                ),
            ],
        )"#;

        let enemy: EnemyDef = ron::from_str(ron).expect("enemy asset");

        assert_eq!(enemy.id, "dummy");
        assert_eq!(enemy.locale.name, "enemy.dummy");
        assert_eq!(enemy.stats.hp, 30);
        assert_eq!(enemy.acts[0].sequence, "battle/check.sequence.ron");
    }

    #[test]
    fn projects_enemy_facts_from_runtime_wrapper() {
        let enemy = EnemyDef(SchemaEnemyDef {
            id: "dummy".to_string(),
            locale: souprune_schema::enemy::LocaleInfo {
                name: "enemy.dummy".to_string(),
                file: "enemies".to_string(),
            },
            stats: souprune_schema::enemy::CombatStats {
                hp: 30,
                attack: 5,
                defense: 2,
            },
            description: "A schema-backed enemy.".to_string(),
            mortar_path: "battle/dummy.mortar".to_string(),
            acts: vec![ActionOption {
                label: "enemy.act.check".to_string(),
                sequence: "battle/check.sequence.ron".to_string(),
                param: "check".to_string(),
            }],
            mercies: vec![],
        });
        let mut facts = bevy_fact_rule_event::FactDatabase::default();

        project_enemy_facts(&enemy, &mut facts);

        assert_eq!(
            facts.get_by_str("dummy.name"),
            Some(&FactValue::String("enemy.dummy".to_string()))
        );
        assert_eq!(facts.get_by_str("dummy.hp"), Some(&FactValue::Int(30)));
        assert_eq!(
            facts.get_by_str("dummy.action_labels"),
            Some(&FactValue::StringList(vec!["enemy.act.check".to_string()]))
        );
    }
}
