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
    let mut times = Vec::with_capacity(timeline.len());
    let mut accumulated = 0.0;

    for event in timeline {
        accumulated = if event.absolute {
            event.t
        } else {
            accumulated + event.t
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
    let mut stack = ActiveDanmakuStack::default();

    for behavior in behaviors {
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
        stack.instances.push(active);
    }

    entity_commands.insert(stack);
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

fn apply_color_tint(sprite: &mut Sprite, color: Option<Color>) {
    if let Some(color) = color {
        sprite.color = color;
    }
}
