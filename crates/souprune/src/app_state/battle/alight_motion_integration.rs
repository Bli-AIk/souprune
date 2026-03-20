//!
//! # Alight Motion 动画集成模块
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module integrates Alight Motion animations into the battle system.
//! It handles loading Alight Motion projects, spawning entities, and adding collision
//! components based on layer naming conventions.
//!
//! 此模块将 Alight Motion 动画集成到战斗系统中。
//! 它处理加载 Alight Motion 项目、生成实体，以及根据图层命名约定添加碰撞组件。
//!
//! ## Layer Naming Conventions / 图层命名约定
//!
//! - Layers matching `bullet_pattern` (default: `^#B`): Bullets with collision
//!   匹配 `bullet_pattern` 的图层（默认：`^#B`）：带碰撞的弹幕
//!
//! - Layers matching `battle_box_pattern` (default: `^#C`): Battle box boundary
//!   匹配 `battle_box_pattern` 的图层（默认：`^#C`）：战斗框边界
//!
//! - If a group layer matches, all children inherit the same behavior
//!   如果编组图层匹配，所有子元素继承相同行为

use bevy::prelude::*;
use bevy_alight_motion::prelude::*;
use regex::Regex;

use crate::app_state::battle::battle_scoped;
use crate::core::alight_motion_runtime::{
    AlightMotionPerformanceState, PlayAlightMotionPerformanceEvent,
};
use crate::core::battle_box::{
    AlightMotionBattleBoxBounds, BattleBox, BattleBoxId, BattleBoxState, BattleBoxVisualStyle,
};
use crate::core::collision::TriggerCollider;
use crate::core::danmaku::{
    Bullet, BulletDamage, BulletHitBehavior, BulletLastHitTime, BulletMotionState,
};

/// Marker component for Alight Motion performance entities.
/// Used to identify and clean up Alight Motion-generated entities.
///
/// Alight Motion 演出实体的标记组件。
/// 用于识别和清理 Alight Motion 生成的实体。
#[derive(Component, Debug, Clone, Default)]
pub struct AlightMotionEntity;

/// Marker for entities that should be treated as bullets (from #B group)
/// Inherited from parent group if parent has this marker.
#[derive(Component, Debug, Clone, Default)]
pub struct AlightMotionBulletMarker;

/// Marker for entities that should be treated as battle box (from #C group)
/// Inherited from parent group if parent has this marker.
#[derive(Component, Debug, Clone, Default)]
pub struct AlightMotionBattleBoxMarker;

/// Marker for entities that should be hidden (based on hidden_pattern config)
/// Inherited from parent group if parent has this marker.
#[derive(Component, Debug, Clone, Default)]
pub struct AlightMotionHiddenMarker;

/// Configuration for Alight Motion battle integration.
/// Place this in your mod's `battle/alight_motion_config.ron` file.
///
/// Alight Motion 战斗集成配置。
/// 将此配置放在 mod 的 `battle/alight_motion_config.ron` 文件中。
///
/// # Example RON file:
/// ```ron
/// (
///     scale: 2.0,
///     offset: (0.0, -50.0),
///     bullet_pattern: "^#B",
///     battle_box_pattern: "^#C",
///     bullet_damage: 1.0,
///     collision_scale: 0.1,  // Scale down collision boxes to 10% of sprite size
/// )
/// ```
#[derive(Resource, Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AlightMotionBattleConfig {
    /// Scale multiplier for Alight Motion project (relative to base scale of 1.0/resolution_scale)
    /// Default: 1.0 (no additional scaling)
    ///
    /// Alight Motion 项目的缩放倍数（相对于基础缩放 1.0/resolution_scale）
    /// 默认：1.0（无额外缩放）
    pub scale: f32,

    /// Offset position for Alight Motion project (x, y)
    /// Default: (0.0, 0.0)
    ///
    /// Alight Motion 项目的偏移位置 (x, y)
    /// 默认：(0.0, 0.0)
    pub offset: (f32, f32),

    /// Regex pattern for bullet layers (default: "^#B")
    /// Layers with names matching this pattern are treated as bullets.
    /// If a group matches, all children inherit bullet behavior.
    ///
    /// 弹幕图层的正则表达式模式（默认："^#B"）
    /// 名称匹配此模式的图层被视为弹幕。
    /// 如果编组匹配，所有子元素继承弹幕行为。
    pub bullet_pattern: String,

    /// Regex pattern for battle box layers (default: "^#C")
    /// Layers with names matching this pattern are treated as battle box boundaries.
    /// If a group matches, all children inherit battle box behavior.
    ///
    /// 战斗框图层的正则表达式模式（默认："^#C"）
    /// 名称匹配此模式的图层被视为战斗框边界。
    /// 如果编组匹配，所有子元素继承战斗框行为。
    pub battle_box_pattern: String,

    /// Regex pattern for layers that should be hidden (default: empty = hide nothing)
    /// Layers with names matching this pattern will have their visibility set to Hidden.
    /// This is useful for hiding collision marker layers that shouldn't be rendered.
    ///
    /// 应该隐藏的图层的正则表达式模式（默认：空 = 不隐藏任何内容）
    /// 名称匹配此模式的图层将被设置为隐藏。
    /// 这对于隐藏不应该渲染的碰撞标记图层很有用。
    pub hidden_pattern: String,

    /// Damage dealt by bullets (default: 1.0)
    ///
    /// 弹幕造成的伤害（默认：1.0）
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
    pub collision_scale: f32,

    /// Default battle box size (width, height) when size cannot be determined from AM layer.
    ///
    /// 当无法从 AM 图层确定大小时使用的默认战斗箱尺寸（宽, 高）。
    #[serde(default = "default_battle_box_size")]
    pub default_battle_box_size: (f32, f32),
}

fn default_battle_box_size() -> (f32, f32) {
    (565.0, 140.0)
}

impl Default for AlightMotionBattleConfig {
    fn default() -> Self {
        Self {
            scale: 1.0,
            offset: (0.0, 0.0),
            bullet_pattern: "^#B".to_string(),
            battle_box_pattern: "^#C".to_string(),
            hidden_pattern: String::new(), // Empty = hide nothing by default
            bullet_damage: 1.0,
            collision_scale: 0.05,
            default_battle_box_size: default_battle_box_size(),
        }
    }
}

/// Compiled regex patterns for runtime matching
#[derive(Resource)]
pub struct AlightMotionBattlePatterns {
    pub bullet_regex: Option<Regex>,
    pub battle_box_regex: Option<Regex>,
    pub hidden_regex: Option<Regex>,
}

/// Plugin for Alight Motion battle integration.
///
/// Alight Motion 战斗集成插件。
pub struct AlightMotionBattlePlugin;

impl Plugin for AlightMotionBattlePlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        app.init_resource::<AlightMotionPerformanceState>()
            .init_resource::<AlightMotionBattleConfig>()
            .add_message::<PlayAlightMotionPerformanceEvent>()
            .add_systems(
                schedule,
                load_am_battle_config.run_if(super::on_entering_battle),
            )
            .add_systems(
                schedule,
                (
                    handle_play_am_performance_event,
                    // Sync fit scale for mask coordinate calculation
                    sync_am_fit_scale_system,
                    // Apply commands so observer results (AlightMotionEntity, AlightMotionHiddenMarker etc.) are available
                    ApplyDeferred,
                    propagate_am_markers_system,
                    // Apply commands before checking markers for collision
                    ApplyDeferred,
                    add_am_collision_system,
                    apply_am_hidden_visibility,
                    // Dynamic update for animated battle boxes
                    update_am_battle_box_bounds_system,
                    check_am_performance_completion,
                )
                    .chain()
                    .in_set(crate::core::battle_runtime::BattleUpdate),
            )
            .add_systems(
                schedule,
                cleanup_am_entities.run_if(super::on_exiting_battle),
            );
    }
}

/// Load and parse Alight Motion battle config from a path.
/// Returns the loaded config and compiled regex patterns.
///
/// 从路径加载并解析 Alight Motion 战斗配置。
/// 返回加载的配置和编译的正则表达式模式。
fn load_alight_motion_config_from_path(
    config_path: &str,
) -> (
    AlightMotionBattleConfig,
    Option<Regex>,
    Option<Regex>,
    Option<Regex>,
) {
    let mut am_config = AlightMotionBattleConfig::default();

    match std::fs::read_to_string(config_path) {
        Ok(content) => match ron::from_str::<AlightMotionBattleConfig>(&content) {
            Ok(config) => {
                am_config = config;
                info!(
                    "[AM Battle] Loaded config from {}: scale={}, offset={:?}, bullet_pattern='{}', battle_box_pattern='{}', damage={}",
                    config_path,
                    am_config.scale,
                    am_config.offset,
                    am_config.bullet_pattern,
                    am_config.battle_box_pattern,
                    am_config.bullet_damage
                );
            }
            Err(e) => {
                warn!(
                    "[AM Battle] Failed to parse {}: {}. Using defaults.",
                    config_path, e
                );
            }
        },
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
            info!(
                "[AM Battle] Compiled bullet regex: '{}'",
                am_config.bullet_pattern
            );
            Some(r)
        }
        Err(e) => {
            warn!(
                "[AM Battle] Invalid bullet pattern '{}': {}",
                am_config.bullet_pattern, e
            );
            None
        }
    };

    let battle_box_regex = match Regex::new(&am_config.battle_box_pattern) {
        Ok(r) => {
            info!(
                "[AM Battle] Compiled battle_box regex: '{}'",
                am_config.battle_box_pattern
            );
            Some(r)
        }
        Err(e) => {
            warn!(
                "[AM Battle] Invalid battle_box pattern '{}': {}",
                am_config.battle_box_pattern, e
            );
            None
        }
    };

    let hidden_regex = if am_config.hidden_pattern.is_empty() {
        None
    } else {
        match Regex::new(&am_config.hidden_pattern) {
            Ok(r) => {
                info!(
                    "[AM Battle] Compiled hidden regex: '{}'",
                    am_config.hidden_pattern
                );
                Some(r)
            }
            Err(e) => {
                warn!(
                    "[AM Battle] Invalid hidden pattern '{}': {}",
                    am_config.hidden_pattern, e
                );
                None
            }
        }
    };

    (am_config, bullet_regex, battle_box_regex, hidden_regex)
}

/// System to load Alight Motion battle config from the mod's battle directory.
///
/// 从 mod 的 battle 目录加载 Alight Motion 战斗配置。
fn load_am_battle_config(
    mut commands: Commands,
    mut am_config: ResMut<AlightMotionBattleConfig>,
    project_config: Res<crate::config::SoupruneConfig>,
) {
    let config_path = format!(
        "{}/{}/states/battle/alight_motion_config.ron",
        crate::config::get_projects_base_path().display(),
        project_config.project.mod_name
    );

    let (config, bullet_regex, battle_box_regex, hidden_regex) =
        load_alight_motion_config_from_path(&config_path);
    *am_config = config;

    commands.insert_resource(AlightMotionBattlePatterns {
        bullet_regex,
        battle_box_regex,
        hidden_regex,
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
    trigger: On<AmEntitySpawned>,
    mut commands: Commands,
    patterns: Option<Res<AlightMotionBattlePatterns>>,
) {
    let event = trigger.event();
    let layer_name = &event.layer_name;

    trace!(
        "[AM Battle] Entity spawned: '{}' (type={:?})",
        layer_name, event.element_type
    );

    // Add AlightMotionEntity marker to all AM entities
    commands.entity(event.entity).insert(AlightMotionEntity);

    // Check regex patterns for bullet/battle_box/hidden markers
    if let Some(patterns) = patterns {
        // Check bullet pattern
        if let Some(ref regex) = patterns.bullet_regex
            && regex.is_match(layer_name)
        {
            commands
                .entity(event.entity)
                .insert(AlightMotionBulletMarker);
            trace!(
                "  → Matched bullet pattern, added AlightMotionBulletMarker to '{}'",
                layer_name
            );
        }

        // Check battle_box pattern
        if let Some(ref regex) = patterns.battle_box_regex
            && regex.is_match(layer_name)
        {
            commands
                .entity(event.entity)
                .insert(AlightMotionBattleBoxMarker);
            trace!(
                "  → Matched battle_box pattern, added AlightMotionBattleBoxMarker to '{}'",
                layer_name
            );
        }

        // Check hidden pattern - mark layers matching this pattern for hiding
        if let Some(ref regex) = patterns.hidden_regex
            && regex.is_match(layer_name)
        {
            // Add both AlightMotionHiddenMarker (for propagation) and AmForceHidden (for AM library)
            commands.entity(event.entity).insert((
                AlightMotionHiddenMarker,
                AmForceHidden, // Tell bevy_alight_motion to keep this hidden
                Visibility::Hidden,
            ));
            // info!(
            //     "  → Matched hidden pattern, added AlightMotionHiddenMarker + AmForceHidden to '{}'",
            //     layer_name
            // );
        }
    }
}

/// System to propagate AM markers from parent groups to children.
///
/// 将 AM 标记从父编组传播到子元素。
fn propagate_am_markers_system(
    mut commands: Commands,
    // All AM entities that might need marker inheritance
    am_entities: Query<
        (
            Entity,
            Option<&AlightMotionBulletMarker>,
            Option<&AlightMotionBattleBoxMarker>,
            Option<&AlightMotionHiddenMarker>,
        ),
        With<AlightMotionEntity>,
    >,
    // Parent hierarchy for inheritance
    parent_query: Query<&ChildOf>,
) {
    // Propagate markers from parents to children
    for (entity, bullet_marker, battle_box_marker, hidden_marker) in am_entities.iter() {
        // Check if already has all markers - can skip
        let has_bullet = bullet_marker.is_some();
        let has_battle_box = battle_box_marker.is_some();
        let has_hidden = hidden_marker.is_some();

        // If already has all markers we care about, skip
        if has_bullet && has_battle_box && has_hidden {
            continue;
        }

        // Check parent chain for markers
        let mut current = entity;
        let mut inherited_bullet = false;
        let mut inherited_battle_box = false;
        let mut inherited_hidden = false;

        while let Ok(child_of) = parent_query.get(current) {
            let parent = child_of.parent();

            // Check if parent has markers
            let Ok((_, parent_bullet, parent_battle_box, parent_hidden)) = am_entities.get(parent)
            else {
                current = parent;
                continue;
            };

            if !has_bullet && parent_bullet.is_some() {
                inherited_bullet = true;
            }
            if !has_battle_box && parent_battle_box.is_some() {
                inherited_battle_box = true;
            }
            if !has_hidden && parent_hidden.is_some() {
                inherited_hidden = true;
            }

            // If found all needed inheritance, stop
            if (has_bullet || inherited_bullet)
                && (has_battle_box || inherited_battle_box)
                && (has_hidden || inherited_hidden)
            {
                break;
            }

            current = parent;
        }

        // Apply inherited markers
        if inherited_bullet {
            commands.entity(entity).insert(AlightMotionBulletMarker);
            info!(
                "[AM Battle] Inherited AlightMotionBulletMarker to entity {:?}",
                entity
            );
        }
        if inherited_battle_box {
            commands.entity(entity).insert(AlightMotionBattleBoxMarker);
            info!(
                "[AM Battle] Inherited AlightMotionBattleBoxMarker to entity {:?}",
                entity
            );
        }
        if inherited_hidden {
            // Add both AlightMotionHiddenMarker (for tracking) and AmForceHidden (for AM library)
            commands.entity(entity).insert((
                AlightMotionHiddenMarker,
                AmForceHidden, // Tell bevy_alight_motion to keep this hidden
                Visibility::Hidden,
            ));
            info!(
                "[AM Battle] Inherited AlightMotionHiddenMarker + AmForceHidden to entity {:?}",
                entity
            );
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
    am_config: Res<AlightMotionBattleConfig>,
    am_state: Res<AlightMotionPerformanceState>,
    // Entities with bullet marker that need collision (newly added)
    bullet_marker_query: Query<Entity, (With<AlightMotionBulletMarker>, Without<Bullet>)>,
    // Entities with battle_box marker that need components (newly added)
    battle_box_marker_query: Query<Entity, (With<AlightMotionBattleBoxMarker>, Without<BattleBox>)>,
    // AmLayerSpec query for collision size (contains actual layer dimensions)
    layer_spec_query: Query<&AmLayerSpec>,
    // AmAnimated query for layer's animated scale
    animated_query: Query<&AmAnimated>,
    // Parent query to traverse hierarchy
    parent_query: Query<&ChildOf>,
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
            AmLayerSpec::Text { .. }
            | AmLayerSpec::Null
            | AmLayerSpec::EmbedScene
            | AmLayerSpec::Camera { .. } => None,
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
            if parts.len() == 2
                && let (Ok(x), Ok(y)) = (
                    parts[0].trim().parse::<f32>(),
                    parts[1].trim().parse::<f32>(),
                )
            {
                return Vec2::new(x.abs(), y.abs());
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
            info!("[AM Battle] SKIPPING entity {:?} - no AmLayerSpec", entity);
            continue;
        };

        // Compute total scale by traversing parent hierarchy
        // This includes: layer's own scale + all parent scales + project root scale (final_scale)
        let total_scale =
            compute_total_scale(entity, &animated_query, &parent_query, am_state.final_scale);

        // Calculate final collision half_size (size * total_scale / 2)
        let half_size = Vec2::new(width * total_scale.x / 2.0, height * total_scale.y / 2.0);

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

    // Add battle box components to battle_box-marked entities
    for entity in battle_box_marker_query.iter() {
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
        let total_scale =
            compute_total_scale(entity, &animated_query, &parent_query, am_state.final_scale);

        // Get size from AmLayerSpec with total_scale
        let (width, height) = if let Ok(spec) = layer_spec_query.get(entity) {
            if let Some((w, h)) = get_layer_size(spec) {
                (w.abs() * total_scale.x, h.abs() * total_scale.y)
            } else {
                am_config.default_battle_box_size
            }
        } else {
            am_config.default_battle_box_size
        };

        // Calculate center_offset from anchor_offset
        // anchor_offset moves entity from center to pivot point
        // So center_offset = -anchor_offset to go back to center
        // Also need to apply scale to the offset
        let center_offset = if let Ok(animated) = animated_query.get(entity) {
            -animated.anchor_offset * total_scale
        } else {
            Vec2::ZERO
        };

        commands.entity(entity).insert((
            BattleBox,
            BattleBoxId("main".to_string()),
            BattleBoxState::default(),
            BattleBoxVisualStyle::default(),
            AlightMotionBattleBoxBounds {
                width,
                height,
                center_offset,
            },
        ));

        info!(
            "[AM Battle] Added BattleBox to entity {:?} (size={}x{}, total_scale={:?}, center_offset={:?})",
            entity, width, height, total_scale, center_offset
        );
    }
}

/// System to handle PlayAlightMotionPerformanceEvent.
///
/// 处理 PlayAlightMotionPerformanceEvent 的系统。
fn handle_play_am_performance_event(
    mut commands: Commands,
    mut events: MessageReader<PlayAlightMotionPerformanceEvent>,
    mut am_state: ResMut<AlightMotionPerformanceState>,
    mut am_config: ResMut<AlightMotionBattleConfig>,
    asset_server: Res<AssetServer>,
    project_config: Res<crate::config::SoupruneConfig>,
) {
    for event in events.read() {
        info!("[AM Battle] Starting performance: {}", event.amproj_path);

        // If a custom am_config path is provided, reload the config
        // Otherwise, use the default config loaded at battle start
        if let Some(custom_config_path) = &event.alight_motion_config_path {
            let full_path = format!(
                "{}/{}/{}",
                crate::config::get_projects_base_path().display(),
                project_config.project.mod_name,
                custom_config_path
            );
            info!("[AM Battle] Using custom config: {}", full_path);
            let (config, bullet_regex, battle_box_regex, hidden_regex) =
                load_alight_motion_config_from_path(&full_path);
            *am_config = config;
            commands.insert_resource(AlightMotionBattlePatterns {
                bullet_regex,
                battle_box_regex,
                hidden_regex,
            });
        }

        // Load the Alight Motion project
        let entity = load_am_project(&mut commands, &asset_server, &event.amproj_path);

        // Calculate scale to fit the Alight Motion project into the camera view
        // We use a fixed base scale of 0.25 to match the behavior at resolution_scale=4.
        // This ensures the Alight Motion project size remains constant relative to the game world
        // regardless of the actual window resolution_scale.
        let base_scale = 0.25;
        let final_scale = base_scale * am_config.scale;

        // Apply offset from config (scaled by base_scale to work in screen coordinates)
        let offset = Vec3::new(
            am_config.offset.0 * base_scale,
            am_config.offset.1 * base_scale,
            0.0,
        );

        // Mark as battle entity and apply scale and offset
        // IMPORTANT: We must update inv_fit_scale when we override the Transform.scale
        // to keep mask coordinate calculations consistent with the actual transform.
        commands.entity(entity).insert((
            battle_scoped(),
            Transform {
                translation: offset,
                scale: Vec3::splat(final_scale),
                ..Default::default()
            },
        ));

        // Update AmPendingLayers.inv_fit_scale to match our custom scale
        // This ensures mask coordinate calculations use the correct scale factor
        commands
            .entity(entity)
            .queue(move |mut entity_world: EntityWorldMut| {
                // Update all descendant AmPendingLayers components
                if let Some(mut pending) = entity_world.get_mut::<AmPendingLayers>() {
                    let old_inv_fit_scale = pending.inv_fit_scale;
                    pending.inv_fit_scale = 1.0 / final_scale;
                    info!(
                        "[AM Battle] Updated inv_fit_scale: {} -> {} (final_scale={})",
                        old_inv_fit_scale, pending.inv_fit_scale, final_scale
                    );
                }
            });

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

/// System to check if Alight Motion performance has completed.
///
/// 检查 Alight Motion 演出是否完成的系统。
fn check_am_performance_completion(
    playback: Option<Res<AmPlayback>>,
    mut am_state: ResMut<AlightMotionPerformanceState>,
) {
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
    query: Query<Entity, With<AlightMotionEntity>>,
    mut am_state: ResMut<AlightMotionPerformanceState>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    am_state.is_playing = false;
    am_state.project_entity = None;

    info!("[AM Battle] Cleaned up AM entities");
}

/// System to apply visibility hidden to entities with AlightMotionHiddenMarker.
/// Runs after propagate_am_markers_system and apply_deferred so all markers are propagated.
///
/// 将带有 AlightMotionHiddenMarker 的实体设置为隐藏。
/// 在 propagate_am_markers_system 和 apply_deferred 之后运行，确保所有标记都已传播。
fn apply_am_hidden_visibility(
    mut hidden_entities: Query<(Entity, &Name, &mut Visibility), With<AlightMotionHiddenMarker>>,
) {
    for (entity, name, mut visibility) in hidden_entities.iter_mut() {
        if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
            // Only log when we actually change visibility
            info!(
                "[AM Battle] Applied Hidden visibility to entity {:?} '{}'",
                entity, name
            );
        }
    }
}

/// System to dynamically update battle box bounds based on current animation time.
/// This handles battle boxes with scale animations (e.g., shrinking/expanding).
///
/// 根据当前动画时间动态更新战斗框边界。
/// 处理带有缩放动画的战斗框（如收缩/扩展）。
fn update_am_battle_box_bounds_system(
    playback: Option<Res<AmPlayback>>,
    am_state: Res<AlightMotionPerformanceState>,
    mut battle_box_query: Query<(
        Entity,
        &AmAnimated,
        &AmLayerSpec,
        &mut AlightMotionBattleBoxBounds,
    )>,
    parent_query: Query<&ChildOf>,
    animated_query: Query<&AmAnimated>,
) {
    let Some(playback) = playback else {
        return;
    };

    if !am_state.is_playing {
        return;
    }

    let current_time_ms = playback.current_time_ms;

    for (entity, animated, layer_spec, mut bounds) in battle_box_query.iter_mut() {
        // Get base size from layer spec
        let (base_width, base_height) = match layer_spec {
            AmLayerSpec::SdfShape { width, height, .. } => (width.abs(), height.abs()),
            AmLayerSpec::Image { width, height, .. } => (width.abs(), height.abs()),
            _ => continue,
        };

        // Calculate total scale by traversing parent hierarchy with current time interpolation
        let total_scale = compute_total_scale_at_time(
            entity,
            &animated_query,
            &parent_query,
            am_state.final_scale,
            current_time_ms,
        );

        // Get this entity's current scale at this time
        let local_time = animated.calc_local_time(current_time_ms);
        let local_scale = get_animated_scale_at_time(&animated.scale, local_time);

        // Final dimensions
        let new_width = base_width * total_scale.x * local_scale.x;
        let new_height = base_height * total_scale.y * local_scale.y;

        // Calculate center_offset with current scale
        // anchor_offset is static, but we need to scale it by current total scale
        let full_scale = total_scale * local_scale;
        let new_center_offset = -animated.anchor_offset * full_scale;

        // Only update if changed significantly (avoid noise)
        if (bounds.width - new_width).abs() > 0.1
            || (bounds.height - new_height).abs() > 0.1
            || (bounds.center_offset - new_center_offset).length() > 0.1
        {
            bounds.width = new_width;
            bounds.height = new_height;
            bounds.center_offset = new_center_offset;
        }
    }
}

/// Get animated scale at a specific local time using interpolation.
///
/// 使用插值获取特定本地时间的动画缩放。
fn get_animated_scale_at_time(scale_prop: &AmAnimatedVec2, local_time_ms: f32) -> Vec2 {
    // Use interpolate_vec2 from bevy_alight_motion
    if let Some([x, y]) = interpolate_vec2(scale_prop, local_time_ms) {
        Vec2::new(x.abs(), y.abs())
    } else {
        // Fall back to default
        Vec2::ONE
    }
}

/// Compute total scale from parent hierarchy at a specific time.
///
/// 计算特定时间下从父级层次结构累积的总缩放。
fn compute_total_scale_at_time(
    entity: Entity,
    animated_query: &Query<&AmAnimated>,
    parent_query: &Query<&ChildOf>,
    final_scale: f32,
    current_time_ms: f32,
) -> Vec2 {
    let mut total_scale = Vec2::splat(final_scale);
    let mut current = entity;

    // Traverse up the hierarchy (skip the entity itself, we handle it separately)
    if let Ok(child_of) = parent_query.get(current) {
        current = child_of.0;
    } else {
        return total_scale;
    }

    // Traverse parent chain
    loop {
        if let Ok(animated) = animated_query.get(current) {
            let local_time = animated.calc_local_time(current_time_ms);
            let scale = get_animated_scale_at_time(&animated.scale, local_time);
            total_scale *= scale;
        }

        if let Ok(child_of) = parent_query.get(current) {
            current = child_of.0;
        } else {
            break;
        }
    }

    total_scale
}

/// System to synchronize inv_fit_scale with the scale applied by souprune.
/// This ensures mask coordinates are correctly calculated when souprune applies
/// additional scaling to the Alight Motion project root entity.
///
/// 同步 inv_fit_scale 与 souprune 应用的缩放。
/// 确保当 souprune 对 Alight Motion 项目根实体应用额外缩放时，遮罩坐标能正确计算。
fn sync_am_fit_scale_system(
    am_state: Res<AlightMotionPerformanceState>,
    mut pending_layers_query: Query<&mut AmPendingLayers>,
) {
    if !am_state.is_playing {
        return;
    }

    // Update inv_fit_scale based on the scale applied by souprune
    // final_scale is the combined scale applied to the project root
    for mut pending_layers in pending_layers_query.iter_mut() {
        let expected_inv_fit_scale = 1.0 / am_state.final_scale;

        // Only update if different (avoid unnecessary mutation)
        if (pending_layers.inv_fit_scale - expected_inv_fit_scale).abs() > 0.0001 {
            info!(
                "[AM Battle] Updating inv_fit_scale from {} to {} (final_scale={})",
                pending_layers.inv_fit_scale, expected_inv_fit_scale, am_state.final_scale
            );
            pending_layers.inv_fit_scale = expected_inv_fit_scale;
        }
    }
}
