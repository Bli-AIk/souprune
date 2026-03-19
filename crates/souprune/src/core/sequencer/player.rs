//! # sequencer/player.rs
//!
//! ## Module Overview
//!
//! Player-related systems for the battle sequencer.
//!
//! 战斗序列管理器的玩家相关系统。

use super::chapter_schema::{Chapter, PlayerAction};
use super::context::*;
use crate::app_state::ModeScoped;
use crate::app_state::battle::collision::BoundToBattleBox;
use crate::app_state::battle::danmaku::BattleInvincibilityConfig;
use crate::app_state::battle::player_config_schema::{BattlePlayerConfig, ColliderShape};
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
pub fn process_player_action_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    asset_server: Res<AssetServer>,
    mut player_query: Query<(Entity, &mut Transform), (With<BehaviorParams>, With<ModeScoped>)>,
) {
    for (entity, active_chapter) in query.iter() {
        let Chapter::SetPlayer(action) = &active_chapter.chapter else {
            continue;
        };
        match action {
            PlayerAction::Spawn {
                config_path,
                position,
            } if config_path.ends_with(".battle_player.ron") => {
                let handle = asset_server.load::<BattlePlayerConfig>(config_path);
                commands.spawn((
                    PlayerSpawnRequest {
                        config_handle: handle,
                        position: position.unwrap_or(Vec2::ZERO),
                    },
                    ModeScoped("battle".to_string()),
                ));
                commands.entity(entity).insert(ChapterFinished);
            }
            // Non-battle config_path → handled by state-specific systems (e.g., overworld)
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

/// System to process player spawn requests when configs are loaded.
///
/// 当配置加载完成时处理玩家生成请求的系统。
pub fn process_player_spawn_requests(
    mut commands: Commands,
    query: Query<(Entity, &PlayerSpawnRequest)>,
    configs: Option<Res<Assets<BattlePlayerConfig>>>,
    asset_server: Res<AssetServer>,
    invincibility_config: Option<ResMut<BattleInvincibilityConfig>>,
) {
    let (Some(configs), Some(mut invincibility_config)) = (configs, invincibility_config) else {
        return;
    };
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

            // Update invincibility config from player config
            // 从玩家配置更新无敌配置
            let inv_cfg = &config.invincibility;
            invincibility_config.duration = inv_cfg.duration;
            invincibility_config.flash_interval = inv_cfg.flash_interval;
            invincibility_config.normal_color = inv_cfg.normal_color;
            invincibility_config.flash_color = inv_cfg.flash_color;
            invincibility_config.damage_sound = inv_cfg.damage_sound.clone();

            if let Some(ref sound) = inv_cfg.damage_sound {
                info!("Configured damage sound: {}", sound);
            }

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
                BoundToBattleBox(config.default_box.clone()),
                ModeScoped("battle".to_string()),
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
