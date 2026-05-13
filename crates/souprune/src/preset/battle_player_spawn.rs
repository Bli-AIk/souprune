//! Battle player spawn system — handles SetPlayer(Spawn) chapters
//! with `.battle_player.ron` configs.
//!
//! 战斗玩家生成系统 — 处理使用 `.battle_player.ron` 配置的 SetPlayer(Spawn) 章节。

use crate::core::battle_box::BoundToBattleBox;
use crate::core::battle_player::{BattleInvincibilityConfig, BattlePlayerConfig};
use crate::core::danmaku::BulletTarget;
use crate::core::mod_system::{BehaviorContext, BehaviorParams, BehaviorVelocity};
use crate::core::mode::ModeScoped;
use crate::core::sequencer::chapter_schema::{Chapter, PlayerAction};
use crate::core::sequencer::context::*;
use bevy::prelude::*;

/// Component for pending player spawn requests.
#[derive(Component)]
pub struct PlayerSpawnRequest {
    pub config_handle: Handle<BattlePlayerConfig>,
    pub position: Vec2,
}

/// System that intercepts SetPlayer(Spawn) chapters with `.battle_player.ron` paths,
/// creating a `PlayerSpawnRequest` entity for deferred processing.
pub fn process_battle_player_spawn_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, active_chapter) in query.iter() {
        let Chapter::SetPlayer(PlayerAction::Spawn {
            config_path,
            position,
        }) = &active_chapter.chapter
        else {
            continue;
        };

        if !config_path.ends_with(".battle_player.ron") {
            continue;
        }

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
}

/// System to process player spawn requests when configs are loaded.
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

            let physics_collider = config.physics_collider();
            let damage_trigger = config.damage_trigger();
            let sprite_path = config.sprite_path().to_string();
            let default_mode_id = config.default_mode_id().to_string();
            let default_box = config.default_box().to_string();

            // Update invincibility config from player config
            let inv_cfg = config.invincibility();
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
                    image: asset_server.load(sprite_path),
                    color: config.sprite_color(),
                    ..default()
                },
                Transform::from_translation(req.position.extend(config.z_position())),
                physics_collider.clone(),
                damage_trigger.clone(),
                BehaviorParams {
                    behavior_id: default_mode_id,
                    context: BehaviorContext::new("battle"),
                },
                BehaviorVelocity::default(),
                BulletTarget::new(),
                BoundToBattleBox(default_box),
                ModeScoped("battle".to_string()),
                Name::new("BattlePlayer"),
            ));

            info!(
                "Spawned player with physics collider: {:?}, damage trigger: {:?}, at z: {}",
                physics_collider,
                damage_trigger,
                config.z_position()
            );

            commands.entity(entity).despawn();
        }
    }
}
