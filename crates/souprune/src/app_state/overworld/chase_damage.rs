//! # chase_damage.rs
//!
//! # chase_damage.rs 文件
//!
//! Player hitbox, damage detection, invincibility, and damage UI systems for chase mode.
//!
//! 追逐战模式下的玩家判定框、伤害检测、无敌状态和受伤UI系统。

use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::prelude::*;

use crate::app_state::overworld::character::components::PlayerControlled;
use crate::app_state::{ModeScoped, SequenceSubState};

use super::chase::{
    ChaseConfig, ChaseHeartMarker, ChaseStateName, ChaseTransition, HitboxShapeConfig,
    is_in_chase_state,
};

// ============================================================================
// Player Hitbox and Damage Detection Systems (experimental feature)
// 玩家判定框和伤害检测系统（experimental 特性）
// ============================================================================

/// Marker component for the player's damage hitbox in chase mode.
///
/// 追逐战模式下玩家伤害判定框的标记组件。
#[derive(Component)]
pub struct ChasePlayerHitbox;

/// Resource to track damage UI display timer.
///
/// 追踪受伤UI显示计时器的资源。
#[derive(Resource, Default)]
pub struct DamageUIState {
    /// Whether damage UI is currently showing
    pub showing: bool,
    /// Timer for auto-hide
    pub timer: f32,
}

/// Resource to track player invincibility state.
/// Used for both OW chase mode and Battle mode.
///
/// 追踪玩家无敌状态的资源。
/// 用于 OW 追逐战模式和战斗模式。
#[derive(Resource, Default)]
pub struct PlayerInvincibility {
    /// Whether player is currently invincible
    pub active: bool,
    /// Remaining invincibility time
    pub timer: f32,
    /// Flash timer for heart color toggle
    pub flash_timer: f32,
    /// Current flash state (true = normal color, false = flash color)
    pub flash_state: bool,
}

impl PlayerInvincibility {
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

/// Event fired when player takes damage in chase mode.
///
/// 玩家在追逐战模式下受伤时触发的事件。
#[derive(Clone)]
pub struct ChasePlayerDamageEvent {
    pub damage: f32,
}

impl Message for ChasePlayerDamageEvent {}

/// System to add TriggerCollider to player when entering chase state.
///
/// 进入追逐战状态时为玩家添加 TriggerCollider 的系统。
pub fn spawn_player_hitbox_system(
    mut commands: Commands,
    chase_config: Res<ChaseConfig>,
    transition: Res<ChaseTransition>,
    player_query: Query<Entity, (With<PlayerControlled>, Without<ChasePlayerHitbox>)>,
) {
    // Only spawn when transitioning in
    if !transition.transitioning_in {
        return;
    }

    let Ok(player_entity) = player_query.single() else {
        return;
    };

    // Create TriggerCollider based on hitbox config
    let trigger_collider = match &chase_config.hitbox.shape {
        HitboxShapeConfig::Circle { radius } => {
            crate::core::collision::TriggerCollider::Circle { radius: *radius }
        }
        HitboxShapeConfig::Box {
            half_width,
            half_height,
        } => crate::core::collision::TriggerCollider::Box {
            half_size: Vec2::new(*half_width, *half_height),
        },
    };

    commands.entity(player_entity).insert((
        ChasePlayerHitbox,
        trigger_collider,
        crate::core::collision::HitboxOffset(chase_config.hitbox.offset.to_vec2()),
    ));

    info!(
        "Chase: Added player hitbox with shape {:?}",
        chase_config.hitbox.shape
    );
}

/// System to remove TriggerCollider from player when exiting chase state.
///
/// 退出追逐战状态时从玩家移除 TriggerCollider 的系统。
pub fn cleanup_player_hitbox_system(
    mut commands: Commands,
    transition: Res<ChaseTransition>,
    player_query: Query<Entity, With<ChasePlayerHitbox>>,
) {
    // Only cleanup when transition out is complete
    if transition.active || transition.transitioning_in || transition.timer > 0.0 {
        return;
    }

    for player_entity in player_query.iter() {
        commands
            .entity(player_entity)
            .remove::<ChasePlayerHitbox>()
            .remove::<crate::core::collision::TriggerCollider>()
            .remove::<crate::core::collision::HitboxOffset>();
        info!("Chase: Removed player hitbox");
    }
}

/// System to detect bullet collision with player hitbox in chase mode.
///
/// 检测追逐战模式下弹幕与玩家判定框碰撞的系统。
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn chase_damage_detection_system(
    mut commands: Commands,
    time: Res<Time>,
    chase_config: Res<ChaseConfig>,
    player_behavior: Res<crate::app_state::overworld::player::config::PlayerBehavior>,
    sub_state: Res<State<SequenceSubState>>,
    chase_state_name: Res<ChaseStateName>,
    asset_server: Res<AssetServer>,
    mut player_invincibility: ResMut<PlayerInvincibility>,
    mut layered_db: ResMut<bevy_fact_rule_event::LayeredFactDatabase>,
    audio: Res<bevy_kira_audio::Audio>,
    player_query: Query<
        (
            &Transform,
            &crate::core::collision::TriggerCollider,
            Option<&crate::core::collision::HitboxOffset>,
        ),
        With<ChasePlayerHitbox>,
    >,
    mut bullet_query: Query<
        (
            Entity,
            &GlobalTransform,
            &crate::core::collision::TriggerCollider,
            &crate::core::danmaku::BulletDamage,
            &crate::core::danmaku::BulletHitBehavior,
            &mut crate::core::danmaku::BulletLastHitTime,
            &crate::core::danmaku::BulletMotionState,
        ),
        With<crate::core::danmaku::Bullet>,
    >,
    mut damage_events: MessageWriter<ChasePlayerDamageEvent>,
    mut last_player_state: Local<Option<(Vec2, f64)>>,
) {
    // Only run in chase state
    if !is_in_chase_state(&sub_state, &chase_state_name) {
        *last_player_state = None; // Reset when not in chase
        return;
    }

    let Ok((player_transform, player_hitbox, hitbox_offset)) = player_query.single() else {
        return;
    };

    let player_center = player_transform.translation.truncate()
        + hitbox_offset
            .map(|o| o.0)
            .unwrap_or(chase_config.hitbox.offset.to_vec2());
    let current_time = time.elapsed_secs_f64();

    // Check if player is moving based on position change
    let player_is_moving = if let Some((last_pos, last_time)) = *last_player_state {
        // If too much time passed (e.g. paused, lag spike), reset detection
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
        if !check_trigger_collision(player_hitbox, player_center, bullet_collider, bullet_center) {
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

            // Fire damage event
            damage_events.write(ChasePlayerDamageEvent {
                damage: bullet_damage.0,
            });

            // Start player invincibility
            player_invincibility.start(player_behavior.invincibility.duration);

            // Play hurt sound from config
            if let Some(sound_path) = &chase_config.damage_ui.damage_sound {
                crate::core::audio::play_sound_full_path(&audio, &asset_server, sound_path);
            }

            info!(
                "Chase: Player hit! Damage: {}, HP: {}/{}",
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

/// Helper function to check collision between two trigger colliders.
///
/// 检查两个触发器碰撞体之间是否发生碰撞的辅助函数。
fn check_trigger_collision(
    a: &crate::core::collision::TriggerCollider,
    a_center: Vec2,
    b: &crate::core::collision::TriggerCollider,
    b_center: Vec2,
) -> bool {
    use crate::core::collision::TriggerCollider;

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

/// System to update player invincibility timer and heart flashing effect.
///
/// 更新玩家无敌时间和心形闪烁效果的系统。
pub fn update_player_invincibility_system(
    time: Res<Time>,
    chase_config: Res<ChaseConfig>,
    player_behavior: Res<crate::app_state::overworld::player::config::PlayerBehavior>,
    sub_state: Res<State<SequenceSubState>>,
    chase_state_name: Res<ChaseStateName>,
    mut player_invincibility: ResMut<PlayerInvincibility>,
    mut heart_markers: Query<&mut Sprite, With<ChaseHeartMarker>>,
) {
    // Only run in chase state
    if !is_in_chase_state(&sub_state, &chase_state_name) {
        return;
    }

    if !player_invincibility.active {
        return;
    }

    let delta = time.delta_secs();
    let config = &player_behavior.invincibility;

    // Update invincibility timer
    player_invincibility.timer -= delta;

    if player_invincibility.timer <= 0.0 {
        // Invincibility ended - reset to normal color
        player_invincibility.active = false;
        player_invincibility.timer = 0.0;
        player_invincibility.flash_state = true;

        // Reset heart color to pure red
        for mut sprite in heart_markers.iter_mut() {
            let heart_config = &chase_config.heart_marker;
            sprite.color = Color::srgba(
                heart_config.color.r,
                heart_config.color.g,
                heart_config.color.b,
                heart_config.color.a,
            );
        }

        info!("Chase: Player invincibility ended");
        return;
    }

    // Update flash timer
    player_invincibility.flash_timer += delta;

    if player_invincibility.flash_timer >= config.flash_interval {
        player_invincibility.flash_timer = 0.0;
        player_invincibility.flash_state = !player_invincibility.flash_state;

        // Toggle heart color
        let color = if player_invincibility.flash_state {
            // Normal color (pure red)
            parse_hex_color_for_heart(&config.normal_color)
        } else {
            // Flash color (dark red)
            parse_hex_color_for_heart(&config.flash_color)
        };

        for mut sprite in heart_markers.iter_mut() {
            if let Some(c) = color {
                sprite.color = c;
            }
        }
    }
}

/// Parse hex color string for heart color.
fn parse_hex_color_for_heart(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::srgb_u8(r, g, b))
        }
        _ => None,
    }
}

/// System to handle damage UI display.
/// Now simplified since HP bar HUD provides primary damage feedback.
/// This only handles a brief screen tint effect on damage.
///
/// 处理受伤UI显示的系统。
/// 由于血条 HUD 提供了主要的受伤反馈，此系统已简化。
/// 仅处理受伤时的短暂屏幕色调效果。
pub fn damage_ui_display_system(
    mut commands: Commands,
    time: Res<Time>,
    mut damage_ui_state: ResMut<DamageUIState>,
    mut damage_events: MessageReader<ChasePlayerDamageEvent>,
    damage_ui_query: Query<Entity, With<DamageUIMarker>>,
    camera_query: Query<&Transform, With<Camera2d>>,
) {
    // Check for new damage events
    for _event in damage_events.read() {
        if !damage_ui_state.showing {
            // Get camera position to center the overlay
            let camera_pos = camera_query
                .single()
                .map(|t| t.translation)
                .unwrap_or(Vec3::ZERO);

            // Spawn a simple red overlay sprite without requiring a texture
            // The HP bar HUD now provides the primary damage feedback
            commands.spawn((
                Sprite {
                    color: Color::srgba(1.0, 0.0, 0.0, 0.3),
                    custom_size: Some(Vec2::new(640.0, 480.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(camera_pos.x, camera_pos.y, 500.0)),
                ModeScoped("overworld".to_string()),
                DamageUIMarker,
                Name::new("DamageFlashOverlay"),
            ));
            damage_ui_state.showing = true;
            damage_ui_state.timer = 0.1; // Short flash duration
            info!("Chase: Showing damage flash overlay");
        } else {
            // Reset timer if already showing
            damage_ui_state.timer = 0.1;
        }
    }

    // Update timer and hide UI when done
    if damage_ui_state.showing {
        damage_ui_state.timer -= time.delta_secs();
        if damage_ui_state.timer <= 0.0 {
            // Despawn damage UI
            for entity in damage_ui_query.iter() {
                commands.entity(entity).despawn();
            }
            damage_ui_state.showing = false;
            info!("Chase: Hiding damage flash overlay");
        }
    }
}

/// Marker component for damage UI entities.
///
/// 受伤UI实体的标记组件。
#[derive(Component)]
pub struct DamageUIMarker;

/// Marker component for Chase HUD UI root entity.
///
/// 追逐战 HUD UI 根实体的标记组件。
#[derive(Component)]
pub struct ChaseHUDRoot;

/// System to setup Chase HUD when entering chase state.
/// Loads the View layout from chase_config.damage_ui.layout_path.
///
/// 进入追逐战状态时设置 HUD 的系统。
/// 从 chase_config.damage_ui.layout_path 加载视图布局。
pub(super) fn setup_chase_hud_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    chase_config: Res<ChaseConfig>,
) {
    info!("Chase: Setting up Chase HUD");

    // Load the chase HUD View layout from config / 从配置加载追逐战 HUD 视图布局
    let ui_path = &chase_config.damage_ui.layout_path;
    let handle = asset_server.load(ui_path.clone());

    // Insert the View layout handle resource
    commands.insert_resource(crate::core::view::ViewLayoutHandle {
        handle,
        last_modified: None,
        path: ui_path.to_string(),
    });

    // Spawn a root entity for the View system to attach to
    // 生成一个根实体供 View 系统附加
    commands.spawn((
        ChaseHUDRoot,
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        ModeScoped("overworld".to_string()),
        Name::new("ChaseHUD Root"),
    ));

    info!("Chase: Chase HUD setup complete, layout: {}", ui_path);
}

/// System to cleanup Chase HUD when exiting chase state.
///
/// 退出追逐战状态时清理 HUD 的系统。
pub(super) fn cleanup_chase_hud_system(
    mut commands: Commands,
    chase_hud_query: Query<Entity, With<ChaseHUDRoot>>,
    ron_driven_ui_query: Query<Entity, With<crate::core::view::RonDrivenView>>,
) {
    info!("Chase: Cleaning up Chase HUD");

    // Despawn the Chase HUD root and all RON-driven UI entities
    for entity in chase_hud_query.iter() {
        commands.entity(entity).despawn();
    }

    // Also despawn any RON-driven UI entities that may have been spawned
    for entity in ron_driven_ui_query.iter() {
        commands.entity(entity).despawn();
    }

    // Remove the View layout handle resource
    commands.remove_resource::<crate::core::view::ViewLayoutHandle>();

    info!("Chase: Chase HUD cleanup complete");
}
