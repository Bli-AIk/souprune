//! # systems.rs
//!
//! ## Module Overview
//!
//! Implements runtime systems for the data-driven danmaku system.
//!
//! 实现数据驱动弹幕系统的运行时系统。

use super::components::*;
use super::patterns::*;
use crate::app_state::battle::BattleEntity;
use crate::core::animation::components::{SpriteAnimationClip, SpriteAnimationTimer};
use crate::core::mod_system::BehaviorParams;
use crate::core::sprite::params::SpriteParams;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

/// System to process spawn pattern events and queue blueprint loads.
///
/// 处理生成弹幕模式事件并排队加载蓝图。
pub fn process_spawn_pattern_events(
    mut events: MessageReader<SpawnPatternEvent>,
    mut pending: ResMut<PendingBlueprintLoads>,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<BehaviorParams>>,
) {
    for event in events.read() {
        // Get player position as default center
        let center = player_query
            .iter()
            .next()
            .map(|t| t.translation.truncate())
            .unwrap_or(event.position);

        let mut event_with_pos = event.clone();
        if event.position == Vec2::ZERO {
            event_with_pos.position = center;
        }

        let handle = asset_server.load::<DanmakuBlueprint>(&event.blueprint_path);
        pending.pending.push((handle, event_with_pos));

        info!("Queued blueprint load: {}", event.blueprint_path);
    }
}

/// System to spawn bullets when blueprints are loaded.
///
/// 当蓝图加载完成时生成弹幕。
pub fn spawn_bullets_from_blueprints(
    mut commands: Commands,
    mut pending: ResMut<PendingBlueprintLoads>,
    blueprints: Res<Assets<DanmakuBlueprint>>,
    mut sprite_params: SpriteParams,
    asset_server: Res<AssetServer>,
) {
    let mut still_pending = Vec::new();

    for (handle, event) in pending.pending.drain(..) {
        if let Some(blueprint) = blueprints.get(&handle) {
            spawn_pattern_from_blueprint(
                &mut commands,
                blueprint,
                &event,
                &mut sprite_params,
                &asset_server,
            );
            info!("Spawned pattern from blueprint: {}", event.blueprint_path);
        } else {
            // Still loading
            still_pending.push((handle, event));
        }
    }

    pending.pending = still_pending;
}

/// Spawn bullets based on blueprint configuration.
fn spawn_pattern_from_blueprint(
    commands: &mut Commands,
    blueprint: &DanmakuBlueprint,
    event: &SpawnPatternEvent,
    sprite_params: &mut SpriteParams,
    asset_server: &AssetServer,
) {
    let spawn_positions = calculate_spawn_positions(blueprint, event);

    for (i, (pos, angle, radius)) in spawn_positions.into_iter().enumerate() {
        spawn_single_bullet(
            commands,
            blueprint,
            pos,
            angle,
            radius,
            event.position,
            i,
            sprite_params,
            asset_server,
        );
    }
}

/// Calculate spawn positions based on SpawnPattern configuration.
fn calculate_spawn_positions(
    blueprint: &DanmakuBlueprint,
    event: &SpawnPatternEvent,
) -> Vec<(Vec2, f32, f32)> {
    let center = event.position;

    match &blueprint.spawn_pattern {
        SpawnPattern::Single => {
            vec![(center, 0.0, 0.0)]
        }
        SpawnPattern::Circle {
            count,
            radius,
            start_angle,
        } => {
            let count = event.count.unwrap_or(*count);
            let angle_step = std::f32::consts::TAU / count as f32;

            (0..count)
                .map(|i| {
                    let angle = start_angle + angle_step * i as f32;
                    let pos = center + Vec2::new(angle.cos(), angle.sin()) * *radius;
                    (pos, angle, *radius)
                })
                .collect()
        }
        SpawnPattern::Line {
            count,
            spacing,
            direction,
        } => {
            let count = event.count.unwrap_or(*count);
            let dir = Vec2::new(direction.0, direction.1).normalize_or_zero();
            let perp = Vec2::new(-dir.y, dir.x);
            let total_width = *spacing * (count - 1) as f32;
            let start_offset = -total_width / 2.0;

            (0..count)
                .map(|i| {
                    let offset = start_offset + *spacing * i as f32;
                    let pos = center + perp * offset;
                    let angle = dir.y.atan2(dir.x);
                    (pos, angle, 0.0)
                })
                .collect()
        }
        SpawnPattern::Edge {
            count,
            side,
            spacing,
            margin,
        } => {
            let count = event.count.unwrap_or(*count);
            let move_dir = side.to_direction();
            let start_offset = side.to_offset(*margin);
            let perp = Vec2::new(-move_dir.y, move_dir.x);
            let total_width = *spacing * (count - 1) as f32;
            let start_perp_offset = -total_width / 2.0;

            (0..count)
                .map(|i| {
                    let perp_offset = start_perp_offset + *spacing * i as f32;
                    let pos = center + start_offset + perp * perp_offset;
                    let angle = move_dir.y.atan2(move_dir.x);
                    (pos, angle, 0.0)
                })
                .collect()
        }
    }
}

/// Spawn a single bullet entity.
fn spawn_single_bullet(
    commands: &mut Commands,
    blueprint: &DanmakuBlueprint,
    position: Vec2,
    angle: f32,
    radius: f32,
    spawn_center: Vec2,
    index: usize,
    sprite_params: &mut SpriteParams,
    asset_server: &AssetServer,
) {
    let mut entity_commands = commands.spawn((
        Bullet,
        Transform::from_translation(position.extend(blueprint.z_index)),
        BulletLifetime::new(blueprint.lifetime),
        BulletDamage(blueprint.damage),
        BulletMotionState::new(spawn_center)
            .with_offset(position - spawn_center)
            .with_angle(angle)
            .with_radius(radius),
        BulletMotionTracks(blueprint.motion_tracks.clone()),
        BattleEntity(),
        Name::new(format!("Bullet_{}", index)),
    ));

    // Add visual component based on blueprint
    match &blueprint.visual {
        BulletVisual::Sprite { path } => {
            entity_commands.insert(Sprite {
                image: asset_server.load(path),
                ..default()
            });

            // Apply rotation for edge-spawned bullets
            if let SpawnPattern::Edge { side, .. } = &blueprint.spawn_pattern {
                entity_commands.insert(
                    Transform::from_translation(position.extend(blueprint.z_index))
                        .with_rotation(Quat::from_rotation_z(side.rotation_angle())),
                );
            }
        }
        BulletVisual::Animation {
            module,
            name,
            frame_duration,
        } => {
            let mut sprite_context = sprite_params.create_sprite_context();
            match SpriteAnimationClip::new(&mut sprite_context, module, name) {
                Ok(clip) => {
                    entity_commands.insert((
                        Sprite::default(),
                        clip,
                        SpriteAnimationTimer::new(*frame_duration),
                    ));
                }
                Err(e) => {
                    warn!("Failed to create animation '{}': {}", name, e);
                    // Fallback to default sprite
                    entity_commands.insert(Sprite::default());
                }
            }
        }
    }
}

/// System to update bullet motion based on their motion tracks.
///
/// 根据运动轨道更新弹幕运动的系统。
pub fn update_bullet_motion(
    time: Res<Time>,
    player_query: Query<&Transform, (With<BehaviorParams>, Without<Bullet>)>,
    mut query: Query<(&mut Transform, &mut BulletMotionState, &BulletMotionTracks), With<Bullet>>,
) {
    let dt = time.delta_secs();
    let player_pos = player_query
        .iter()
        .next()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (mut transform, mut state, tracks) in query.iter_mut() {
        state.elapsed += dt;

        // Calculate position from motion stack
        let mut position = state.spawn_center + state.initial_offset;
        let mut rotation_delta = 0.0;

        for track in &tracks.0 {
            match track {
                MotionTrack::Linear { direction, speed } => {
                    let dir = Vec2::new(direction.0, direction.1).normalize_or_zero();
                    position += dir * *speed * state.elapsed;
                }
                MotionTrack::Circular {
                    angular_velocity,
                    radial_velocity,
                } => {
                    let current_angle = state.initial_angle + angular_velocity * state.elapsed;
                    let current_radius = state.initial_radius + radial_velocity * state.elapsed;
                    let current_radius = current_radius.max(0.0);

                    position = state.spawn_center
                        + Vec2::new(current_angle.cos(), current_angle.sin()) * current_radius;
                    rotation_delta += angular_velocity * dt;
                }
                MotionTrack::Sine {
                    axis,
                    amplitude,
                    frequency,
                    phase,
                } => {
                    let axis_vec = Vec2::new(axis.0, axis.1).normalize_or_zero();
                    let wave = (state.elapsed * frequency * std::f32::consts::TAU + phase).sin();
                    position += axis_vec * wave * *amplitude;
                }
                MotionTrack::Homing {
                    strength,
                    max_turn_rate,
                } => {
                    let to_player = (player_pos - position).normalize_or_zero();
                    let current_dir = state.velocity_direction;

                    // Gradually turn towards player
                    let turn_amount = (strength * dt).min(*max_turn_rate * dt);
                    let new_dir = current_dir.lerp(to_player, turn_amount).normalize_or_zero();
                    state.velocity_direction = new_dir;

                    // Homing typically needs a base speed, apply it
                    position += new_dir * 100.0 * dt; // Default homing speed
                }
                MotionTrack::Keyframed {
                    target,
                    keyframes,
                    loop_mode,
                } => {
                    if let Some(value) = evaluate_keyframes(keyframes, state.elapsed, *loop_mode) {
                        match target {
                            PropertyTarget::Position => {
                                if let KeyframeValue::Vec2(x, y) = value {
                                    position += Vec2::new(x, y);
                                }
                            }
                            PropertyTarget::PositionX => {
                                if let KeyframeValue::Float(x) = value {
                                    position.x += x;
                                }
                            }
                            PropertyTarget::PositionY => {
                                if let KeyframeValue::Float(y) = value {
                                    position.y += y;
                                }
                            }
                            PropertyTarget::Rotation => {
                                if let KeyframeValue::Float(r) = value {
                                    rotation_delta += r;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                MotionTrack::Algorithmic { .. } => {
                    // TODO: Implement algorithm registry for custom motion
                }
            }
        }

        transform.translation.x = position.x;
        transform.translation.y = position.y;

        if rotation_delta != 0.0 {
            transform.rotate_z(rotation_delta);
        }
    }
}

/// Evaluate keyframes at a given time.
fn evaluate_keyframes(
    keyframes: &[Keyframe],
    time: f32,
    loop_mode: LoopMode,
) -> Option<KeyframeValue> {
    if keyframes.is_empty() {
        return None;
    }

    let duration = keyframes.last()?.t;
    if duration <= 0.0 {
        return Some(keyframes[0].value.clone());
    }

    // Apply loop mode
    let effective_time = match loop_mode {
        LoopMode::Once => time.min(duration),
        LoopMode::Loop => time % duration,
        LoopMode::PingPong => {
            let cycle = (time / duration) as i32;
            let t = time % duration;
            if cycle % 2 == 0 { t } else { duration - t }
        }
    };

    // Find keyframe pair
    let mut prev_kf = &keyframes[0];
    for kf in keyframes {
        if kf.t > effective_time {
            let local_t = if kf.t == prev_kf.t {
                0.0
            } else {
                (effective_time - prev_kf.t) / (kf.t - prev_kf.t)
            };
            let eased_t = kf.ease.apply(local_t);

            return Some(interpolate_keyframe_values(
                &prev_kf.value,
                &kf.value,
                eased_t,
            ));
        }
        prev_kf = kf;
    }

    Some(keyframes.last()?.value.clone())
}

/// Interpolate between two keyframe values.
fn interpolate_keyframe_values(a: &KeyframeValue, b: &KeyframeValue, t: f32) -> KeyframeValue {
    match (a, b) {
        (KeyframeValue::Float(a), KeyframeValue::Float(b)) => KeyframeValue::Float(a + (b - a) * t),
        (KeyframeValue::Vec2(ax, ay), KeyframeValue::Vec2(bx, by)) => {
            KeyframeValue::Vec2(ax + (bx - ax) * t, ay + (by - ay) * t)
        }
        _ => a.clone(),
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
