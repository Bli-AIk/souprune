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
use crate::core::mod_system::{ActiveDanmaku, DanmakuRegistry, LoadedMods};
use crate::core::sprite::params::SpriteParams;
use crate::core::visual::{
    DEFAULT_FRAME_DURATION, ResolvedVisual, get_asset_path, resolve_visual_path,
};
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

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
    mut loaded_mods: NonSendMut<LoadedMods>,
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
                &mut loaded_mods,
                &spawn_context,
                &mut sprite_params,
                &asset_server,
            );

            player.next_event_index += 1;
        }

        // Check if performance is finished (all events fired)
        if player.next_event_index >= performance.timeline.len() {
            player.finished = true;
            // Only despawn the player; the container and its bullet children
            // remain alive until bullets expire via BulletLifetime.
            // Empty containers are cleaned up by cleanup_empty_containers.
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
    loaded_mods: &mut LoadedMods,
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

    for_each_spawn_point(&event.pattern, spawn_center, |i, point| {
        spawn_single_bullet(
            commands,
            prototype,
            &behaviors,
            &point,
            spawn_center,
            player_pos,
            i,
            container_entity,
            danmaku_registry,
            loaded_mods,
            spawn_context,
            sprite_params,
            asset_server,
        );
    });
}

/// A computed spawn point for a bullet within a pattern.
struct SpawnPoint {
    /// World position where the bullet should spawn
    position: Vec2,
    /// Initial angle in radians (from center to this point)
    angle: f32,
    /// Distance from pattern center
    radius: f32,
}

/// Iterate over spawn points for a given pattern, invoking `f` for each one.
/// Uses a callback to avoid heap allocation (especially for `Single`).
fn for_each_spawn_point(
    pattern: &SpawnPattern,
    center: Vec2,
    mut f: impl FnMut(usize, SpawnPoint),
) {
    match pattern {
        SpawnPattern::Single { offset } => {
            f(
                0,
                SpawnPoint {
                    position: center + Vec2::new(offset.0, offset.1),
                    angle: 0.0,
                    radius: 0.0,
                },
            );
        }
        SpawnPattern::RingGenerator {
            count,
            radius,
            start_angle,
        } => {
            let angle_step = std::f32::consts::TAU / *count as f32;
            for i in 0..*count {
                let angle = start_angle + angle_step * i as f32;
                let pos = center + Vec2::from_angle(angle) * *radius;
                f(
                    i,
                    SpawnPoint {
                        position: pos,
                        angle,
                        radius: *radius,
                    },
                );
            }
        }
        SpawnPattern::LineGenerator {
            count,
            spacing,
            direction,
        } => {
            let dir = Vec2::new(direction.0, direction.1).normalize_or_zero();
            let perp = dir.perp();
            let total_width = *spacing * (*count - 1) as f32;
            let start_offset = -total_width / 2.0;
            let angle = dir.to_angle();

            for i in 0..*count {
                let offset = start_offset + *spacing * i as f32;
                let pos = center + perp * offset;
                f(
                    i,
                    SpawnPoint {
                        position: pos,
                        angle,
                        radius: 0.0,
                    },
                );
            }
        }
        SpawnPattern::EdgeGenerator {
            count,
            side,
            spacing,
            margin,
        } => {
            let move_dir = side.to_direction();
            let start_offset = side.to_offset(*margin);
            let perp = move_dir.perp();
            let total_width = *spacing * (*count - 1) as f32;
            let start_perp_offset = -total_width / 2.0;
            let angle = move_dir.to_angle();

            for i in 0..*count {
                let perp_offset = start_perp_offset + *spacing * i as f32;
                let pos = center + start_offset + perp * perp_offset;
                f(
                    i,
                    SpawnPoint {
                        position: pos,
                        angle,
                        radius: 0.0,
                    },
                );
            }
        }
        SpawnPattern::CustomGenerator { id, .. } => {
            warn!("Custom spawn pattern '{}' not yet implemented", id);
            f(
                0,
                SpawnPoint {
                    position: center,
                    angle: 0.0,
                    radius: 0.0,
                },
            );
        }
    }
}

/// Spawn a single bullet entity with BehaviorStack.
fn spawn_single_bullet(
    commands: &mut Commands,
    prototype: &BulletPrototype,
    behaviors: &[BulletBehavior],
    point: &SpawnPoint,
    spawn_center: Vec2,
    player_pos: Vec2,
    index: usize,
    container_entity: Option<Entity>,
    danmaku_registry: &DanmakuRegistry,
    loaded_mods: &mut LoadedMods,
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
        Transform::from_translation(point.position.extend(prototype.z_index))
            .with_scale(Vec3::splat(scale)),
        GlobalTransform::default(),
        BulletLifetime::new(prototype.lifetime),
        BulletDamage(prototype.damage),
        BulletBaseScale(scale), // Store base scale for Tween calculations
        BulletMotionState::new(spawn_center)
            .with_offset(point.position - spawn_center)
            .with_angle(point.angle)
            .with_radius(point.radius),
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
            if let Some(mut active_danmaku) = danmaku_registry.create(id, loaded_mods) {
                active_danmaku.props = props.clone();

                // Build initial context and call on_enter
                let offset = point.position - spawn_center;
                let ctx = souprune_api::BulletContext {
                    elapsed: 0.0,
                    delta_time: 0.0,
                    spawn_pos: souprune_api::Vec2::new(spawn_center.x, spawn_center.y),
                    offset: souprune_api::Vec2::new(offset.x, offset.y),
                    initial_angle: point.angle,
                    initial_radius: point.radius,
                    player_pos: souprune_api::Vec2::new(player_pos.x, player_pos.y),
                    props: props
                        .iter()
                        .map(|(name, value)| souprune_api::Prop {
                            name: name.clone(),
                            value: *value,
                        })
                        .collect(),
                };
                active_danmaku.call_on_enter(&ctx, loaded_mods);

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
        let parts: Vec<&str> = visual_path.split('/').collect();

        let mut sprite_context = sprite_params.create_sprite_context();
        let mut resolved = false;

        if parts.len() >= 2 {
            let module = parts[0];
            let name = parts.last().unwrap_or(&"");

            if let Ok(mut sprite) = sprite_context.get_sprite(module, name) {
                apply_color_tint(&mut sprite, effective_color);
                entity_commands.insert(sprite);
                resolved = true;
            } else if let Ok(clip) = SpriteAnimationClip::new(&mut sprite_context, module, name) {
                let mut sprite = Sprite::default();
                apply_color_tint(&mut sprite, effective_color);
                entity_commands.insert((sprite, clip, SpriteAnimationTimer::new(frame_duration)));
                resolved = true;
            }
        }

        // Plain name without "/": search common modules in config.toml
        if !resolved {
            let name = parts.last().unwrap_or(&"");
            for module in &["battle", "common", "overworld"] {
                if let Ok(mut sprite) = sprite_context.get_sprite(module, name) {
                    apply_color_tint(&mut sprite, effective_color);
                    entity_commands.insert(sprite);
                    resolved = true;
                    break;
                }
            }
        }

        if !resolved {
            warn!("Failed to resolve visual: {}", visual_path);
            entity_commands.insert(Sprite::default());
        }
    }
}

// ============================================================================
// Bullet Motion System
// ============================================================================

/// System to update bullet motion based on BehaviorStack.
/// Processes both built-in behaviors and WASM mod algorithm calls.
///
/// 根据行为栈更新弹幕运动的系统。
/// 同时处理内置行为和 WASM mod 算法调用。
pub fn update_bullet_motion(
    time: Res<Time>,
    mut loaded_mods: NonSendMut<LoadedMods>,
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
                BulletBehavior::Stationary() => {
                    // skip - Because it is just stationary!
                }
            }
        }

        // Handle ActiveDanmaku (WASM-based API)
        if let Some(mut danmaku) = active_danmaku {
            let ctx = souprune_api::BulletContext {
                elapsed: state.elapsed,
                delta_time: dt,
                spawn_pos: souprune_api::Vec2::new(state.spawn_center.x, state.spawn_center.y),
                offset: souprune_api::Vec2::new(state.initial_offset.x, state.initial_offset.y),
                initial_angle: state.initial_angle,
                initial_radius: state.initial_radius,
                player_pos: souprune_api::Vec2::new(player_pos.x, player_pos.y),
                props: danmaku
                    .props
                    .iter()
                    .map(|(name, value)| souprune_api::Prop {
                        name: name.clone(),
                        value: *value,
                    })
                    .collect(),
            };

            let output = danmaku.call_on_update(&ctx, &mut loaded_mods);
            position += Vec2::new(output.offset.x, output.offset.y);
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
/// Calls WASM on_exit for any active danmaku before despawning.
///
/// 清理标记为销毁的弹幕的系统。
/// 在销毁前为活跃的 WASM 弹幕调用 on_exit。
pub fn cleanup_dead_bullets(
    mut commands: Commands,
    mut query: Query<(Entity, Option<&mut ActiveDanmaku>), (With<Bullet>, With<DespawnBullet>)>,
    mut loaded_mods: NonSendMut<LoadedMods>,
) {
    for (entity, active_danmaku) in query.iter_mut() {
        if let Some(mut danmaku) = active_danmaku {
            danmaku.call_on_exit(&mut loaded_mods);
        }
        commands.entity(entity).despawn();
    }
}

/// System to despawn bullet containers that have no remaining children.
/// Runs after dead bullets are cleaned up so empty containers are removed.
///
/// 销毁没有剩余子实体的弹幕容器。
/// 在清理死亡弹幕之后运行，以便移除空容器。
pub fn cleanup_empty_containers(
    mut commands: Commands,
    container_query: Query<(Entity, &Children), With<BulletContainer>>,
) {
    for (entity, children) in container_query.iter() {
        if children.is_empty() {
            commands.entity(entity).despawn();
        }
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
