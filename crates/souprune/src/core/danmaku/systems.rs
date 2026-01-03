//! # danmaku/systems.rs
//!
//! ## Module Overview
//!
//! Implements runtime systems for the timeline-based danmaku system.
//! This is the core state-agnostic implementation.
//!
//! 实现基于时间轴的弹幕系统的运行时系统。
//! 这是与状态无关的核心实现。

use super::DanmakuSpawnContext;
use super::components::*;
use super::patterns::*;
use super::target::BulletTarget;
use crate::app_state::battle::BattleEntity;
use crate::app_state::overworld::OverworldEntity;
use crate::core::animation::components::{SpriteAnimationClip, SpriteAnimationTimer};
use crate::core::collision::TriggerCollider;
use crate::core::mod_system::DanmakuRegistry;
use crate::core::sprite::params::SpriteParams;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use souprune_api::BulletContextC;

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
    spawn_context: Res<DanmakuSpawnContext>,
) {
    let mut still_pending = Vec::new();

    for (handle, event) in pending.pending.drain(..) {
        if performances.get(&handle).is_some() {
            let mut entity_commands = commands.spawn((
                PerformancePlayer::new(event.position),
                PerformanceHandle(handle.clone()),
                PerformancePlayerMarker,
                Name::new("PerformancePlayer"),
            ));

            // Add context-specific marker
            match *spawn_context {
                DanmakuSpawnContext::Battle => {
                    entity_commands.insert(BattleEntity);
                }
                DanmakuSpawnContext::Overworld => {
                    entity_commands.insert(OverworldEntity());
                }
            }

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
    danmaku_registry: Res<DanmakuRegistry>,
    spawn_context: Res<DanmakuSpawnContext>,
    mut query: Query<(Entity, &mut PerformancePlayer, &PerformanceHandle)>,
    // Use BulletTarget instead of BehaviorParams for generalized targeting
    player_query: Query<&Transform, (With<BulletTarget>, Without<Bullet>)>,
    mut sprite_params: SpriteParams,
    asset_server: Res<AssetServer>,
) {
    let dt = time.delta_secs();

    // Get player position for aimed/homing behaviors
    let player_pos = player_query
        .iter()
        .next()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (entity, mut player, perf_handle) in query.iter_mut() {
        if player.finished {
            continue;
        }

        player.elapsed += dt;

        let Some(performance) = performances.get(&perf_handle.0) else {
            continue;
        };

        // Calculate absolute trigger times from timeline events
        // 从时间线事件计算绝对触发时间
        let trigger_times = calculate_absolute_trigger_times(&performance.timeline);

        // Process timeline events that should fire
        while player.next_event_index < performance.timeline.len() {
            let event = &performance.timeline[player.next_event_index];
            let trigger_time = trigger_times[player.next_event_index];

            if trigger_time > player.elapsed {
                break;
            }

            // Fire this event
            spawn_bullets_from_timeline_event(
                &mut commands,
                performance,
                event,
                player.spawn_center,
                player_pos,
                &danmaku_registry,
                &spawn_context,
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

/// Calculate absolute trigger times from timeline events.
/// Handles both absolute and relative time modes.
///
/// 从时间线事件计算绝对触发时间。
/// 同时处理绝对时间和相对时间模式。
fn calculate_absolute_trigger_times(timeline: &[TimelineEvent]) -> Vec<f32> {
    let mut times = Vec::with_capacity(timeline.len());
    let mut accumulated = 0.0;

    for event in timeline {
        if event.absolute {
            // Absolute time: use t directly
            accumulated = event.t;
        } else {
            // Relative time: add t to accumulated
            accumulated += event.t;
        }
        times.push(accumulated);
    }

    times
}

/// Spawn bullets from a timeline event.
fn spawn_bullets_from_timeline_event(
    commands: &mut Commands,
    performance: &DanmakuPerformance,
    event: &TimelineEvent,
    spawn_center: Vec2,
    player_pos: Vec2,
    danmaku_registry: &DanmakuRegistry,
    spawn_context: &DanmakuSpawnContext,
    sprite_params: &mut SpriteParams,
    asset_server: &AssetServer,
) {
    let Some(prototype) = performance.prototypes.get(&event.spawn) else {
        warn!("Prototype not found: {}", event.spawn);
        return;
    };

    // Collect behaviors: first from references, then inline behaviors
    // 收集行为：首先是引用的行为，然后是内联行为
    let mut behaviors: Vec<BulletBehavior> = event
        .apply
        .iter()
        .filter_map(|id| performance.behaviors.get(id).cloned())
        .collect();

    // Append inline behaviors
    // 追加内联行为
    behaviors.extend(event.behaviors.clone());

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
            player_pos,
            i,
            danmaku_registry,
            spawn_context,
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
        SpawnPattern::RingGenerator {
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
        SpawnPattern::LineGenerator {
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
        SpawnPattern::EdgeGenerator {
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
        SpawnPattern::CustomGenerator { id, .. } => {
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
    player_pos: Vec2,
    index: usize,
    danmaku_registry: &DanmakuRegistry,
    spawn_context: &DanmakuSpawnContext,
    sprite_params: &mut SpriteParams,
    asset_server: &AssetServer,
) {
    // Convert ColliderShape to TriggerCollider
    // 将 ColliderShape 转换为 TriggerCollider
    let trigger_collider = match &prototype.collider {
        ColliderShape::CircleCollider(r) => TriggerCollider::Circle { radius: *r },
        ColliderShape::BoxCollider(w, h) => TriggerCollider::Box {
            half_size: Vec2::new(*w, *h),
        },
    };

    let mut entity_commands = commands.spawn((
        Bullet,
        Transform::from_translation(position.extend(prototype.z_index)),
        BulletLifetime::new(prototype.lifetime),
        BulletDamage(prototype.damage),
        BulletMotionState::new(spawn_center)
            .with_offset(position - spawn_center)
            .with_angle(angle)
            .with_radius(radius),
        BehaviorStack::new(behaviors.to_vec()),
        TweenState::default(),
        trigger_collider,
        Name::new(format!("Bullet_{}", index)),
    ));

    // Add context-specific entity marker
    // 添加上下文特定的实体标记
    match spawn_context {
        DanmakuSpawnContext::Battle => {
            entity_commands.insert(BattleEntity);
        }
        DanmakuSpawnContext::Overworld => {
            entity_commands.insert(OverworldEntity());
        }
    }

    // Create ActiveDanmaku instances for Custom behaviors and call on_enter
    // 为 Custom 行为创建 ActiveDanmaku 实例并调用 on_enter
    for behavior in behaviors {
        if let BulletBehavior::Custom { id, props } = behavior {
            if let Some(instance) = danmaku_registry.create(id) {
                let mut active_danmaku = ActiveDanmaku::new(instance, props.clone(), Vec::new());
                let (props_ptr, props_len) = active_danmaku.ffi_props();

                // Build initial context and call on_enter
                // 构建初始上下文并调用 on_enter
                let ctx = BulletContextC {
                    elapsed: 0.0,
                    delta_time: 0.0,
                    spawn_x: spawn_center.x,
                    spawn_y: spawn_center.y,
                    offset_x: position.x - spawn_center.x,
                    offset_y: position.y - spawn_center.y,
                    initial_angle: angle,
                    initial_radius: radius,
                    player_x: player_pos.x,
                    player_y: player_pos.y,
                    props: props_ptr,
                    props_len,
                    params: std::ptr::null(),
                    params_len: 0,
                };
                active_danmaku.call_on_enter(&ctx);

                entity_commands.insert(active_danmaku);
                // Only support one Custom behavior per bullet for now
                break;
            } else {
                warn!("Danmaku algorithm '{}' not found in registry", id);
            }
        }
    }

    match &prototype.visual {
        BulletVisual::Sprite { path } => {
            entity_commands.insert(Sprite {
                image: asset_server.load(path),
                ..default()
            });
        }
        BulletVisual::SpriteRef { module, name } => {
            let mut sprite_context = sprite_params.create_sprite_context();
            match sprite_context.get_sprite(module, name) {
                Ok(sprite) => {
                    entity_commands.insert(sprite);
                }
                Err(e) => {
                    warn!("Failed to load sprite '{}': {}", name, e);
                    entity_commands.insert(Sprite::default());
                }
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
    // Use BulletTarget for generalized targeting
    player_query: Query<&Transform, (With<BulletTarget>, Without<Bullet>)>,
    mut query: Query<
        (
            &mut Transform,
            &mut BulletMotionState,
            &BehaviorStack,
            &mut TweenState,
            Option<&mut Sprite>,
            Option<&mut ActiveDanmaku>,
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

    for (mut transform, mut state, behavior_stack, mut tween_state, sprite, active_danmaku) in
        query.iter_mut()
    {
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

                BulletBehavior::Orbital(config) => {
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

                // Custom behaviors are handled separately via ActiveDanmaku
                BulletBehavior::Custom { .. } => {
                    // Skip - handled below
                }
            }
        }

        // Handle ActiveDanmaku (new VTable-based API)
        // 处理 ActiveDanmaku（新的基于 VTable 的 API）
        if let Some(mut danmaku) = active_danmaku {
            let (props_ptr, props_len) = danmaku.ffi_props();
            let ctx = BulletContextC {
                elapsed: state.elapsed,
                delta_time: dt,
                spawn_x: state.spawn_center.x,
                spawn_y: state.spawn_center.y,
                offset_x: state.initial_offset.x,
                offset_y: state.initial_offset.y,
                initial_angle: state.initial_angle,
                initial_radius: state.initial_radius,
                player_x: player_pos.x,
                player_y: player_pos.y,
                props: props_ptr,
                props_len,
                params: danmaku.params.as_ptr(),
                params_len: danmaku.params.len(),
            };

            let output = danmaku.call_on_update(&ctx);
            position += Vec2::new(output.offset_x, output.offset_y);
            rotation_delta += output.rotation;
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
