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
use super::danmaku_schema::*;
use super::target::BulletTarget;
use crate::app_state::ModeScoped;
use crate::config::load_config;
use crate::core::animation::components::{SpriteAnimationClip, SpriteAnimationTimer};
use crate::core::collision::TriggerCollider;
use crate::core::mod_system::DanmakuRegistry;
use crate::core::sprite::params::SpriteParams;
use crate::core::visual::{
    DEFAULT_FRAME_DURATION, ResolvedVisual, get_asset_path, resolve_visual_path,
};
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
            // Create BulletContainer first
            let mut container_commands = commands.spawn((
                BulletContainer {
                    center: event.position,
                },
                Transform::from_translation(event.position.extend(0.0)),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                Name::new("BulletContainer"),
            ));

            // Add mode-scoped marker to container
            if let Some(ref mode) = spawn_context.mode {
                container_commands.insert(ModeScoped(mode.clone()));
            }

            let container_entity = container_commands.id();

            // Create PerformancePlayer and link it to the container
            let mut player = PerformancePlayer::new(event.position);
            player.container_entity = Some(container_entity);

            let mut player_commands = commands.spawn((
                player,
                PerformanceHandle(handle.clone()),
                PerformancePlayerMarker,
                Name::new("PerformancePlayer"),
            ));

            // Add mode-scoped marker to PerformancePlayer too
            if let Some(ref mode) = spawn_context.mode {
                player_commands.insert(ModeScoped(mode.clone()));
            }

            info!(
                "Started performance: {} with container {:?}",
                event.performance_path, container_entity
            );
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
                player.container_entity,
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
            // Despawn the container (which will automatically despawn children in Bevy)
            if let Some(container) = player.container_entity {
                commands.entity(container).despawn();
            }
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
    container_entity: Option<Entity>,
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
            container_entity,
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
    container_entity: Option<Entity>,
    danmaku_registry: &DanmakuRegistry,
    _spawn_context: &DanmakuSpawnContext,
    sprite_params: &mut SpriteParams,
    asset_server: &AssetServer,
) {
    // Get scale from prototype
    let scale = prototype.scale;

    // Convert ColliderShape to TriggerCollider, scaled by prototype.scale
    // 将 ColliderShape 转换为 TriggerCollider，根据 prototype.scale 缩放
    let trigger_collider = match &prototype.collider {
        ColliderShape::CircleCollider(r) => TriggerCollider::Circle { radius: *r * scale },
        ColliderShape::BoxCollider(w, h) => TriggerCollider::Box {
            half_size: Vec2::new(*w * scale, *h * scale),
        },
    };

    // Convert HitBehaviorPreset to BulletHitBehavior component
    // 将 HitBehaviorPreset 转换为 BulletHitBehavior 组件
    let hit_behavior = match &prototype.hit_behavior {
        HitBehaviorPreset::Default => BulletHitBehavior::default_despawn(),
        HitBehaviorPreset::Persistent => BulletHitBehavior::persistent(),
        HitBehaviorPreset::DamageWhenMoving => BulletHitBehavior::damage_when_moving(),
        HitBehaviorPreset::DamageWhenStationary => BulletHitBehavior::damage_when_stationary(),
        HitBehaviorPreset::Custom {
            despawn_on_hit,
            damage_on_player_moving,
            damage_on_player_stationary,
            invincibility_duration,
        } => BulletHitBehavior {
            despawn_on_hit: *despawn_on_hit,
            damage_on_player_moving: *damage_on_player_moving,
            damage_on_player_stationary: *damage_on_player_stationary,
            invincibility_duration: *invincibility_duration,
        },
    };

    let mut entity_commands = commands.spawn((
        Bullet,
        Transform::from_translation(position.extend(prototype.z_index))
            .with_scale(Vec3::splat(scale)),
        GlobalTransform::default(),
        BulletLifetime::new(prototype.lifetime),
        BulletDamage(prototype.damage),
        BulletBaseScale(scale), // Store base scale for Tween calculations
        BulletMotionState::new(spawn_center)
            .with_offset(position - spawn_center)
            .with_angle(angle)
            .with_radius(radius),
        BehaviorStack::new(behaviors.to_vec()),
        TweenState::default(),
        trigger_collider,
        hit_behavior,
        BulletLastHitTime::default(),
        Name::new(format!("Bullet_{}", index)),
    ));

    // Set parent to container entity if available
    // 如果容器实体可用，将其设置为父实体
    if let Some(container) = container_entity {
        entity_commands.insert(ChildOf(container));
    } else {
        warn!("No container entity available for bullet {}", index);
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

    // Instantiate visual using the unified Visual type
    let config = load_config();
    let visual_path = prototype.visual.path();

    // Get rendering properties from prototype
    let effective_color = prototype.color_tint.to_color();
    let flip_x = prototype.flip_x;
    let flip_y = prototype.flip_y;
    let frame_duration = prototype.frame_duration.unwrap_or(DEFAULT_FRAME_DURATION);

    // Try to resolve the visual path
    if let Some(resolved) = resolve_visual_path(visual_path, &config.project.mod_name) {
        // Convert full path to asset-relative path
        let asset_path = get_asset_path(&resolved, &config.project.mod_name);

        match resolved {
            ResolvedVisual::Sprite(_) => {
                let mut sprite = Sprite {
                    image: asset_server.load(&asset_path),
                    flip_x,
                    flip_y,
                    ..default()
                };
                if let Some(color) = effective_color {
                    sprite.color = color;
                }
                entity_commands.insert(sprite);
            }
            ResolvedVisual::FrameAnimation(_dir_path) => {
                // For frame animations, we need to use the existing animation system
                let mut sprite = Sprite {
                    flip_x,
                    flip_y,
                    ..default()
                };
                if let Some(color) = effective_color {
                    sprite.color = color;
                }

                // Extract directory name from asset path for registry lookup
                let dir_name = std::path::Path::new(&asset_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                let mut sprite_context = sprite_params.create_sprite_context();
                // Try to find animation by directory name in any module
                if let Ok(clip) = SpriteAnimationClip::new(&mut sprite_context, "battle", dir_name)
                    .or_else(|_| SpriteAnimationClip::new(&mut sprite_context, "common", dir_name))
                    .or_else(|_| {
                        SpriteAnimationClip::new(&mut sprite_context, "overworld", dir_name)
                    })
                {
                    entity_commands.insert((
                        sprite,
                        clip,
                        SpriteAnimationTimer::new(frame_duration),
                    ));
                } else {
                    // Fallback: load first image from directory using asset-relative path
                    let fallback_path = format!("{}/0.png", asset_path);
                    sprite.image = asset_server.load(&fallback_path);
                    warn!(
                        "Animation '{}' not found in registry, using static fallback",
                        dir_name
                    );
                    entity_commands.insert(sprite);
                }
            }
            ResolvedVisual::CharacterAnimation(_path) => {
                // Character animations are not typically used for bullets
                warn!(
                    "Character animations not supported for bullets: {}",
                    visual_path
                );
                entity_commands.insert(Sprite::default());
            }
        }
    } else {
        // Fallback: try legacy module/name lookup for backwards compatibility
        // This handles cases like "battle/bullets/spear" that might reference config.toml
        let parts: Vec<&str> = visual_path.split('/').collect();
        if parts.len() < 2 {
            warn!("Failed to resolve visual: {}", visual_path);
            entity_commands.insert(Sprite::default());
        } else {
            let module = parts[0];
            let name = parts.last().unwrap_or(&"");

            let mut sprite_context = sprite_params.create_sprite_context();
            if let Ok(mut sprite) = sprite_context.get_sprite(module, name) {
                apply_color_tint(&mut sprite, effective_color);
                entity_commands.insert(sprite);
            } else if let Ok(clip) = SpriteAnimationClip::new(&mut sprite_context, module, name) {
                let mut sprite = Sprite::default();
                apply_color_tint(&mut sprite, effective_color);
                entity_commands.insert((sprite, clip, SpriteAnimationTimer::new(frame_duration)));
            } else {
                warn!("Failed to resolve visual: {}", visual_path);
                entity_commands.insert(Sprite::default());
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
    container_query: Query<&Transform, (With<BulletContainer>, Without<Bullet>)>,
    mut query: Query<
        (
            &mut Transform,
            &ChildOf,
            &mut BulletMotionState,
            &BehaviorStack,
            &mut TweenState,
            &BulletBaseScale,
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

    for (
        mut transform,
        parent,
        mut state,
        behavior_stack,
        mut tween_state,
        base_scale,
        sprite,
        active_danmaku,
    ) in query.iter_mut()
    {
        state.elapsed += dt;

        // Calculate world position based on behaviors
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
                    apply_tween_behavior(
                        config,
                        &mut tween_state,
                        i,
                        dt,
                        &mut opacity,
                        &mut scale_delta,
                        &mut position,
                        &mut rotation_delta,
                    );
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

        // Convert world position to local position relative to parent container
        // 将世界位置转换为相对于父容器的局部位置
        if let Ok(parent_transform) = container_query.get(parent.0) {
            let parent_pos = parent_transform.translation.truncate();
            let local_pos = position - parent_pos;
            transform.translation.x = local_pos.x;
            transform.translation.y = local_pos.y;
        } else {
            // Fallback to world position if parent not found
            transform.translation.x = position.x;
            transform.translation.y = position.y;
        }

        if rotation_delta != 0.0 {
            transform.rotate_z(rotation_delta);
        }

        // Apply scale based on base_scale * (1.0 + tween_delta)
        // This ensures Tween scales are relative to the prototype's base scale
        if scale_delta != Vec2::ZERO {
            transform.scale.x = base_scale.0 * (1.0 + scale_delta.x);
            transform.scale.y = base_scale.0 * (1.0 + scale_delta.y);
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

// ============================================================================
// Helper Functions
// ============================================================================

/// Apply optional color tint to a sprite.
fn apply_color_tint(sprite: &mut Sprite, color: Option<Color>) {
    if let Some(color) = color {
        sprite.color = color;
    }
}

/// Apply a tween value to the corresponding target property (during active tween).
/// Apply a tween behavior for a single frame, updating position/rotation/scale/opacity.
fn apply_tween_behavior(
    config: &TweenConfig,
    tween_state: &mut TweenState,
    index: usize,
    dt: f32,
    opacity: &mut Option<f32>,
    scale_delta: &mut Vec2,
    position: &mut Vec2,
    rotation_delta: &mut f32,
) {
    tween_state.timers[index] += dt;
    let t = tween_state.timers[index] - config.delay;

    if t >= 0.0 && t < config.duration {
        let progress = (t / config.duration).clamp(0.0, 1.0);
        let eased = config.ease.apply(progress);
        let value = config.range.0 + (config.range.1 - config.range.0) * eased;
        apply_tween_value(
            config.target,
            value,
            opacity,
            scale_delta,
            position,
            rotation_delta,
        );
    } else if t >= config.duration {
        let value = config.range.1;
        apply_tween_final_value(config.target, value, opacity, scale_delta);
    }
}

fn apply_tween_value(
    target: TweenTarget,
    value: f32,
    opacity: &mut Option<f32>,
    scale_delta: &mut Vec2,
    position: &mut Vec2,
    rotation_delta: &mut f32,
) {
    match target {
        TweenTarget::Opacity => *opacity = Some(value),
        TweenTarget::Scale => *scale_delta = Vec2::splat(value - 1.0),
        TweenTarget::ScaleX => scale_delta.x = value - 1.0,
        TweenTarget::ScaleY => scale_delta.y = value - 1.0,
        TweenTarget::PositionX => position.x += value,
        TweenTarget::PositionY => position.y += value,
        TweenTarget::Rotation => *rotation_delta += value,
    }
}

/// Apply a tween final value after the tween has completed (only persistent targets).
fn apply_tween_final_value(
    target: TweenTarget,
    value: f32,
    opacity: &mut Option<f32>,
    scale_delta: &mut Vec2,
) {
    match target {
        TweenTarget::Opacity => *opacity = Some(value),
        TweenTarget::Scale => *scale_delta = Vec2::splat(value - 1.0),
        TweenTarget::ScaleX => scale_delta.x = value - 1.0,
        TweenTarget::ScaleY => scale_delta.y = value - 1.0,
        _ => {}
    }
}
