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
use crate::core::mod_system::{
    ActiveDanmakuStack, DanmakuRegistry, LoadedMods, SpawnPatternRegistry,
};
use crate::core::sprite::params::SpriteParams;
use crate::core::visual::{
    DEFAULT_FRAME_DURATION, ResolvedVisual, get_asset_path, resolve_visual_path,
};
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use std::collections::HashMap;

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
    pattern_registry: Res<SpawnPatternRegistry>,
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
                &pattern_registry,
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
    pattern_registry: &SpawnPatternRegistry,
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

    // Compute effective center with event offset
    let effective_center = spawn_center + Vec2::new(event.offset.0, event.offset.1);

    let points = collect_spawn_points(
        &event.pattern,
        effective_center,
        player_pos,
        0.0,
        pattern_registry,
        loaded_mods,
    );

    for (i, point) in points.iter().enumerate() {
        spawn_single_bullet(
            commands,
            prototype,
            &behaviors,
            point,
            effective_center,
            player_pos,
            i,
            container_entity,
            danmaku_registry,
            loaded_mods,
            spawn_context,
            sprite_params,
            asset_server,
        );
    }
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

/// Compute spawn points for a given pattern via WASM dispatch.
/// All patterns (including builtins) are resolved through the SpawnPatternRegistry.
fn collect_spawn_points(
    pattern: &SpawnPattern,
    center: Vec2,
    player_pos: Vec2,
    time: f32,
    pattern_registry: &SpawnPatternRegistry,
    loaded_mods: &mut LoadedMods,
) -> Vec<SpawnPoint> {
    let (id, params_map) = pattern.to_wasm_call();

    let ctx = souprune_api::SpawnContext {
        center_x: center.x,
        center_y: center.y,
        player_x: player_pos.x,
        player_y: player_pos.y,
        time,
    };
    let api_params: Vec<souprune_api::PatternParam> = params_map
        .iter()
        .map(|(name, value)| souprune_api::PatternParam {
            name: name.clone(),
            value: f64::from(*value),
        })
        .collect();

    match pattern_registry.generate(&id, &ctx, &api_params, loaded_mods) {
        Some(pts) => pts
            .into_iter()
            .map(|p| SpawnPoint {
                position: Vec2::new(p.x, p.y),
                angle: p.angle,
                radius: p.radius,
            })
            .collect(),
        None => {
            warn!("Pattern '{}' not found in any loaded WASM module", id);
            vec![SpawnPoint {
                position: center,
                angle: 0.0,
                radius: 0.0,
            }]
        }
    }
}

/// Spawn a single bullet entity with all behaviors dispatched via WASM.
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
    let scale = prototype.scale;

    let trigger_collider = match &prototype.collider {
        ColliderShape::CircleCollider(r) => TriggerCollider::Circle { radius: *r * scale },
        ColliderShape::BoxCollider(w, h) => TriggerCollider::Box {
            half_size: Vec2::new(*w * scale, *h * scale),
        },
    };

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

    let motion_state = BulletMotionState::new(spawn_center)
        .with_offset(point.position - spawn_center)
        .with_angle(point.angle)
        .with_radius(point.radius);

    let mut entity_commands = commands.spawn((
        Bullet,
        Transform::from_translation(point.position.extend(prototype.z_index))
            .with_scale(Vec3::splat(scale)),
        GlobalTransform::default(),
        BulletLifetime::new(prototype.lifetime),
        BulletDamage(prototype.damage),
        BulletBaseScale(scale),
        motion_state,
        BehaviorStack::new(behaviors.to_vec()),
        trigger_collider,
        hit_behavior,
        BulletLastHitTime::default(),
        Name::new(format!("Bullet_{}", index)),
    ));

    if let Some(container) = container_entity {
        entity_commands.insert(ChildOf(container));
    } else {
        warn!("No container entity available for bullet {}", index);
    }

    // Create ActiveDanmakuStack: each behavior becomes a WASM instance
    let offset = point.position - spawn_center;
    let mut stack = ActiveDanmakuStack::default();

    for behavior in behaviors {
        let (id, props) = behavior.to_wasm_call();

        let Some(mut active) = danmaku_registry.create(&id, loaded_mods) else {
            warn!("Danmaku algorithm '{}' not found in registry", id);
            continue;
        };
        active.props = props.clone();

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
        active.call_on_enter(&ctx, loaded_mods);
        stack.instances.push(active);
    }

    entity_commands.insert(stack);

    // Instantiate visual
    spawn_bullet_visual(&mut entity_commands, prototype, sprite_params, asset_server);
}

/// Resolve and attach the visual component to a bullet entity.
fn spawn_bullet_visual(
    entity_commands: &mut EntityCommands,
    prototype: &BulletPrototype,
    sprite_params: &mut SpriteParams,
    asset_server: &AssetServer,
) {
    let config = load_config();
    let visual_path = prototype.visual.path();
    let effective_color = prototype.color_tint.to_color();
    let flip_x = prototype.flip_x;
    let flip_y = prototype.flip_y;
    let frame_duration = prototype.frame_duration.unwrap_or(DEFAULT_FRAME_DURATION);

    if let Some(resolved) = resolve_visual_path(visual_path, &config.project.mod_name) {
        let asset_path = get_asset_path(&resolved, &config.project.mod_name);
        spawn_resolved_visual(
            entity_commands,
            resolved,
            asset_path,
            effective_color,
            flip_x,
            flip_y,
            frame_duration,
            sprite_params,
            asset_server,
            visual_path,
        );
        return;
    }

    // Fallback: try resolving as module/name path
    let parts: Vec<&str> = visual_path.split('/').collect();
    let mut sprite_context = sprite_params.create_sprite_context();

    if parts.len() >= 2 {
        let module = parts[0];
        let name = parts.last().unwrap_or(&"");

        if let Ok(mut sprite) = sprite_context.get_sprite(module, name) {
            apply_color_tint(&mut sprite, effective_color);
            entity_commands.insert(sprite);
            return;
        }
        if let Ok(clip) = SpriteAnimationClip::new(&mut sprite_context, module, name) {
            let mut sprite = Sprite::default();
            apply_color_tint(&mut sprite, effective_color);
            entity_commands.insert((sprite, clip, SpriteAnimationTimer::new(frame_duration)));
            return;
        }
    }

    // Plain name without "/": search common modules
    let name = parts.last().unwrap_or(&"");
    for module in &["battle", "common", "overworld"] {
        if let Ok(mut sprite) = sprite_context.get_sprite(module, name) {
            apply_color_tint(&mut sprite, effective_color);
            entity_commands.insert(sprite);
            return;
        }
    }

    warn!("Failed to resolve visual: {}", visual_path);
    entity_commands.insert(Sprite::default());
}

fn spawn_resolved_visual(
    entity_commands: &mut EntityCommands,
    resolved: ResolvedVisual,
    asset_path: String,
    effective_color: Option<Color>,
    flip_x: bool,
    flip_y: bool,
    frame_duration: f32,
    sprite_params: &mut SpriteParams,
    asset_server: &AssetServer,
    visual_path: &str,
) {
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
        ResolvedVisual::FrameAnimation(_) => {
            let mut sprite = Sprite {
                flip_x,
                flip_y,
                ..default()
            };
            if let Some(color) = effective_color {
                sprite.color = color;
            }

            let dir_name = std::path::Path::new(&asset_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let mut sprite_context = sprite_params.create_sprite_context();
            let clip_result = SpriteAnimationClip::new(&mut sprite_context, "battle", dir_name)
                .or_else(|_| SpriteAnimationClip::new(&mut sprite_context, "common", dir_name))
                .or_else(|_| SpriteAnimationClip::new(&mut sprite_context, "overworld", dir_name));

            if let Ok(clip) = clip_result {
                entity_commands.insert((sprite, clip, SpriteAnimationTimer::new(frame_duration)));
            } else {
                let fallback_path = format!("{}/0.png", asset_path);
                sprite.image = asset_server.load(&fallback_path);
                warn!(
                    "Animation '{}' not found in registry, using static fallback",
                    dir_name
                );
                entity_commands.insert(sprite);
            }
        }
        ResolvedVisual::CharacterAnimation(_) => {
            warn!(
                "Character animations not supported for bullets: {}",
                visual_path
            );
            entity_commands.insert(Sprite::default());
        }
    }
}

// ============================================================================
// Bullet Motion System
// ============================================================================

fn build_bullet_ctx(
    state: &BulletMotionState,
    dt: f32,
    player_pos: Vec2,
    props: &HashMap<String, f32>,
) -> souprune_api::BulletContext {
    souprune_api::BulletContext {
        elapsed: state.elapsed,
        delta_time: dt,
        spawn_pos: souprune_api::Vec2::new(state.spawn_center.x, state.spawn_center.y),
        offset: souprune_api::Vec2::new(state.initial_offset.x, state.initial_offset.y),
        initial_angle: state.initial_angle,
        initial_radius: state.initial_radius,
        player_pos: souprune_api::Vec2::new(player_pos.x, player_pos.y),
        props: props
            .iter()
            .map(|(name, value)| souprune_api::Prop {
                name: name.clone(),
                value: *value,
            })
            .collect(),
    }
}

fn apply_output_extras(
    output: &souprune_api::BulletOutput,
    opacity: &mut Option<f32>,
    scale_delta: &mut Vec2,
) {
    if output.opacity >= 0.0 {
        *opacity = Some(output.opacity);
    }
    if output.scale_x != 0.0 {
        scale_delta.x += output.scale_x;
    }
    if output.scale_y != 0.0 {
        scale_delta.y += output.scale_y;
    }
}

/// System to update bullet motion via WASM-dispatched behaviors.
/// All behaviors (including builtins) are processed through ActiveDanmakuStack.
///
/// 通过 WASM 调度的行为更新弹幕运动的系统。
/// 所有行为（包括内置行为）都通过 ActiveDanmakuStack 处理。
pub fn update_bullet_motion(
    time: Res<Time>,
    mut loaded_mods: NonSendMut<LoadedMods>,
    container_query: Query<&Transform, (With<BulletContainer>, Without<Bullet>)>,
    mut query: Query<
        (
            &mut Transform,
            &ChildOf,
            &mut BulletMotionState,
            &BehaviorStack,
            &BulletBaseScale,
            Option<&mut Sprite>,
            Option<&mut ActiveDanmakuStack>,
        ),
        With<Bullet>,
    >,
    player_query: Query<&Transform, (With<BulletTarget>, Without<Bullet>)>,
) {
    let dt = time.delta_secs();
    let player_pos = player_query
        .iter()
        .next()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (mut transform, parent, mut state, behavior_stack, base_scale, sprite, danmaku_stack) in
        query.iter_mut()
    {
        state.elapsed += dt;

        let mut position = state.spawn_center + state.initial_offset;
        let mut rotation_delta = 0.0;
        let mut scale_delta = Vec2::ZERO;
        let mut opacity: Option<f32> = None;

        // Process all behaviors via WASM instances
        let Some(mut stack) = danmaku_stack else {
            continue;
        };
        for (i, instance) in stack.instances.iter_mut().enumerate() {
            let props = behavior_stack
                .behaviors
                .get(i)
                .map(|b| b.to_wasm_call().1)
                .unwrap_or_else(|| instance.props.clone());

            let ctx = build_bullet_ctx(&state, dt, player_pos, &props);
            let output = instance.call_on_update(&ctx, &mut loaded_mods);

            position += Vec2::new(output.offset.x, output.offset.y);
            rotation_delta += output.rotation;
            apply_output_extras(&output, &mut opacity, &mut scale_delta);
        }

        // Convert world position to local position relative to parent container
        if let Ok(parent_transform) = container_query.get(parent.0) {
            let parent_pos = parent_transform.translation.truncate();
            let local_pos = position - parent_pos;
            transform.translation.x = local_pos.x;
            transform.translation.y = local_pos.y;
        } else {
            transform.translation.x = position.x;
            transform.translation.y = position.y;
        }

        if rotation_delta != 0.0 {
            transform.rotate_z(rotation_delta);
        }

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
    mut query: Query<
        (Entity, Option<&mut ActiveDanmakuStack>),
        (With<Bullet>, With<DespawnBullet>),
    >,
    mut loaded_mods: NonSendMut<LoadedMods>,
) {
    for (entity, danmaku_stack) in query.iter_mut() {
        if let Some(mut stack) = danmaku_stack {
            stack.call_on_exit_all(&mut loaded_mods);
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
