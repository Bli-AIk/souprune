use super::super::chapter_schema::Chapter;
use super::super::context::{ActiveChapter, ChapterFinished};
use bevy::prelude::*;
use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};

#[derive(Component)]
pub struct LoadEnemiesState {
    pub handles: Vec<Handle<crate::core::enemy::EnemyDef>>,
    pub processed: bool,
}

pub fn process_load_enemies_chapter_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<ChapterFinished>, Without<LoadEnemiesState>)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, active) in query.iter() {
        if let Chapter::LoadEnemies { enemies } = &active.chapter {
            let handles = enemies
                .iter()
                .map(|path| {
                    info!("LoadEnemies Chapter: Loading '{}'", path);
                    asset_server.load::<crate::core::enemy::EnemyDef>(path.clone())
                })
                .collect();

            commands.entity(entity).insert(LoadEnemiesState {
                handles,
                processed: false,
            });
        }
    }
}

pub fn complete_load_enemies_chapter_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LoadEnemiesState), Without<ChapterFinished>>,
    mut layered_db: ResMut<LayeredFactDatabase>,
    enemy_assets: Res<Assets<crate::core::enemy::EnemyDef>>,
    mut enemy_registry: ResMut<crate::core::enemy::EnemyRegistry>,
) {
    for (entity, mut state) in query.iter_mut() {
        if state.processed {
            continue;
        }

        let all_loaded = state.handles.iter().all(|h| enemy_assets.contains(h));
        if !all_loaded {
            continue;
        }

        let mut enemy_ids = Vec::new();
        let mut enemy_names = Vec::new();
        let mut enemy_hps = Vec::new();
        let mut enemy_hp_maxs = Vec::new();
        let mut enemy_attacks = Vec::new();
        let mut enemy_defenses = Vec::new();

        for handle in &state.handles {
            if let Some(enemy) = enemy_assets.get(handle) {
                crate::core::enemy::project_enemy_facts(enemy, layered_db.local_mut());

                enemy_ids.push(enemy.id.clone());
                enemy_names.push(enemy.locale.name.clone());
                enemy_hps.push(enemy.stats.hp);
                enemy_hp_maxs.push(enemy.stats.hp);
                enemy_attacks.push(enemy.stats.attack);
                enemy_defenses.push(enemy.stats.defense);

                enemy_registry.0.insert(enemy.id.clone(), enemy.clone());
                info!("LoadEnemies: Loaded enemy '{}'", enemy.id);
            }
        }

        layered_db.set("enemy_ids", FactValue::StringList(enemy_ids));
        layered_db.set("enemy_names", FactValue::StringList(enemy_names));
        layered_db.set("enemy_hps", FactValue::IntList(enemy_hps));
        layered_db.set("enemy_hp_maxs", FactValue::IntList(enemy_hp_maxs));
        layered_db.set("enemy_attacks", FactValue::IntList(enemy_attacks));
        layered_db.set("enemy_defenses", FactValue::IntList(enemy_defenses));

        state.processed = true;
        commands.entity(entity).insert(ChapterFinished);
        info!("LoadEnemies Chapter: Completed");
    }
}
