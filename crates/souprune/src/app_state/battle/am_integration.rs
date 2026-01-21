//! # am_integration.rs
//!
//! # AM 动画集成模块
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module integrates Alight Motion animations into the battle system.
//! It handles loading AM projects, spawning entities, and adding collision
//! components based on layer naming conventions.
//!
//! 此模块将 Alight Motion 动画集成到战斗系统中。
//! 它处理加载 AM 项目、生成实体，以及根据图层命名约定添加碰撞组件。
//!
//! ## Layer Naming Conventions / 图层命名约定
//!
//! - Layers matching `bullet_pattern` (default: `^#B`): Bullets with collision
//!   匹配 `bullet_pattern` 的图层（默认：`^#B`）：带碰撞的弹幕
//!
//! - Layers matching `battlebox_pattern` (default: `^#C`): Battle box boundary
//!   匹配 `battlebox_pattern` 的图层（默认：`^#C`）：战斗框边界
//!
//! - If a group layer matches, all children inherit the same behavior
//!   如果编组图层匹配，所有子元素继承相同行为

use bevy::prelude::*;
use bevy_alight_motion::prelude::*;
use regex::Regex;

use crate::app_state::battle::collision::{AmBattleBoxBounds, BattleBox};
use crate::app_state::battle::BattleEntity;
use crate::core::collision::TriggerCollider;
use crate::core::danmaku::{
    Bullet, BulletDamage, BulletHitBehavior, BulletLastHitTime, BulletMotionState,
};

/// Marker component for AM performance entities.
/// Used to identify and clean up AM-generated entities.
///
/// AM 演出实体的标记组件。
/// 用于识别和清理 AM 生成的实体。
#[derive(Component, Debug, Clone, Default)]
pub struct AmBattleEntity;

/// Marker for entities that should be treated as bullets (from #B group)
/// Inherited from parent group if parent has this marker.
#[derive(Component, Debug, Clone, Default)]
pub struct AmBulletMarker;

/// Marker for entities that should be treated as battle box (from #C group)
/// Inherited from parent group if parent has this marker.
#[derive(Component, Debug, Clone, Default)]
pub struct AmBattleBoxMarker;

/// Marker for entities that need collision setup in the next frame.
/// This allows GlobalTransform to propagate before calculating collision size.
#[derive(Component, Debug, Clone, Default)]
pub struct NeedsCollisionSetup;

/// Configuration for AM battle integration.
/// Place this in your mod's `battle/am_config.ron` file.
///
/// AM 战斗集成配置。
/// 将此配置放在 mod 的 `battle/am_config.ron` 文件中。
///
/// # Example RON file:
/// ```ron
/// (
///     scale: 2.0,
///     offset: (0.0, -50.0),
///     bullet_pattern: "^#B",
///     battlebox_pattern: "^#C",
///     bullet_damage: 1.0,
///     collision_scale: 0.1,  // Scale down collision boxes to 10% of sprite size
/// )
/// ```
#[derive(Resource, Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AmBattleConfig {
    /// Scale multiplier for AM project (relative to base scale of 1.0/resolution_scale)
    /// Default: 1.0 (no additional scaling)
    ///
    /// AM 项目的缩放倍数（相对于基础缩放 1.0/resolution_scale）
    /// 默认：1.0（无额外缩放）
    #[serde(default = "default_scale")]
    pub scale: f32,
    
    /// Offset position for AM project (x, y)
    /// Default: (0.0, 0.0)
    ///
    /// AM 项目的偏移位置 (x, y)
    /// 默认：(0.0, 0.0)
    #[serde(default = "default_offset")]
    pub offset: (f32, f32),
    
    /// Regex pattern for bullet layers (default: "^#B")
    /// Layers with names matching this pattern are treated as bullets.
    /// If a group matches, all children inherit bullet behavior.
    ///
    /// 弹幕图层的正则表达式模式（默认："^#B"）
    /// 名称匹配此模式的图层被视为弹幕。
    /// 如果编组匹配，所有子元素继承弹幕行为。
    #[serde(default = "default_bullet_pattern")]
    pub bullet_pattern: String,
    
    /// Regex pattern for battle box layers (default: "^#C")
    /// Layers with names matching this pattern are treated as battle box boundaries.
    /// If a group matches, all children inherit battle box behavior.
    ///
    /// 战斗框图层的正则表达式模式（默认："^#C"）
    /// 名称匹配此模式的图层被视为战斗框边界。
    /// 如果编组匹配，所有子元素继承战斗框行为。
    #[serde(default = "default_battlebox_pattern")]
    pub battlebox_pattern: String,
    
    /// Damage dealt by bullets (default: 1.0)
    ///
    /// 弹幕造成的伤害（默认：1.0）
    #[serde(default = "default_bullet_damage")]
    pub bullet_damage: f32,
    
    /// Scale factor for bullet collision boxes relative to sprite size (default: 0.05)
    /// Since AM sprites often have large transparent areas, this scales down
    /// the collision box to better match the actual visible content.
    /// For example, 0.05 means collision box is 5% of the sprite size.
    ///
    /// 弹幕碰撞体相对于精灵大小的缩放因子（默认：0.05）
    /// 由于 AM 精灵通常有大面积透明区域，这个参数用于缩小
    /// 碰撞体以更好地匹配实际可见内容。
    /// 例如，0.05 表示碰撞体是精灵大小的 5%。
    #[serde(default = "default_collision_scale")]
    pub collision_scale: f32,
}

fn default_scale() -> f32 {
    1.0
}

fn default_offset() -> (f32, f32) {
    (0.0, 0.0)
}

fn default_bullet_pattern() -> String {
    "^#B".to_string()
}

fn default_battlebox_pattern() -> String {
    "^#C".to_string()
}

fn default_bullet_damage() -> f32 {
    1.0
}

fn default_collision_scale() -> f32 {
    0.05 // Default to 5% of sprite size since AM sprites often have large transparent areas
}

impl Default for AmBattleConfig {
    fn default() -> Self {
        Self {
            scale: 1.0,
            offset: (0.0, 0.0),
            bullet_pattern: default_bullet_pattern(),
            battlebox_pattern: default_battlebox_pattern(),
            bullet_damage: default_bullet_damage(),
            collision_scale: default_collision_scale(),
        }
    }
}

/// Compiled regex patterns for runtime matching
#[derive(Resource)]
pub struct AmBattlePatterns {
    pub bullet_regex: Option<Regex>,
    pub battlebox_regex: Option<Regex>,
}

/// Resource to track active AM performance state.
///
/// 追踪活跃 AM 演出状态的资源。
#[derive(Resource, Default)]
pub struct AmPerformanceState {
    /// Whether an AM performance is currently playing
    pub is_playing: bool,
    /// Total duration of the performance in milliseconds
    pub total_duration_ms: f32,
    /// Entity ID of the AM project root (if any)
    pub project_entity: Option<Entity>,
    /// The final scale applied to the AM project (base_scale * config.scale)
    /// Used for collision calculations
    pub final_scale: f32,
}

/// Event to request starting an AM performance.
///
/// 请求开始 AM 演出的事件。
#[derive(bevy::ecs::message::Message, Debug, Clone)]
pub struct PlayAmPerformanceEvent {
    pub amproj_path: String,
    pub wait_for_completion: bool,
}

impl PlayAmPerformanceEvent {
    pub fn new(amproj_path: String) -> Self {
        Self {
            amproj_path,
            wait_for_completion: true,
        }
    }
}

/// Plugin for AM battle integration.
///
/// AM 战斗集成插件。
pub struct AmBattlePlugin;

impl Plugin for AmBattlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AmPerformanceState>()
            .init_resource::<AmBattleConfig>()
            .add_message::<PlayAmPerformanceEvent>()
            .add_systems(
                OnEnter(crate::app_state::AppState::Battle),
                load_am_battle_config,
            )
            .add_systems(
                Update,
                (
                    handle_play_am_performance_event,
                    propagate_am_markers_system,
                    // Apply commands before checking markers for collision
                    ApplyDeferred,
                    add_am_collision_system,
                    check_am_performance_completion,
                    debug_am_entities,
                )
                    .chain()
                    .in_set(crate::app_state::battle::BattleUpdate),
            )
            .add_systems(
                OnExit(crate::app_state::AppState::Battle),
                cleanup_am_entities,
            );
    }
}

/// System to load AM battle config from the mod's battle directory.
///
/// 从 mod 的 battle 目录加载 AM 战斗配置。
fn load_am_battle_config(
    mut commands: Commands,
    mut am_config: ResMut<AmBattleConfig>,
    game_config: Res<crate::config::GameConfig>,
) {
    let config_path = format!(
        "projects/{}/battle/am_config.ron",
        game_config.mod_name
    );
    
    match std::fs::read_to_string(&config_path) {
        Ok(content) => {
            match ron::from_str::<AmBattleConfig>(&content) {
                Ok(config) => {
                    *am_config = config;
                    info!(
                        "[AM Battle] Loaded config from {}: scale={}, offset={:?}, bullet_pattern='{}', battlebox_pattern='{}', damage={}",
                        config_path, am_config.scale, am_config.offset, 
                        am_config.bullet_pattern, am_config.battlebox_pattern, am_config.bullet_damage
                    );
                }
                Err(e) => {
                    warn!(
                        "[AM Battle] Failed to parse {}: {}. Using defaults.",
                        config_path, e
                    );
                }
            }
        }
        Err(e) => {
            info!(
                "[AM Battle] Config file {} not found ({}). Using defaults: scale={}, offset={:?}",
                config_path, e, am_config.scale, am_config.offset
            );
        }
    }
    
    // Compile regex patterns
    let bullet_regex = match Regex::new(&am_config.bullet_pattern) {
        Ok(r) => {
            info!("[AM Battle] Compiled bullet regex: '{}'", am_config.bullet_pattern);
            Some(r)
        }
        Err(e) => {
            warn!("[AM Battle] Invalid bullet pattern '{}': {}", am_config.bullet_pattern, e);
            None
        }
    };
    
    let battlebox_regex = match Regex::new(&am_config.battlebox_pattern) {
        Ok(r) => {
            info!("[AM Battle] Compiled battlebox regex: '{}'", am_config.battlebox_pattern);
            Some(r)
        }
        Err(e) => {
            warn!("[AM Battle] Invalid battlebox pattern '{}': {}", am_config.battlebox_pattern, e);
            None
        }
    };
    
    commands.insert_resource(AmBattlePatterns {
        bullet_regex,
        battlebox_regex,
    });
}

/// Observer function that handles AmEntitySpawned events.
/// Adds marker components based on layer naming conventions.
/// Collision components are added later by propagate_am_markers_system.
///
/// 处理 AmEntitySpawned 事件的观察者函数。
/// 根据图层命名约定添加标记组件。
/// 碰撞组件由 propagate_am_markers_system 稍后添加。
pub fn on_am_entity_spawned(
    trigger: Trigger<AmEntitySpawned>,
    mut commands: Commands,
    patterns: Option<Res<AmBattlePatterns>>,
) {
    let event = trigger.event();
    let layer_name = &event.layer_name;

    info!(
        "[AM Battle] Entity spawned: '{}' (type={:?})",
        layer_name, event.element_type
    );

    // Add AmBattleEntity marker to all AM entities
    commands.entity(event.entity).insert(AmBattleEntity);

    // Check regex patterns for bullet/battlebox markers
    if let Some(patterns) = patterns {
        // Check bullet pattern
        if let Some(ref regex) = patterns.bullet_regex {
            if regex.is_match(layer_name) {
                commands.entity(event.entity).insert(AmBulletMarker);
                info!("  → Matched bullet pattern, added AmBulletMarker to '{}'", layer_name);
            }
        }
        
        // Check battlebox pattern
        if let Some(ref regex) = patterns.battlebox_regex {
            if regex.is_match(layer_name) {
                commands.entity(event.entity).insert(AmBattleBoxMarker);
                info!("  → Matched battlebox pattern, added AmBattleBoxMarker to '{}'", layer_name);
            }
        }
    }
}

/// System to propagate AM markers from parent groups to children.
///
/// 将 AM 标记从父编组传播到子元素。
fn propagate_am_markers_system(
    mut commands: Commands,
    // All AM entities that might need marker inheritance
    am_entities: Query<(Entity, Option<&AmBulletMarker>, Option<&AmBattleBoxMarker>), With<AmBattleEntity>>,
    // Parent hierarchy for inheritance
    parent_query: Query<&ChildOf>,
) {
    // Propagate markers from parents to children
    for (entity, bullet_marker, battlebox_marker) in am_entities.iter() {
        // If already has markers, skip
        if bullet_marker.is_some() || battlebox_marker.is_some() {
            continue;
        }
        
        // Check parent chain for markers
        let mut current = entity;
        let mut inherited_bullet = false;
        let mut inherited_battlebox = false;
        
        while let Ok(child_of) = parent_query.get(current) {
            let parent = child_of.parent();
            
            // Check if parent has bullet marker
            if let Ok((_, parent_bullet, parent_battlebox)) = am_entities.get(parent) {
                if parent_bullet.is_some() {
                    inherited_bullet = true;
                }
                if parent_battlebox.is_some() {
                    inherited_battlebox = true;
                }
            }
            
            if inherited_bullet || inherited_battlebox {
                break;
            }
            
            current = parent;
        }
        
        // Apply inherited markers
        if inherited_bullet {
            commands.entity(entity).insert(AmBulletMarker);
            info!("[AM Battle] Inherited AmBulletMarker to entity {:?}", entity);
        }
        if inherited_battlebox {
            commands.entity(entity).insert(AmBattleBoxMarker);
            info!("[AM Battle] Inherited AmBattleBoxMarker to entity {:?}", entity);
        }
    }
}

/// System to add collision components to marked AM entities.
/// Runs after `propagate_am_markers_system` and `apply_deferred`.
///
/// 为标记的 AM 实体添加碰撞组件。
/// 在 `propagate_am_markers_system` 和 `apply_deferred` 之后运行。
fn add_am_collision_system(
    mut commands: Commands,
    am_config: Res<AmBattleConfig>,
    am_state: Res<AmPerformanceState>,
    // Entities with bullet marker that need collision (newly added)
    bullet_marker_query: Query<Entity, (With<AmBulletMarker>, Without<Bullet>)>,
    // Entities with battlebox marker that need components (newly added)
    battlebox_marker_query: Query<Entity, (With<AmBattleBoxMarker>, Without<BattleBox>)>,
    // AmLayerSpec query for collision size (contains actual layer dimensions)
    layer_spec_query: Query<&AmLayerSpec>,
    // AmAnimated query for layer's animated scale
    animated_query: Query<&AmAnimated>,
    // Parent query to traverse hierarchy
    parent_query: Query<&ChildOf>,
    // Visibility query for hiding bullet layers
    mut visibility_query: Query<&mut Visibility>,
) {
    // Helper function to check if layer spec is a visual element that should have collision
    fn is_visual_element(spec: &AmLayerSpec) -> bool {
        matches!(
            spec,
            AmLayerSpec::SpriteShape { .. }
                | AmLayerSpec::SdfShape { .. }
                | AmLayerSpec::Image { .. }
                | AmLayerSpec::Text { .. }
        )
    }
    
    // Helper function to get size from AmLayerSpec (SDF shapes have actual dimensions)
    fn get_layer_size(spec: &AmLayerSpec) -> Option<(f32, f32)> {
        match spec {
            AmLayerSpec::SpriteShape { width, height, .. } => Some((*width, *height)),
            AmLayerSpec::SdfShape { width, height, .. } => Some((*width, *height)),
            AmLayerSpec::Image { width, height, .. } => Some((*width, *height)),
            AmLayerSpec::Text { .. } | AmLayerSpec::Null | AmLayerSpec::EmbedScene => None,
        }
    }
    
    // Helper function to get initial scale from AmAnimated.scale
    fn get_animated_scale(animated: &AmAnimated) -> Vec2 {
        // First try static value
        if let Some(val) = &animated.scale.value {
            return Vec2::new(val[0].abs(), val[1].abs());
        }
        // Then try first keyframe
        if let Some(kf) = animated.scale.keyframes.first() {
            // Parse "x,y" format
            let parts: Vec<&str> = kf.value.split(',').collect();
            if parts.len() == 2 {
                if let (Ok(x), Ok(y)) = (parts[0].trim().parse::<f32>(), parts[1].trim().parse::<f32>()) {
                    return Vec2::new(x.abs(), y.abs());
                }
            }
        }
        // Default to 1.0
        Vec2::ONE
    }
    
    // Helper function to compute total scale by traversing parent hierarchy
    fn compute_total_scale(
        entity: Entity,
        animated_query: &Query<&AmAnimated>,
        parent_query: &Query<&ChildOf>,
        final_scale: f32,
    ) -> Vec2 {
        let mut total_scale = Vec2::splat(final_scale);
        let mut current = entity;
        
        // Traverse up the hierarchy
        loop {
            // Get this entity's own scale
            if let Ok(animated) = animated_query.get(current) {
                let scale = get_animated_scale(animated);
                total_scale *= scale;
            }
            
            // Move to parent
            if let Ok(child_of) = parent_query.get(current) {
                current = child_of.0;
            } else {
                break;
            }
        }
        
        total_scale
    }
    
    // Add collision components to bullet-marked entities
    // Only add collision to actual visual elements, not groups (Null/EmbedScene)
    // Now using SDF shape dimensions directly from AmLayerSpec
    for entity in bullet_marker_query.iter() {
        // Check if this is a visual element and get size from AmLayerSpec
        let (width, height) = if let Ok(spec) = layer_spec_query.get(entity) {
            if let Some((w, h)) = get_layer_size(spec) {
                info!(
                    "[AM Battle] Entity {:?} layer spec size: {}x{} (spec={:?})",
                    entity, w, h, spec
                );
                (w, h)
            } else {
                info!(
                    "[AM Battle] SKIPPING entity {:?} - not a visual element (spec={:?})",
                    entity, spec
                );
                continue; // Skip non-visual elements
            }
        } else {
            info!(
                "[AM Battle] SKIPPING entity {:?} - no AmLayerSpec",
                entity
            );
            continue;
        };
        
        // Compute total scale by traversing parent hierarchy
        // This includes: layer's own scale + all parent scales + project root scale (final_scale)
        let total_scale = compute_total_scale(entity, &animated_query, &parent_query, am_state.final_scale);
        
        // Calculate final collision half_size (size * total_scale / 2)
        let half_size = Vec2::new(
            width * total_scale.x / 2.0,
            height * total_scale.y / 2.0,
        );

        commands.entity(entity).insert((
            Bullet,
            TriggerCollider::Box { half_size },
            BulletDamage(am_config.bullet_damage),
            // AM bullets use no invincibility_duration since they're animated
            // and motion_state.elapsed doesn't track their real age
            BulletHitBehavior {
                despawn_on_hit: false,
                damage_on_player_moving: false,
                damage_on_player_stationary: false,
                invincibility_duration: 0.0, // Disable bullet i-frames for AM bullets
            },
            BulletLastHitTime::default(),
            BulletMotionState::new(Vec2::ZERO),
        ));
        
        // TODO: Temporarily disabled for debugging
        // Hide the bullet layer (set visibility to Hidden)
        // if let Ok(mut visibility) = visibility_query.get_mut(entity) {
        //     *visibility = Visibility::Hidden;
        //     info!(
        //         "[AM Battle] Hidden bullet entity {:?}",
        //         entity
        //     );
        // }

        info!(
            "[AM Battle] ADDED COLLISION to entity {:?} (half_size={:?}, size=({:.1}x{:.1}), total_scale={:?}, damage={})",
            entity, half_size, width, height, total_scale, am_config.bullet_damage
        );
    }
    
    // Add battle box components to battlebox-marked entities
    for entity in battlebox_marker_query.iter() {
        // Check if this is a visual element (skip groups)
        let (is_visual, _spec_debug) = if let Ok(spec) = layer_spec_query.get(entity) {
            (is_visual_element(spec), format!("{:?}", spec))
        } else {
            (false, "No AmLayerSpec".to_string())
        };
        
        if !is_visual {
            continue;
        }
        
        // Compute total scale by traversing parent hierarchy
        let total_scale = compute_total_scale(entity, &animated_query, &parent_query, am_state.final_scale);
        
        // Get size from AmLayerSpec with total_scale
        let (width, height) = if let Ok(spec) = layer_spec_query.get(entity) {
            if let Some((w, h)) = get_layer_size(spec) {
                (w.abs() * total_scale.x, h.abs() * total_scale.y)
            } else {
                (565.0, 140.0)
            }
        } else {
            (565.0, 140.0)
        };

        commands.entity(entity).insert((
            BattleBox,
            AmBattleBoxBounds { width, height },
        ));

        info!(
            "[AM Battle] Added BattleBox to entity {:?} (size={}x{}, total_scale={:?})",
            entity, width, height, total_scale
        );
    }
}

/// System to handle PlayAmPerformanceEvent.
///
/// 处理 PlayAmPerformanceEvent 的系统。
fn handle_play_am_performance_event(
    mut commands: Commands,
    mut events: bevy::ecs::message::MessageReader<PlayAmPerformanceEvent>,
    mut am_state: ResMut<AmPerformanceState>,
    asset_server: Res<AssetServer>,
    resolution_scale: Res<crate::app_state::app_setup::ResolutionScale>,
    am_config: Res<AmBattleConfig>,
) {
    for event in events.read() {
        info!("[AM Battle] Starting performance: {}", event.amproj_path);

        // Load the AM project
        let entity = load_am_project(&mut commands, &asset_server, &event.amproj_path);

        // Calculate scale to fit the AM project into the camera view
        // Camera scale = 1.0 / resolution_scale, so visible area = window_size * camera_scale
        // AM project needs to be scaled by the same factor as the camera
        // Then apply additional scale from config
        let base_scale = 1.0 / resolution_scale.get() as f32;
        let final_scale = base_scale * am_config.scale;
        
        // Apply offset from config (scaled by base_scale to work in screen coordinates)
        let offset = Vec3::new(
            am_config.offset.0 * base_scale,
            am_config.offset.1 * base_scale,
            0.0,
        );
        
        // Mark as battle entity and apply scale and offset
        commands.entity(entity).insert((
            BattleEntity,
            Transform {
                translation: offset,
                scale: Vec3::splat(final_scale),
                ..Default::default()
            },
        ));
        
        info!(
            "[AM Battle] Performance started, entity: {:?}, base_scale: {}, config_scale: {}, final_scale: {}, offset: {:?}",
            entity, base_scale, am_config.scale, final_scale, am_config.offset
        );

        // Register the observer for this project's spawned entities
        commands.add_observer(on_am_entity_spawned);

        // Update state
        am_state.is_playing = true;
        am_state.project_entity = Some(entity);
        am_state.final_scale = final_scale;
    }
}

/// System to check if AM performance has completed.
///
/// 检查 AM 演出是否完成的系统。
fn check_am_performance_completion(
    playback: Option<Res<AmPlayback>>,
    mut am_state: ResMut<AmPerformanceState>,
    am_roots: Query<(Entity, &Name, &AmProjectRoot, &GlobalTransform), With<AmProjectRoot>>,
) {
    // Debug: Log all AM project roots
    for (entity, name, root, transform) in am_roots.iter() {
        info!(
            "[AM Battle Debug] Project root: {:?} '{}' spawned={} pos={:?}",
            entity,
            name,
            root.spawned,
            transform.translation()
        );
    }

    if !am_state.is_playing {
        return;
    }

    // Check if playback exists and has finished
    if let Some(playback) = playback {
        let total_duration = playback.total_time_ms;
        am_state.total_duration_ms = total_duration;

        // Check if animation has finished
        if playback.current_time_ms >= total_duration {
            info!(
                "[AM Battle] Performance completed ({}ms / {}ms)",
                playback.current_time_ms, total_duration
            );
            am_state.is_playing = false;
        }
    }
}

/// System to cleanup AM entities when exiting battle.
///
/// 退出战斗时清理 AM 实体的系统。
fn cleanup_am_entities(
    mut commands: Commands,
    query: Query<Entity, With<AmBattleEntity>>,
    mut am_state: ResMut<AmPerformanceState>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    am_state.is_playing = false;
    am_state.project_entity = None;

    info!("[AM Battle] Cleaned up AM entities");
}

/// Debug system to log AM entity properties once after spawning.
///
/// 调试系统：在生成后记录一次 AM 实体属性。
fn debug_am_entities(
    query: Query<
        (
            Entity,
            &Name,
            &GlobalTransform,
            Option<&Visibility>,
            Option<&InheritedVisibility>,
            Option<&Sprite>,
        ),
        (With<AmBattleEntity>, Added<AmBattleEntity>),
    >,
) {
    for (entity, name, global_transform, visibility, inherited_vis, sprite) in query.iter() {
        let translation = global_transform.translation();
        let scale = global_transform.to_scale_rotation_translation().0;
        
        let vis_str = match visibility {
            Some(Visibility::Inherited) => "Inherited",
            Some(Visibility::Visible) => "Visible",
            Some(Visibility::Hidden) => "Hidden",
            None => "None",
        };
        
        let inherited_vis_str = match inherited_vis {
            Some(v) if v.get() => "true",
            Some(_) => "false",
            None => "None",
        };
        
        let sprite_info = if let Some(s) = sprite {
            format!(
                "rect={:?}, custom_size={:?}, color={:?}",
                s.rect, s.custom_size, s.color
            )
        } else {
            "NO SPRITE".to_string()
        };
        
        info!(
            "[AM Debug] Entity {:?} '{}': pos={:?}, z={}, scale={:?}, vis={}, inherited={}, sprite=[{}]",
            entity,
            name,
            Vec2::new(translation.x, translation.y),
            translation.z,
            scale,
            vis_str,
            inherited_vis_str,
            sprite_info
        );
    }
}
