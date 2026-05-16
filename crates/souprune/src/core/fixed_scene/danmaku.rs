//! # danmaku.rs
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Battle-specific danmaku configuration.
//! This module configures the core danmaku system for fixed-scene mode.
//!
//! 战斗特定的弹幕配置。
//! 此模块为战斗状态配置核心弹幕系统。

pub use crate::core::danmaku::*;

use crate::core::collision::TriggerCollider;
use crate::core::mod_system::BehaviorParams;
use crate::core::mode::SequenceMode;
use bevy::prelude::*;

use crate::core::fixed_scene::FixedSceneUpdate;

const INVINCIBILITY_DURATION_FACT: &str = "battle:damage_target:invincibility_duration";
const FLASH_INTERVAL_FACT: &str = "battle:damage_target:flash_interval";
const DAMAGE_SOUND_FACT: &str = "battle:damage_target:damage_sound";
const NORMAL_COLOR_PREFIX: &str = "battle:damage_target:normal_color";
const FLASH_COLOR_PREFIX: &str = "battle:damage_target:flash_color";

/// Resource to track player invincibility state in battle mode.
///
/// 追踪战斗模式下玩家无敌状态的资源。
#[derive(Resource, Default)]
pub struct BattleTargetInvincibility {
    /// Whether player is currently invincible
    pub active: bool,
    /// Remaining invincibility time
    pub timer: f32,
    /// Flash timer for heart color toggle
    pub flash_timer: f32,
    /// Current flash state (true = normal color, false = flash color)
    pub flash_state: bool,
}

impl BattleTargetInvincibility {
    /// Start invincibility with the given duration.
    pub fn start(&mut self, duration: f32) {
        self.active = true;
        self.timer = duration;
        self.flash_timer = 0.0;
        self.flash_state = true;
    }

    /// Check if player is invincible.
    pub fn is_invincible(&self) -> bool {
        self.active && self.timer > 0.0
    }
}

/// Battle-specific danmaku plugin.
/// Configures CoreDanmakuPlugin for fixed-scene mode.
///
/// 战斗特定的弹幕插件。
/// 为战斗状态配置 CoreDanmakuPlugin。
pub struct DanmakuPlugin;

impl Plugin for DanmakuPlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        // Set spawn context to fixed-scene mode when entering fixed-scene mode
        app.add_systems(
            schedule,
            set_battle_context.run_if(super::on_entering_fixed_scene),
        );

        // Add damage detection and invincibility systems
        app.init_resource::<BattleTargetInvincibility>()
            .add_systems(
                schedule,
                (
                    battle_damage_detection_system,
                    update_battle_invincibility_system,
                )
                    .chain()
                    .in_set(FixedSceneUpdate),
            );

        // Danmaku asset schema now comes from `souprune_schema`.
        // Runtime keeps only a thin wrapper and helper layer here.
    }
}

fn set_battle_context(
    mut spawn_context: ResMut<DanmakuSpawnContext>,
    sequence_mode: Res<SequenceMode>,
) {
    let Some(mode_name) = sequence_mode.0.as_deref() else {
        warn!("FixedScene danmaku: no active mode while setting spawn context.");
        return;
    };
    *spawn_context = DanmakuSpawnContext::with_mode(mode_name);
    info!("Danmaku: Set spawn context to {mode_name}");
}

/// System to detect bullet collision with player in battle mode.
///
/// 检测战斗模式下弹幕与玩家碰撞的系统。
fn battle_damage_detection_system(
    mut commands: Commands,
    time: Res<Time>,
    mut player_invincibility: ResMut<BattleTargetInvincibility>,
    mut layered_db: ResMut<bevy_fact_rule_event::LayeredFactDatabase>,
    player_query: Query<(&GlobalTransform, &TriggerCollider), With<BehaviorParams>>,
    mut bullet_query: Query<
        (
            Entity,
            &GlobalTransform,
            &TriggerCollider,
            &crate::core::danmaku::BulletDamage,
            &crate::core::danmaku::BulletHitBehavior,
            &mut crate::core::danmaku::BulletLastHitTime,
            &crate::core::danmaku::BulletMotionState,
        ),
        With<crate::core::danmaku::Bullet>,
    >,
    mut last_player_state: Local<Option<(Vec2, f64)>>,
    mut last_damage_log_time: Local<f64>,
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    mut audio_cache: ResMut<crate::core::audio::AudioSourceCache>,
) {
    let Ok((player_transform, player_hitbox)) = player_query.single() else {
        return;
    };

    let player_center = player_transform.translation().truncate();
    let current_time = time.elapsed_secs_f64();

    // Battle player movement detection
    // Check if player position changed significantly since last frame
    let player_is_moving = if let Some((last_pos, last_time)) = *last_player_state {
        // If too much time passed (e.g. paused, lag spike, or system didn't run), reset detection
        if current_time - last_time > time.delta_secs_f64() * 1.5 {
            false
        } else {
            player_center.distance_squared(last_pos) > 0.0001 // sqrt(0.0001) = 0.01 threshold
        }
    } else {
        false
    };

    // Update last state
    *last_player_state = Some((player_center, current_time));

    let damage_settings = battle_damage_settings(&layered_db);

    // Check if player is invincible
    let is_invincible = player_invincibility.is_invincible();

    // Debug: Count bullets
    let bullet_count = bullet_query.iter().count();
    if bullet_count > 0 && current_time - *last_damage_log_time > 1.0 {
        *last_damage_log_time = current_time;
        info!(
            "[Battle Damage] Found {} bullets, player at {:?}",
            bullet_count, player_center
        );
    }

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
        let bullet_center = bullet_transform.translation().truncate();

        // Check collision between player hitbox and bullet collider
        if !check_battle_collision(player_hitbox, player_center, bullet_collider, bullet_center) {
            continue;
        }

        // Check bullet's own invincibility frames (for persistent bullets)
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

        // If player is invincible, don't deal damage but still handle despawn
        if is_invincible {
            // Handle despawn behavior even during invincibility
            if hit_behavior.despawn_on_hit && should_damage {
                commands
                    .entity(bullet_entity)
                    .insert(crate::core::danmaku::DespawnBullet);
            }
            continue;
        }

        if should_damage {
            // Update last hit time
            last_hit_time.0 = motion_state.elapsed;

            // Apply damage to player HP (fixed integer damage)
            let damage = bullet_damage.0 as usize;
            let current_hp = layered_db.get_int("player:hp").unwrap_or(20) as usize;
            let hp_max = layered_db.get_int("player:hp_max").unwrap_or(20) as usize;
            let new_hp = current_hp.saturating_sub(damage);
            layered_db.set_global("player:hp", new_hp as i64);

            // Start player invincibility
            player_invincibility.start(damage_settings.duration);

            // Play hurt sound from config
            if let Some(sound_path) = &damage_settings.damage_sound {
                crate::core::audio::play_sound_full_path(
                    &audio,
                    &asset_server,
                    &mut audio_cache,
                    sound_path,
                );
            }

            info!(
                "Battle: Player hit! Damage: {}, HP: {}/{}",
                damage, new_hp, hp_max
            );

            // Handle despawn behavior
            if hit_behavior.despawn_on_hit {
                commands
                    .entity(bullet_entity)
                    .insert(crate::core::danmaku::DespawnBullet);
            }

            // Only one bullet can deal damage per frame
            break;
        }
    }
}

/// System to update battle player invincibility timer and heart flashing effect.
///
/// 更新战斗玩家无敌时间和心形闪烁效果的系统。
fn update_battle_invincibility_system(
    time: Res<Time>,
    layered_db: Res<bevy_fact_rule_event::LayeredFactDatabase>,
    mut player_invincibility: ResMut<BattleTargetInvincibility>,
    mut player_query: Query<&mut Sprite, With<BehaviorParams>>,
) {
    if !player_invincibility.active {
        return;
    }

    let delta = time.delta_secs();
    let damage_settings = battle_damage_settings(&layered_db);

    // Update invincibility timer
    player_invincibility.timer -= delta;

    if player_invincibility.timer <= 0.0 {
        // Invincibility ended - reset to normal color
        player_invincibility.active = false;
        player_invincibility.timer = 0.0;
        player_invincibility.flash_state = true;

        // Reset heart color to pure red
        for mut sprite in player_query.iter_mut() {
            sprite.color = damage_settings.normal_color;
        }

        info!("Battle: Player invincibility ended");
        return;
    }

    // Update flash timer
    player_invincibility.flash_timer += delta;

    if player_invincibility.flash_timer >= damage_settings.flash_interval {
        player_invincibility.flash_timer = 0.0;
        player_invincibility.flash_state = !player_invincibility.flash_state;

        // Toggle heart color
        let color = if player_invincibility.flash_state {
            damage_settings.normal_color
        } else {
            damage_settings.flash_color
        };

        for mut sprite in player_query.iter_mut() {
            sprite.color = color;
        }
    }
}

#[derive(Clone)]
struct BattleDamageSettings {
    duration: f32,
    flash_interval: f32,
    normal_color: Color,
    flash_color: Color,
    damage_sound: Option<String>,
}

fn battle_damage_settings(db: &bevy_fact_rule_event::LayeredFactDatabase) -> BattleDamageSettings {
    BattleDamageSettings {
        duration: read_f32_fact(db, INVINCIBILITY_DURATION_FACT, 1.0),
        flash_interval: read_f32_fact(db, FLASH_INTERVAL_FACT, 0.1).max(f32::EPSILON),
        normal_color: read_color_fact(db, NORMAL_COLOR_PREFIX, Color::srgb(1.0, 0.0, 0.0)),
        flash_color: read_color_fact(db, FLASH_COLOR_PREFIX, Color::srgb(0.5, 0.0, 0.0)),
        damage_sound: db.get_string(DAMAGE_SOUND_FACT).map(ToString::to_string),
    }
}

fn read_f32_fact(db: &bevy_fact_rule_event::LayeredFactDatabase, key: &str, default: f32) -> f32 {
    db.get_float(key).map_or(default, |value| value as f32)
}

fn read_color_fact(
    db: &bevy_fact_rule_event::LayeredFactDatabase,
    prefix: &str,
    default: Color,
) -> Color {
    let default = default.to_srgba();
    Color::srgba(
        read_f32_fact(db, &format!("{prefix}:red"), default.red),
        read_f32_fact(db, &format!("{prefix}:green"), default.green),
        read_f32_fact(db, &format!("{prefix}:blue"), default.blue),
        read_f32_fact(db, &format!("{prefix}:alpha"), default.alpha),
    )
}

/// Helper function to check collision between two trigger colliders.
///
/// 检查两个触发器碰撞体之间是否发生碰撞的辅助函数。
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
