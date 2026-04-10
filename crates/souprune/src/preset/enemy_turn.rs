//! Enemy turn dispatch — resolves `PickEnemyTurn` chapters into concrete `RunSequence`.
//!
//! 敌人回合调度 — 将 `PickEnemyTurn` 章节解析为具体的 `RunSequence`。
//!
//! Runs before `advance_battle_flow_system` so that by the time the sequencer
//! pops the next chapter, `PickEnemyTurn` has already been replaced.
//!
//! 在 `advance_battle_flow_system` 之前运行，使得 sequencer 弹出下一个章节时，
//! `PickEnemyTurn` 已经被替换。

use bevy::prelude::*;
use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};
use rand::prelude::*;
use std::collections::HashMap;

use crate::core::sequencer::chapter_schema::Chapter;
use crate::core::sequencer::context::SequenceContext;
use crate::core::view::{ActiveView, ViewRoot};
use crate::preset::enemy::EnemyRegistry;
use souprune_schema::enemy::TurnStrategy;

/// Resolve `PickEnemyTurn` at the front of the queue into a `RunSequence`.
///
/// 将队列头部的 `PickEnemyTurn` 解析为 `RunSequence`。
pub fn resolve_pick_enemy_turn_system(
    mut context: ResMut<SequenceContext>,
    registry: Res<EnemyRegistry>,
    mut layered_db: ResMut<LayeredFactDatabase>,
    view_roots: Query<&ViewRoot, With<ActiveView>>,
) {
    let Some(Chapter::PickEnemyTurn {
        enemy_id,
        enemy_id_fact,
    }) = context.chapters.first()
    else {
        return;
    };

    // Resolve the actual enemy id — literal takes precedence over fact lookup.
    // Checks ViewRoot local_facts first (where RunSequence params are injected),
    // then falls back to the global LayeredFactDatabase.
    let resolved_id = if let Some(id) = enemy_id {
        id.clone()
    } else if let Some(fact_key) = enemy_id_fact {
        resolve_fact_string(fact_key, &layered_db, &view_roots)
    } else {
        warn!("PickEnemyTurn: neither enemy_id nor enemy_id_fact specified — skipping");
        context.chapters.remove(0);
        return;
    };

    let Some(enemy) = registry.get(&resolved_id) else {
        warn!("PickEnemyTurn: enemy '{resolved_id}' not found in registry — skipping");
        context.chapters.remove(0);
        return;
    };

    if enemy.turns.is_empty() {
        warn!("PickEnemyTurn: enemy '{resolved_id}' has no turns defined — skipping");
        context.chapters.remove(0);
        return;
    };

    let turn_path = pick_turn(
        &resolved_id,
        &enemy.turns,
        &enemy.turn_strategy,
        &mut layered_db,
    );

    info!(
        "PickEnemyTurn: resolved enemy '{}' → turn '{}'",
        resolved_id, turn_path
    );

    // Replace PickEnemyTurn with RunSequence.
    context.chapters[0] = Chapter::RunSequence {
        path: Some(turn_path),
        path_fact: None,
        params: HashMap::new(),
    };
}

/// Read a string value from ViewRoot local_facts (where RunSequence params live)
/// or fall back to the layered fact database.
///
/// 从 ViewRoot local_facts（RunSequence 参数的存储位置）读取字符串值，
/// 如果找不到则回退到分层事实数据库。
fn resolve_fact_string(
    key: &str,
    layered_db: &LayeredFactDatabase,
    view_roots: &Query<&ViewRoot, With<ActiveView>>,
) -> String {
    // Check ViewRoot local_facts first (RunSequence params are injected here).
    for view_root in view_roots.iter() {
        if let Some(s) = view_root.local_facts.get_string(key) {
            return s.to_string();
        }
    }
    // Fall back to layered DB.
    if let Some(FactValue::String(s)) = layered_db.get_by_str(key) {
        return s.clone();
    }
    warn!("PickEnemyTurn: fact key '{key}' not found or not a string");
    String::new()
}

/// Select the next turn sequence path based on the enemy's strategy.
fn pick_turn(
    enemy_id: &str,
    turns: &[String],
    strategy: &TurnStrategy,
    db: &mut LayeredFactDatabase,
) -> String {
    let index_key = format!("{enemy_id}.turn_index");

    match strategy {
        TurnStrategy::Sequential => {
            let current = db
                .get_by_str(&index_key)
                .and_then(|v| match v {
                    FactValue::Int(i) => Some(*i as usize),
                    _ => None,
                })
                .unwrap_or(0);
            let path = turns[current % turns.len()].clone();
            db.set(index_key, FactValue::Int(((current + 1) % turns.len()) as i64));
            path
        }
        TurnStrategy::Random => {
            let mut rng = rand::rng();
            let idx = rng.random_range(0..turns.len());
            turns[idx].clone()
        }
        TurnStrategy::Shuffle => {
            let pool_key = format!("{enemy_id}.turn_pool");
            let mut pool: Vec<usize> = db
                .get_by_str(&pool_key)
                .and_then(|v| match v {
                    FactValue::StringList(list) => Some(
                        list.iter()
                            .filter_map(|s| s.parse::<usize>().ok())
                            .collect(),
                    ),
                    _ => None,
                })
                .unwrap_or_default();

            if pool.is_empty() {
                pool = (0..turns.len()).collect();
                pool.shuffle(&mut rand::rng());
            }

            let idx = pool.remove(0);
            let path = turns[idx % turns.len()].clone();

            db.set(
                pool_key,
                FactValue::StringList(pool.iter().map(|i| i.to_string()).collect()),
            );
            path
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};

    #[test]
    fn sequential_strategy_cycles() {
        let mut db = LayeredFactDatabase::default();
        let turns = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        let t0 = pick_turn("e", &turns, &TurnStrategy::Sequential, &mut db);
        let t1 = pick_turn("e", &turns, &TurnStrategy::Sequential, &mut db);
        let t2 = pick_turn("e", &turns, &TurnStrategy::Sequential, &mut db);
        let t3 = pick_turn("e", &turns, &TurnStrategy::Sequential, &mut db);

        assert_eq!(t0, "a");
        assert_eq!(t1, "b");
        assert_eq!(t2, "c");
        assert_eq!(t3, "a"); // wraps around
    }

    #[test]
    fn shuffle_strategy_exhausts_pool() {
        let mut db = LayeredFactDatabase::default();
        let turns = vec!["x".to_string(), "y".to_string()];

        let t0 = pick_turn("e", &turns, &TurnStrategy::Shuffle, &mut db);
        let t1 = pick_turn("e", &turns, &TurnStrategy::Shuffle, &mut db);

        // Both turns should appear exactly once before reshuffle.
        let mut picked = vec![t0, t1];
        picked.sort();
        assert_eq!(picked, vec!["x", "y"]);

        // Pool should now be empty → next pick triggers reshuffle.
        let pool = db.get_by_str("e.turn_pool");
        assert_eq!(pool, Some(&FactValue::StringList(vec![])));
    }
}
