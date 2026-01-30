//! # sequencer/player.rs
//!
//! ## Module Overview
//!
//! Player-related systems for the battle sequencer.
//!
//! 战斗序列管理器的玩家相关系统。

use super::super::chapter_schema::{Chapter, PlayerAction};
use super::super::player_config_schema::{BattlePlayerConfig, ColliderShape};
use super::context::*;
use crate::core::collision::{PhysicsCollider, TriggerCollider};
use crate::core::danmaku::BulletTarget;
use crate::core::mod_system::{BehaviorParams, BehaviorVelocity};
use bevy::prelude::*;

/// Component for pending player spawn requests.
///
/// 待处理的玩家生成请求组件。
#[derive(Component)]
pub struct PlayerSpawnRequest {
    pub config_handle: Handle<BattlePlayerConfig>,
    pub position: Vec2,
}

/// System to process player actions.
///
/// 处理玩家动作的系统。
#[allow(clippy::type_complexity)]
pub fn process_player_action_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    asset_server: Res<AssetServer>,
    mut player_query: Query<
        &mut Transform,
        (
            With<BehaviorParams>,
            With<crate::app_state::battle::BattleEntity>,
        ),
    >,
) {
    for (entity, active_chapter) in query.iter() {
        if let Chapter::SetPlayer(action) = &active_chapter.chapter {
            match action {
                PlayerAction::Spawn {
                    config_path,
                    position,
                } => {
                    let handle = asset_server.load::<BattlePlayerConfig>(config_path);
                    commands.spawn((
                        PlayerSpawnRequest {
                            config_handle: handle,
                            position: *position,
                        },
                        crate::app_state::battle::BattleEntity,
                    ));
                }
                PlayerAction::Teleport(pos) => {
                    for mut transform in player_query.iter_mut() {
                        transform.translation = pos.extend(0.0);
                        info!("Player teleported to {}", pos);
                    }
                }
                _ => {}
            }
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}

/// System to process player spawn requests when configs are loaded.
///
/// 当配置加载完成时处理玩家生成请求的系统。
pub fn process_player_spawn_requests(
    mut commands: Commands,
    query: Query<(Entity, &PlayerSpawnRequest)>,
    configs: Res<Assets<BattlePlayerConfig>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, req) in query.iter() {
        if let Some(config) = configs.get(&req.config_handle) {
            info!("Config loaded. Spawning player...");

            let physics_collider = match &config.physics_collider.shape {
                ColliderShape::Circle { radius } => PhysicsCollider::Circle { radius: *radius },
                ColliderShape::Box { half_size } => PhysicsCollider::Box {
                    half_size: *half_size,
                },
            };

            let damage_trigger = match &config.damage_trigger.shape {
                ColliderShape::Circle { radius } => TriggerCollider::Circle { radius: *radius },
                ColliderShape::Box { half_size } => TriggerCollider::Box {
                    half_size: *half_size,
                },
            };

            commands.spawn((
                Sprite {
                    image: asset_server.load(&config.sprite_path),
                    color: config.color,
                    ..default()
                },
                Transform::from_translation(req.position.extend(config.z_position)),
                physics_collider.clone(),
                damage_trigger.clone(),
                BehaviorParams {
                    mode_id: config.default_mode_id.clone(),
                },
                BehaviorVelocity::default(),
                BulletTarget::new(),
                crate::app_state::battle::BattleEntity,
                Name::new("BattlePlayer"),
            ));

            info!(
                "Spawned player with physics collider: {:?}, damage trigger: {:?}, at z: {}",
                physics_collider, damage_trigger, config.z_position
            );

            commands.entity(entity).despawn();
        }
    }
}
