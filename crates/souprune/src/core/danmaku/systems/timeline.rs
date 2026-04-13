//! # timeline.rs
//!
//! # timeline.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Advances danmaku performances along their authored timeline. It computes absolute
//! trigger times, resolves the pattern-generated spawn points for each event, and turns each
//! timeline hit into concrete bullets with the right behaviors and container relationship.
//!
//! 负责沿着作者编排好的时间轴推进弹幕演出。它会计算每个事件的绝对触发时间，解析
//! 图案生成出的发射点，并把每一次时间轴命中转换成真正带有行为和容器关系的子弹实体。

use super::*;

/// A computed spawn point for a bullet within a pattern.
struct SpawnPoint {
    /// World position where the bullet should spawn.
    position: Vec2,
    /// Initial angle in radians (from center to this point).
    angle: f32,
    /// Distance from pattern center.
    radius: f32,
}

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
    player_query: Query<&Transform, (With<BulletTarget>, Without<Bullet>)>,
    viewbox_query: Query<(
        &Name,
        &crate::core::view::components::box_components::ViewBox,
    )>,
    mut sprite_params: SpriteParams,
    asset_server: Res<AssetServer>,
) {
    let dt = time.delta_secs();

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

        let trigger_times = calculate_absolute_trigger_times(&performance.timeline);

        while player.next_event_index < performance.timeline.len() {
            let event = &performance.timeline[player.next_event_index];
            let trigger_time = trigger_times[player.next_event_index];

            if trigger_time > player.elapsed {
                break;
            }

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
                &viewbox_query,
                &mut sprite_params,
                &asset_server,
            );

            player.next_event_index += 1;
        }

        if player.next_event_index >= performance.timeline.len() {
            player.finished = true;
            commands.entity(entity).despawn();
        }
    }
}

/// Calculate absolute trigger times from timeline events.
fn calculate_absolute_trigger_times(timeline: &[TimelineEvent]) -> Vec<f32> {
    use souprune_schema::danmaku::TimeMode;

    let mut times = Vec::with_capacity(timeline.len());
    let mut accumulated = 0.0;

    for event in timeline {
        accumulated = match event.time_mode {
            TimeMode::Absolute => event.t,
            TimeMode::Delta => accumulated + event.t,
        };
        times.push(accumulated);
    }

    times
}

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
    viewbox_query: &Query<(
        &Name,
        &crate::core::view::components::box_components::ViewBox,
    )>,
    sprite_params: &mut SpriteParams,
    asset_server: &AssetServer,
) {
    let Some(prototype) = performance.prototypes.get(&event.spawn) else {
        warn!("Prototype not found: {}", event.spawn);
        return;
    };

    let mut behaviors: Vec<BulletBehavior> = event
        .apply
        .iter()
        .filter_map(|id| performance.behaviors.get(id).cloned())
        .collect();
    behaviors.extend(event.behaviors.clone());

    let resolved_pattern = resolve_pattern_with_viewbox(&event.pattern, viewbox_query);

    let effective_center = spawn_center + Vec2::new(event.offset.0, event.offset.1);
    let points = collect_spawn_points(
        &resolved_pattern,
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

/// Resolves `BoxEdgeGenerator` patterns by looking up the named ViewBox's
/// current dimensions. Other patterns pass through unchanged.
///
/// 通过查找命名 ViewBox 的当前尺寸解析 `BoxEdgeGenerator`。
/// 其他 pattern 原样返回。
fn resolve_pattern_with_viewbox(
    pattern: &SpawnPattern,
    viewbox_query: &Query<(
        &Name,
        &crate::core::view::components::box_components::ViewBox,
    )>,
) -> SpawnPattern {
    if let SpawnPattern::BoxEdgeGenerator { box_name, .. } = pattern {
        let found = viewbox_query
            .iter()
            .find(|(name, _)| name.as_str() == box_name);

        match found {
            Some((_, view_box)) => {
                resolve_box_edge_pattern(pattern, view_box.width, view_box.height)
            }
            None => {
                warn!(
                    "BoxEdgeGenerator: ViewBox '{}' not found, falling back to zero box size",
                    box_name
                );
                resolve_box_edge_pattern(pattern, 0.0, 0.0)
            }
        }
    } else {
        pattern.clone()
    }
}

fn collect_spawn_points(
    pattern: &SpawnPattern,
    center: Vec2,
    player_pos: Vec2,
    time: f32,
    pattern_registry: &SpawnPatternRegistry,
    loaded_mods: &mut LoadedMods,
) -> Vec<SpawnPoint> {
    let (id, params_map) = spawn_pattern_to_wasm_call(pattern);

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
            .with_scale(Vec3::splat(scale))
            .with_rotation(Quat::from_rotation_z(prototype.rotation.to_radians())),
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

    let offset = point.position - spawn_center;
    let mut builtin_stack = crate::core::danmaku::builtin_motion::BuiltinMotionStack::default();
    let mut wasm_stack = ActiveDanmakuStack::default();

    for behavior in behaviors {
        if crate::core::danmaku::builtin_motion::is_builtin(behavior) {
            // Builtin: initialize state directly in Rust
            if let Some(bs) = crate::core::danmaku::builtin_motion::init_builtin_behavior(
                behavior,
                spawn_center,
                offset,
                player_pos,
            ) {
                builtin_stack.states.push(bs);
            }
        } else {
            // Custom: create WASM instance
            let (id, props) = behavior_to_wasm_call(behavior);
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
            wasm_stack.instances.push(active);
        }
    }

    entity_commands.insert(builtin_stack);
    entity_commands.insert(wasm_stack);
    spawn_bullet_visual(&mut entity_commands, prototype, sprite_params, asset_server);
}

fn spawn_bullet_visual(
    entity_commands: &mut EntityCommands,
    prototype: &BulletPrototype,
    sprite_params: &mut SpriteParams,
    asset_server: &AssetServer,
) {
    let config = load_config();
    let visual_path = prototype.visual.as_str();
    let effective_color = color_tint_to_color(&prototype.color_tint);
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

            // Extract the module-relative path from the full asset_path.
            // asset_path format: "assets/textures/{module}/{relative_path}"
            let relative_path = extract_module_relative_path(&asset_path);

            let mut sprite_context = sprite_params.create_sprite_context();
            let clip_result = SpriteAnimationClip::new(
                &mut sprite_context,
                "battle",
                &relative_path,
                flip_x,
                flip_y,
                true,
                frame_duration,
            )
            .or_else(|_| {
                SpriteAnimationClip::new(
                    &mut sprite_context,
                    "common",
                    &relative_path,
                    flip_x,
                    flip_y,
                    true,
                    frame_duration,
                )
            })
            .or_else(|_| {
                SpriteAnimationClip::new(
                    &mut sprite_context,
                    "overworld",
                    &relative_path,
                    flip_x,
                    flip_y,
                    true,
                    frame_duration,
                )
            });

            if let Ok(clip) = clip_result {
                entity_commands.insert((sprite, clip, SpriteAnimationTimer::new(frame_duration)));
            } else {
                let fallback_path = format!("{}/0.png", asset_path);
                sprite.image = asset_server.load(&fallback_path);
                warn!(
                    "Animation '{}' not found in atlas, using static fallback",
                    relative_path
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

/// Extract module-relative path from a full asset path.
///
/// `"assets/textures/battle/danmaku/spear"` → `"danmaku/spear"`
fn extract_module_relative_path(asset_path: &str) -> String {
    let normalized = asset_path.replace('\\', "/");
    if let Some(rest) = normalized.strip_prefix("assets/textures/")
        && let Some(slash_pos) = rest.find('/')
    {
        return rest[slash_pos + 1..].to_string();
    }
    normalized
}

fn apply_color_tint(sprite: &mut Sprite, color: Option<Color>) {
    if let Some(color) = color {
        sprite.color = color;
    }
}
