//! # danmaku.rs
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Battle-specific danmaku configuration.
//! This module configures the core danmaku system for battle state.
//!
//! 战斗特定的弹幕配置。
//! 此模块为战斗状态配置核心弹幕系统。

// Re-export core danmaku types for backwards compatibility
pub use crate::core::danmaku::*;

use crate::app_state::AppState;
use crate::core::collision::TriggerCollider;
use crate::core::danmaku::DanmakuSpawnContext;
#[cfg(not(feature = "experimental"))]
use crate::core::danmaku::DanmakuUpdate;
use crate::core::mod_system::BehaviorParams;
use bevy::prelude::*;

use super::BattleUpdate;

/// Battle-specific danmaku plugin.
/// Configures CoreDanmakuPlugin for battle state.
///
/// 战斗特定的弹幕插件。
/// 为战斗状态配置 CoreDanmakuPlugin。
pub struct DanmakuPlugin;

impl Plugin for DanmakuPlugin {
    fn build(&self, app: &mut App) {
        // Configure DanmakuUpdate run condition:
        // When experimental feature is OFF, only run in Battle state
        // When experimental feature is ON, the run_if is configured in overworld.rs to allow both states
        #[cfg(not(feature = "experimental"))]
        app.configure_sets(Update, DanmakuUpdate.run_if(in_state(AppState::Battle)));

        // Set spawn context to Battle when entering battle state
        app.add_systems(OnEnter(AppState::Battle), set_battle_context);

        // Add damage detection system (experimental feature)
        #[cfg(feature = "experimental")]
        app.add_systems(Update, battle_damage_detection_system.in_set(BattleUpdate));

        // Register reflect types for inspector
        app.register_type::<DanmakuPerformance>()
            .register_type::<BulletPrototype>()
            .register_type::<BulletVisual>()
            .register_type::<ColliderShape>()
            .register_type::<BulletBehavior>()
            .register_type::<LinearConfig>()
            .register_type::<OrbitalConfig>()
            .register_type::<SineConfig>()
            .register_type::<TweenConfig>()
            .register_type::<TweenTarget>()
            .register_type::<Easing>()
            .register_type::<TimelineEvent>()
            .register_type::<SpawnPattern>()
            .register_type::<EdgeSide>();
    }
}

fn set_battle_context(mut spawn_context: ResMut<DanmakuSpawnContext>) {
    *spawn_context = DanmakuSpawnContext::battle();
    info!("Danmaku: Set spawn context to Battle");
}

/// System to detect bullet collision with player in battle mode.
///
/// 检测战斗模式下弹幕与玩家碰撞的系统。
#[cfg(feature = "experimental")]
fn battle_damage_detection_system(
    mut commands: Commands,
    player_query: Query<(&Transform, &TriggerCollider), With<BehaviorParams>>,
    mut bullet_query: Query<
        (
            Entity,
            &Transform,
            &TriggerCollider,
            &crate::core::danmaku::BulletDamage,
            &crate::core::danmaku::BulletHitBehavior,
            &mut crate::core::danmaku::BulletLastHitTime,
            &crate::core::danmaku::BulletMotionState,
        ),
        With<crate::core::danmaku::Bullet>,
    >,
    #[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))] audio: Res<
        bevy_kira_audio::Audio,
    >,
    asset_server: Res<AssetServer>,
) {
    let Ok((player_transform, player_hitbox)) = player_query.single() else {
        return;
    };

    let player_center = player_transform.translation.truncate();

    // TODO: Battle player movement detection for blue/orange soul mechanics
    // For now, assume player is always "moving" in battle mode
    let player_is_moving = true;

    for (
        bullet_entity,
        bullet_transform,
        bullet_collider,
        bullet_damage,
        hit_behavior,
        mut last_hit_time,
        motion_state,
    ) in bullet_query.iter_mut()
    {
        let bullet_center = bullet_transform.translation.truncate();

        // Check collision between player hitbox and bullet collider
        if !check_battle_collision(player_hitbox, player_center, bullet_collider, bullet_center) {
            continue;
        }

        // Check invincibility frames
        if hit_behavior.invincibility_duration > 0.0 {
            let time_since_last_hit = motion_state.elapsed - last_hit_time.0;
            if time_since_last_hit < hit_behavior.invincibility_duration {
                continue;
            }
        }

        // Check movement-based damage conditions
        let should_damage = if hit_behavior.damage_on_player_moving {
            // "Blue soul" style: only damage when player is moving
            player_is_moving
        } else if hit_behavior.damage_on_player_stationary {
            // "Orange soul" style: only damage when player is stationary
            !player_is_moving
        } else {
            // Default: always damage
            true
        };

        if should_damage {
            // Update last hit time
            last_hit_time.0 = motion_state.elapsed;

            // Play hurt sound
            #[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
            crate::core::audio::play_sound(&audio, &asset_server, "hurtsound.wav");
            #[cfg(feature = "firewheel")]
            crate::core::audio::play_sound(&mut commands, &asset_server, "hurtsound.wav");

            info!("Battle: Player hit by bullet! Damage: {}", bullet_damage.0);
        }

        // Handle despawn behavior
        if hit_behavior.despawn_on_hit && should_damage {
            commands
                .entity(bullet_entity)
                .insert(crate::core::danmaku::DespawnBullet);
        }
    }
}

/// Helper function to check collision between two trigger colliders.
///
/// 检查两个触发器碰撞体之间是否发生碰撞的辅助函数。
#[cfg(feature = "experimental")]
fn check_battle_collision(
    a: &TriggerCollider,
    a_center: Vec2,
    b: &TriggerCollider,
    b_center: Vec2,
) -> bool {
    match (a, b) {
        (TriggerCollider::Circle { radius: r1 }, TriggerCollider::Circle { radius: r2 }) => {
            let dist = a_center.distance(b_center);
            dist <= r1 + r2
        }
        (TriggerCollider::Box { half_size: hs1 }, TriggerCollider::Box { half_size: hs2 }) => {
            let diff = (a_center - b_center).abs();
            diff.x <= hs1.x + hs2.x && diff.y <= hs1.y + hs2.y
        }
        (TriggerCollider::Circle { radius }, TriggerCollider::Box { half_size })
        | (TriggerCollider::Box { half_size }, TriggerCollider::Circle { radius }) => {
            let (circle_center, box_center, box_half) =
                if matches!(a, TriggerCollider::Circle { .. }) {
                    (a_center, b_center, *half_size)
                } else {
                    (b_center, a_center, *half_size)
                };
            // Closest point on box to circle center
            let closest = Vec2::new(
                circle_center
                    .x
                    .clamp(box_center.x - box_half.x, box_center.x + box_half.x),
                circle_center
                    .y
                    .clamp(box_center.y - box_half.y, box_center.y + box_half.y),
            );
            circle_center.distance(closest) <= *radius
        }
    }
}
