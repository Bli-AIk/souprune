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

pub use crate::core::danmaku::*;

use crate::app_state::AppState;
use crate::core::collision::TriggerCollider;
use crate::core::danmaku::DanmakuSpawnContext;
use crate::core::mod_system::BehaviorParams;
use bevy::prelude::*;

use super::BattleUpdate;

/// Battle invincibility configuration.
/// Similar to chase config but for battle mode.
///
/// 战斗模式的无敌时间配置。
#[derive(Resource)]
pub struct BattleInvincibilityConfig {
    /// Duration of invincibility in seconds after taking damage
    pub duration: f32,
    /// Interval for heart color flash during invincibility (in seconds)
    pub flash_interval: f32,
    /// Normal heart color (pure red)
    pub normal_color: Color,
    /// Flash heart color (dark red)
    pub flash_color: Color,
    /// Sound to play when taking damage (full path, e.g., "audios/sfx/hurtsound.wav")
    /// If None, no sound is played.
    ///
    /// 受伤时播放的音效（完整路径，如 "audios/sfx/hurtsound.wav"）
    /// 如果为 None，则不播放音效。
    pub damage_sound: Option<String>,
}

impl Default for BattleInvincibilityConfig {
    fn default() -> Self {
        Self {
            duration: 1.0,
            flash_interval: 0.25,
            normal_color: Color::srgb(1.0, 0.0, 0.0), // #FF0000
            flash_color: Color::srgb(0.5, 0.0, 0.0),  // #800000
            damage_sound: Some("audios/sfx/hurtsound.wav".to_string()),
        }
    }
}

/// Resource to track player invincibility state in battle mode.
///
/// 追踪战斗模式下玩家无敌状态的资源。
#[derive(Resource, Default)]
pub struct BattlePlayerInvincibility {
    /// Whether player is currently invincible
    pub active: bool,
    /// Remaining invincibility time
    pub timer: f32,
    /// Flash timer for heart color toggle
    pub flash_timer: f32,
    /// Current flash state (true = normal color, false = flash color)
    pub flash_state: bool,
}

impl BattlePlayerInvincibility {
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
/// Configures CoreDanmakuPlugin for battle state.
///
/// 战斗特定的弹幕插件。
/// 为战斗状态配置 CoreDanmakuPlugin。
pub struct DanmakuPlugin;

impl Plugin for DanmakuPlugin {
    fn build(&self, app: &mut App) {
        // Set spawn context to Battle when entering battle state
        app.add_systems(OnEnter(AppState::Battle), set_battle_context);

        // Add damage detection and invincibility systems
        app.init_resource::<BattleInvincibilityConfig>()
            .init_resource::<BattlePlayerInvincibility>()
            .add_systems(
                Update,
                (
                    battle_damage_detection_system,
                    update_battle_invincibility_system,
                )
                    .chain()
                    .in_set(BattleUpdate),
            );

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
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn battle_damage_detection_system(
    mut commands: Commands,
    time: Res<Time>,
    invincibility_config: Res<BattleInvincibilityConfig>,
    mut player_invincibility: ResMut<BattlePlayerInvincibility>,
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
    #[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))] audio: Res<
        bevy_kira_audio::Audio,
    >,
    asset_server: Res<AssetServer>,
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

    // Check if player is invincible
    let is_invincible = player_invincibility.is_invincible();

    // Debug: Count bullets
    let bullet_count = bullet_query.iter().count();
    if bullet_count > 0 {
        // Only log occasionally to avoid spam (every second or so)
        static mut LAST_LOG_TIME: f64 = 0.0;
        let should_log = unsafe {
            if current_time - LAST_LOG_TIME > 1.0 {
                LAST_LOG_TIME = current_time;
                true
            } else {
                false
            }
        };
        if should_log {
            info!(
                "[Battle Damage] Found {} bullets, player at {:?}",
                bullet_count, player_center
            );
        }
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
            let current_hp = layered_db.get_int("player_hp").unwrap_or(20) as usize;
            let hp_max = layered_db.get_int("player_hp_max").unwrap_or(20) as usize;
            let new_hp = current_hp.saturating_sub(damage);
            layered_db.set_global("player_hp", new_hp as i64);

            // Start player invincibility
            player_invincibility.start(invincibility_config.duration);

            // Play hurt sound from config
            if let Some(sound_path) = &invincibility_config.damage_sound {
                #[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
                crate::core::audio::play_sound_full_path(&audio, &asset_server, sound_path);
                #[cfg(feature = "firewheel")]
                crate::core::audio::play_sound_full_path(&mut commands, &asset_server, sound_path);
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
    invincibility_config: Res<BattleInvincibilityConfig>,
    mut player_invincibility: ResMut<BattlePlayerInvincibility>,
    mut player_query: Query<&mut Sprite, With<BehaviorParams>>,
) {
    if !player_invincibility.active {
        return;
    }

    let delta = time.delta_secs();

    // Update invincibility timer
    player_invincibility.timer -= delta;

    if player_invincibility.timer <= 0.0 {
        // Invincibility ended - reset to normal color
        player_invincibility.active = false;
        player_invincibility.timer = 0.0;
        player_invincibility.flash_state = true;

        // Reset heart color to pure red
        for mut sprite in player_query.iter_mut() {
            sprite.color = invincibility_config.normal_color;
        }

        info!("Battle: Player invincibility ended");
        return;
    }

    // Update flash timer
    player_invincibility.flash_timer += delta;

    if player_invincibility.flash_timer >= invincibility_config.flash_interval {
        player_invincibility.flash_timer = 0.0;
        player_invincibility.flash_state = !player_invincibility.flash_state;

        // Toggle heart color
        let color = if player_invincibility.flash_state {
            invincibility_config.normal_color
        } else {
            invincibility_config.flash_color
        };

        for mut sprite in player_query.iter_mut() {
            sprite.color = color;
        }
    }
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
