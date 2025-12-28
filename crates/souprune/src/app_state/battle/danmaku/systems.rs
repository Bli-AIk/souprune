//! # systems.rs
//!
//! ## Module Overview
//!
//! Implements runtime systems for the timeline-based danmaku system.
//!
//! 实现基于时间轴的弹幕系统的运行时系统。

use super::components::*;
use super::patterns::*;
use crate::app_state::battle::BattleEntity;
use crate::core::animation::components::{SpriteAnimationClip, SpriteAnimationTimer};
use crate::core::mod_system::{BehaviorParams, DanmakuRegistry};
use crate::core::sprite::params::SpriteParams;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use souprune_api::{BulletStateC, Vec2C};

// ============================================================================
// Performance System: Event Processing and Asset Loading
// ============================================================================

/// System to process play performance events and queue asset loads.
///
/// 处理播放演出事件并排队加载资产。
pub fn process_play_performance_events(
    mut events: MessageReader<PlayPerformanceEvent>,
    mut pending: ResMut<PendingPerformanceLoads>,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        let handle = asset_server.load::<DanmakuPerformance>(&event.performance_path);
        pending.pending.push((handle, event.clone()));
        info!("Queued performance load: {}", event.performance_path);
    }
}

/// System to spawn performance players when assets are loaded.
///
/// 当资产加载完成时生成演出播放器。
pub fn spawn_performance_players(
    mut commands: Commands,
    mut pending: ResMut<PendingPerformanceLoads>,
    performances: Res<Assets<DanmakuPerformance>>,
) {
    let mut still_pending = Vec::new();

    for (handle, event) in pending.pending.drain(..) {
        if performances.get(&handle).is_some() {
            commands.spawn((
                PerformancePlayer::new(event.position),
                PerformanceHandle(handle.clone()),
                PerformancePlayerMarker,
                Name::new("PerformancePlayer"),
            ));
            info!("Started performance: {}", event.performance_path);
        } else {
            still_pending.push((handle, event));
        }
    }

    pending.pending = still_pending;
}

// ============================================================================
// Performance System: Timeline Execution
// ============================================================================

/// System to advance performance timeline and spawn bullets.
///
/// 推进演出时间轴并生成弹幕。
pub fn advance_performance_timeline(
    mut commands: Commands,
    time: Res<Time>,
    performances: Res<Assets<DanmakuPerformance>>,
    mut query: Query<(Entity, &mut PerformancePlayer, &PerformanceHandle)>,
    mut sprite_params: SpriteParams,
    asset_server: Res<AssetServer>,
) {
    let dt = time.delta_secs();

    for (entity, mut player, perf_handle) in query.iter_mut() {
        if player.finished {
            continue;
        }

        player.elapsed += dt;

        let Some(performance) = performances.get(&perf_handle.0) else {
            continue;
        };

        // Process timeline events that should fire
        while player.next_event_index < performance.timeline.len() {
            let event = &performance.timeline[player.next_event_index];

            if event.t > player.elapsed {
                break;
            }

            // Fire this event
            spawn_bullets_from_timeline_event(
                &mut commands,
                performance,
                event,
                player.spawn_center,
                &mut sprite_params,
                &asset_server,
            );

            player.next_event_index += 1;
        }

        // Check if performance is finished
        if player.next_event_index >= performance.timeline.len() {
            player.finished = true;
            commands.entity(entity).despawn();
        }
    }
}

/// Spawn bullets from a timeline event.
fn spawn_bullets_from_timeline_event(
    commands: &mut Commands,
    performance: &DanmakuPerformance,
    event: &TimelineEvent,
    spawn_center: Vec2,
    sprite_params: &mut SpriteParams,
    asset_server: &AssetServer,
) {
    let Some(prototype) = performance.prototypes.get(&event.spawn) else {
        warn!("Prototype not found: {}", event.spawn);
        return;
    };

    let behaviors: Vec<BulletBehavior> = event
        .apply
        .iter()
        .filter_map(|id| performance.behaviors.get(id).cloned())
        .collect();

    let spawn_positions = calculate_spawn_positions(&event.pattern, spawn_center);

    for (i, (pos, angle, radius)) in spawn_positions.into_iter().enumerate() {
        spawn_single_bullet(
            commands,
            prototype,
            &behaviors,
            pos,
            angle,
            radius,
            spawn_center,
            i,
            sprite_params,
            asset_server,
        );
    }
}

/// Calculate spawn positions based on SpawnPattern configuration.
fn calculate_spawn_positions(pattern: &SpawnPattern, center: Vec2) -> Vec<(Vec2, f32, f32)> {
    match pattern {
        SpawnPattern::Single => {
            vec![(center, 0.0, 0.0)]
        }
        SpawnPattern::Ring {
            count,
            radius,
            start_angle,
        } => {
            let angle_step = std::f32::consts::TAU / *count as f32;
            (0..*count)
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
            let dir = Vec2::new(direction.0, direction.1).normalize_or_zero();
            let perp = Vec2::new(-dir.y, dir.x);
            let total_width = *spacing * (*count - 1) as f32;
            let start_offset = -total_width / 2.0;

            (0..*count)
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
            let move_dir = side.to_direction();
            let start_offset = side.to_offset(*margin);
            let perp = Vec2::new(-move_dir.y, move_dir.x);
            let total_width = *spacing * (*count - 1) as f32;
            let start_perp_offset = -total_width / 2.0;

            (0..*count)
                .map(|i| {
                    let perp_offset = start_perp_offset + *spacing * i as f32;
                    let pos = center + start_offset + perp * perp_offset;
                    let angle = move_dir.y.atan2(move_dir.x);
                    (pos, angle, 0.0)
                })
                .collect()
        }
        SpawnPattern::Custom { id, .. } => {
            warn!("Custom spawn pattern '{}' not yet implemented", id);
            vec![(center, 0.0, 0.0)]
        }
    }
}

/// Spawn a single bullet entity with BehaviorStack.
fn spawn_single_bullet(
    commands: &mut Commands,
    prototype: &BulletPrototype,
    behaviors: &[BulletBehavior],
    position: Vec2,
    angle: f32,
    radius: f32,
    spawn_center: Vec2,
    index: usize,
    sprite_params: &mut SpriteParams,
    asset_server: &AssetServer,
) {
    let cached_params: Vec<f32> = behaviors
        .iter()
        .filter_map(|b| {
            if let BulletBehavior::Algo { params, .. } = b {
                Some(params.clone())
            } else {
                None
            }
        })
        .flatten()
        .collect();

    let mut entity_commands = commands.spawn((
        Bullet,
        Transform::from_translation(position.extend(prototype.z_index)),
        BulletLifetime::new(prototype.lifetime),
        BulletDamage(prototype.damage),
        BulletMotionState::new(spawn_center)
            .with_offset(position - spawn_center)
            .with_angle(angle)
            .with_radius(radius),
        BehaviorStack::new(behaviors.to_vec()).with_cached_params(cached_params),
        TweenState::default(),
        BattleEntity(),
        Name::new(format!("Bullet_{}", index)),
    ));

    match &prototype.visual {
        BulletVisual::Sprite { path } => {
            entity_commands.insert(Sprite {
                image: asset_server.load(path),
                ..default()
            });
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
                    entity_commands.insert(Sprite::default());
                }
            }
        }
    }
}

// ============================================================================
// Bullet Motion System
// ============================================================================

/// System to update bullet motion based on BehaviorStack.
/// Processes both built-in behaviors and FFI algorithm calls.
///
/// 根据行为栈更新弹幕运动的系统。
/// 同时处理内置行为和 FFI 算法调用。
pub fn update_bullet_motion(
    time: Res<Time>,
    danmaku_registry: Res<DanmakuRegistry>,
    player_query: Query<&Transform, (With<BehaviorParams>, Without<Bullet>)>,
    mut query: Query<
        (
            &mut Transform,
            &mut BulletMotionState,
            &BehaviorStack,
            &mut TweenState,
            Option<&mut Sprite>,
        ),
        With<Bullet>,
    >,
) {
    let dt = time.delta_secs();
    let player_pos = player_query
        .iter()
        .next()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (mut transform, mut state, behavior_stack, mut tween_state, sprite) in query.iter_mut() {
        state.elapsed += dt;

        let mut position = state.spawn_center + state.initial_offset;
        let mut rotation_delta = 0.0;
        let mut scale_delta = Vec2::ZERO;
        let mut opacity: Option<f32> = None;

        if tween_state.timers.len() < behavior_stack.behaviors.len() {
            tween_state
                .timers
                .resize(behavior_stack.behaviors.len(), 0.0);
        }

        for (i, behavior) in behavior_stack.behaviors.iter().enumerate() {
            match behavior {
                BulletBehavior::Linear(config) => {
                    let dir = Vec2::new(config.dir.0, config.dir.1).normalize_or_zero();
                    position += dir * config.speed * state.elapsed;
                }

                BulletBehavior::Circular(config) => {
                    let current_angle =
                        state.initial_angle + config.angular_velocity * state.elapsed;
                    let current_radius =
                        (state.initial_radius + config.radial_velocity * state.elapsed).max(0.0);

                    position = state.spawn_center
                        + Vec2::new(current_angle.cos(), current_angle.sin()) * current_radius;
                    rotation_delta += config.angular_velocity * dt;
                }

                BulletBehavior::Sine(config) => {
                    let axis_vec = Vec2::new(config.axis.0, config.axis.1).normalize_or_zero();
                    let wave = (state.elapsed * config.frequency * std::f32::consts::TAU
                        + config.phase)
                        .sin();
                    position += axis_vec * wave * config.amplitude;
                }

                BulletBehavior::Homing(config) => {
                    let to_player = (player_pos - position).normalize_or_zero();
                    let current_dir = state.velocity_direction;
                    let turn_amount = (config.strength * dt).min(config.max_turn_rate * dt);
                    let new_dir = current_dir.lerp(to_player, turn_amount).normalize_or_zero();
                    state.velocity_direction = new_dir;
                    position += new_dir * config.speed * dt;
                }

                BulletBehavior::Tween(config) => {
                    tween_state.timers[i] += dt;
                    let t = tween_state.timers[i] - config.delay;

                    if t >= 0.0 && t < config.duration {
                        let progress = (t / config.duration).clamp(0.0, 1.0);
                        let eased = config.ease.apply(progress);
                        let value = config.range.0 + (config.range.1 - config.range.0) * eased;

                        match config.target {
                            TweenTarget::Opacity => opacity = Some(value),
                            TweenTarget::Scale => scale_delta = Vec2::splat(value - 1.0),
                            TweenTarget::ScaleX => scale_delta.x = value - 1.0,
                            TweenTarget::ScaleY => scale_delta.y = value - 1.0,
                            TweenTarget::PositionX => position.x += value,
                            TweenTarget::PositionY => position.y += value,
                            TweenTarget::Rotation => rotation_delta += value,
                        }
                    } else if t >= config.duration {
                        let value = config.range.1;
                        match config.target {
                            TweenTarget::Opacity => opacity = Some(value),
                            TweenTarget::Scale => scale_delta = Vec2::splat(value - 1.0),
                            TweenTarget::ScaleX => scale_delta.x = value - 1.0,
                            TweenTarget::ScaleY => scale_delta.y = value - 1.0,
                            _ => {}
                        }
                    }
                }

                BulletBehavior::Algo { id, params } => {
                    if let Some(algo_fn) = danmaku_registry.get(id) {
                        let state_c = BulletStateC {
                            elapsed: state.elapsed,
                            spawn_x: state.spawn_center.x,
                            spawn_y: state.spawn_center.y,
                            offset_x: state.initial_offset.x,
                            offset_y: state.initial_offset.y,
                            initial_angle: state.initial_angle,
                            initial_radius: state.initial_radius,
                            dir_x: state.velocity_direction.x,
                            dir_y: state.velocity_direction.y,
                            params: params.as_ptr(),
                            params_len: params.len(),
                        };

                        let result: Vec2C = algo_fn(&state_c);
                        position += Vec2::new(result.x, result.y);
                    } else {
                        warn!("Danmaku algorithm '{}' not found in registry", id);
                    }
                }
            }
        }

        transform.translation.x = position.x;
        transform.translation.y = position.y;

        if rotation_delta != 0.0 {
            transform.rotate_z(rotation_delta);
        }

        if scale_delta != Vec2::ZERO {
            transform.scale.x = 1.0 + scale_delta.x;
            transform.scale.y = 1.0 + scale_delta.y;
        }

        if let (Some(opacity_val), Some(mut sprite)) = (opacity, sprite) {
            sprite.color.set_alpha(opacity_val);
        }
    }
}

// ============================================================================
// Bullet Lifecycle Systems
// ============================================================================

/// System to update bullet lifetime timers.
///
/// 更新弹幕生命周期计时器的系统。
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
