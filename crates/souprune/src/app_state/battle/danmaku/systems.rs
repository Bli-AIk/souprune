//! # systems.rs
//!
//! ## Module Overview
//!
//! Implements runtime systems for the danmaku system.
//!
//! 实现弹幕系统的运行时系统。

use super::components::*;
use super::patterns::*;
use crate::app_state::battle::BattleEntity;
use crate::core::animation::components::{SpriteAnimationClip, SpriteAnimationTimer};
use crate::core::mod_system::BehaviorParams;
use crate::core::sprite::params::SpriteParams;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

/// System to process spawn pattern events and create bullets.
///
/// 处理生成弹幕模式事件并创建弹幕的系统。
pub fn process_spawn_pattern_events(
    mut commands: Commands,
    mut events: MessageReader<SpawnPatternEvent>,
    registry: Res<PatternRegistry>,
    mut sprite_params: SpriteParams,
    player_query: Query<&Transform, With<BehaviorParams>>,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        let Some(pattern) = registry.get(&event.pattern_id) else {
            warn!("Pattern not found: {}", event.pattern_id);
            continue;
        };

        // Get player position as center for patterns that need it
        let player_pos = player_query
            .iter()
            .next()
            .map(|t| t.translation.truncate())
            .unwrap_or(event.position);

        match pattern.clone() {
            PatternType::FloweyPelletsCircle {
                count,
                radius,
                converge_speed,
                lifetime,
            } => {
                spawn_flowey_pellets_circle(
                    &mut commands,
                    &mut sprite_params,
                    player_pos,
                    count,
                    radius,
                    converge_speed,
                    lifetime,
                );
            }
            PatternType::UndyneSpearSweep {
                count,
                direction,
                speed,
                spacing,
                lifetime,
            } => {
                spawn_undyne_spear_sweep(
                    &mut commands,
                    &asset_server,
                    player_pos,
                    count,
                    direction,
                    speed,
                    spacing,
                    lifetime,
                );
            }
        }

        info!("Spawned pattern: {}", event.pattern_id);
    }
}

/// Spawns the Flowey pellet circle pattern.
fn spawn_flowey_pellets_circle(
    commands: &mut Commands,
    sprite_params: &mut SpriteParams,
    center: Vec2,
    count: usize,
    radius: f32,
    converge_speed: f32,
    lifetime: f32,
) {
    let angle_step = std::f32::consts::TAU / count as f32;

    for i in 0..count {
        let angle = angle_step * i as f32;

        // Create animation clip for flowey pellet
        let mut sprite_context = sprite_params.create_sprite_context();
        let clip = match SpriteAnimationClip::new(&mut sprite_context, "battle", "flowey_pellet") {
            Ok(clip) => clip,
            Err(e) => {
                warn!("Failed to create flowey_pellet animation: {}", e);
                continue;
            }
        };

        let circular_motion =
            CircularMotion::new(center, radius, angle, 0.5).with_radial_velocity(-converge_speed);

        // Calculate initial position
        let initial_pos = center + Vec2::new(angle.cos(), angle.sin()) * radius;

        commands.spawn((
            Bullet,
            Sprite::default(),
            clip,
            SpriteAnimationTimer::new(0.05),
            Transform::from_translation(initial_pos.extend(5.0)),
            circular_motion,
            BulletLifetime::new(lifetime),
            BulletDamage::default(),
            BattleEntity(),
            Name::new(format!("FloweyPellet_{}", i)),
        ));
    }
}

/// Spawns the Undyne spear sweep pattern.
fn spawn_undyne_spear_sweep(
    commands: &mut Commands,
    asset_server: &AssetServer,
    center: Vec2,
    count: usize,
    direction: SpearDirection,
    speed: f32,
    spacing: f32,
    lifetime: f32,
) {
    let screen_margin = 200.0;
    let start_offset = direction.start_offset(screen_margin);
    let move_direction = direction.to_vec2();
    let rotation = direction.rotation_angle();

    // Calculate perpendicular direction for spacing
    let perp = Vec2::new(-move_direction.y, move_direction.x);

    // Calculate total width to center the pattern
    let total_width = spacing * (count - 1) as f32;
    let start_offset_perp = -total_width / 2.0;

    for i in 0..count {
        let perp_offset = start_offset_perp + spacing * i as f32;
        let spawn_pos = center + start_offset + perp * perp_offset;
        let end_pos = center - start_offset + perp * perp_offset;

        let duration = (end_pos - spawn_pos).length() / speed;

        commands.spawn((
            Bullet,
            Sprite {
                image: asset_server.load("textures/battle/bullets/spear/spear.png"),
                ..default()
            },
            Transform::from_translation(spawn_pos.extend(5.0))
                .with_rotation(Quat::from_rotation_z(rotation)),
            SweepMotion::new(spawn_pos, end_pos, duration),
            BulletLifetime::new(lifetime),
            BulletDamage(2.0),
            BattleEntity(),
            Name::new(format!("UndyneSpear_{}", i)),
        ));
    }
}

/// System to update bullet motion based on their motion components.
///
/// 根据运动组件更新弹幕运动的系统。
pub fn update_bullet_motion(
    time: Res<Time>,
    mut circular_query: Query<(&mut Transform, &mut CircularMotion), With<Bullet>>,
    mut linear_query: Query<
        (&mut Transform, &LinearMotion),
        (With<Bullet>, Without<CircularMotion>),
    >,
    mut sweep_query: Query<
        (&mut Transform, &mut SweepMotion),
        (With<Bullet>, Without<CircularMotion>, Without<LinearMotion>),
    >,
) {
    let dt = time.delta_secs();

    // Update circular motion bullets
    for (mut transform, mut motion) in circular_query.iter_mut() {
        motion.current_angle += motion.angular_velocity * dt;
        motion.radius += motion.radial_velocity * dt;

        // Clamp radius to prevent negative values
        motion.radius = motion.radius.max(0.0);

        let new_pos = motion.center
            + Vec2::new(motion.current_angle.cos(), motion.current_angle.sin()) * motion.radius;

        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;
    }

    // Update linear motion bullets
    for (mut transform, motion) in linear_query.iter_mut() {
        let velocity = motion.direction * motion.speed;
        transform.translation.x += velocity.x * dt;
        transform.translation.y += velocity.y * dt;
    }

    // Update sweep motion bullets
    for (mut transform, mut motion) in sweep_query.iter_mut() {
        motion.elapsed += dt;
        let t = motion.progress();

        // Use easing for smoother motion
        let eased_t = ease_in_out_quad(t);

        let new_pos = motion.start_pos.lerp(motion.end_pos, eased_t);
        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;
    }
}

/// Quadratic ease in-out function.
fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

/// System to update bullet lifetime and mark dead bullets.
///
/// 更新弹幕生命周期并标记死亡弹幕的系统。
pub fn update_bullet_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BulletLifetime), With<Bullet>>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.timer.tick(time.delta());

        if lifetime.timer.is_finished() {
            commands.entity(entity).insert(DespawnBullet);
        }
    }
}

/// System to cleanup bullets marked for despawn.
///
/// 清理标记为销毁的弹幕的系统。
pub fn cleanup_dead_bullets(
    mut commands: Commands,
    query: Query<Entity, (With<Bullet>, With<DespawnBullet>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
