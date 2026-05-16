//! # sequencer/player.rs
//!
//! ## Module Overview
//!
//! Player-related systems for the battle sequencer.
//!
//! 战斗序列管理器的玩家相关系统。

use super::chapter_schema::{Chapter, PlayerAction};
use super::context::*;
use crate::core::mod_system::BehaviorParams;
use crate::core::mode::ModeScoped;
use bevy::prelude::*;

/// System to process generic player actions (Teleport, Despawn, SetMode, SetActive).
///
/// Spawn handling is owned by the active mode runtime or project custom actions.
///
/// 处理通用玩家动作的系统。Spawn 由当前模式运行时或项目自定义 action 处理。
pub fn process_player_action_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    mut player_query: Query<(Entity, &mut Transform), (With<BehaviorParams>, With<ModeScoped>)>,
) {
    for (entity, active_chapter) in query.iter() {
        let Chapter::SetPlayer(action) = &active_chapter.chapter else {
            continue;
        };
        match action {
            // Spawn is handled by mode-specific systems.
            PlayerAction::Spawn { .. } => {}
            PlayerAction::Teleport(pos) => {
                for (_, mut transform) in player_query.iter_mut() {
                    transform.translation = pos.extend(0.0);
                    info!("Player teleported to {}", pos);
                }
                commands.entity(entity).insert(ChapterFinished);
            }
            PlayerAction::Despawn => {
                for (player_entity, _) in player_query.iter() {
                    commands.entity(player_entity).despawn();
                    info!("Battle player despawned");
                }
                commands.entity(entity).insert(ChapterFinished);
            }
            PlayerAction::SetMode(_) | PlayerAction::SetActive(_) => {
                // TODO: Implement mode switching and active state toggling
                commands.entity(entity).insert(ChapterFinished);
            }
        }
    }
}
