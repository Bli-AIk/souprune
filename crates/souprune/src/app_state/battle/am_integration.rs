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
//! - `#B*`: Bullet layers - get TriggerCollider + BulletDamage
//!   `#B*`: 弹幕图层 - 添加 TriggerCollider + BulletDamage
//!
//! - `#C*`: Battle box boundary layers - get BattleBox marker
//!   `#C*`: 战斗框边界图层 - 添加 BattleBox 标记

use bevy::prelude::*;
use bevy_alight_motion::prelude::*;

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
}

fn default_scale() -> f32 {
    1.0
}

fn default_offset() -> (f32, f32) {
    (0.0, 0.0)
}

impl Default for AmBattleConfig {
    fn default() -> Self {
        Self {
            scale: 1.0,
            offset: (0.0, 0.0),
        }
    }
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
                        "[AM Battle] Loaded config from {}: scale={}, offset={:?}",
                        config_path, am_config.scale, am_config.offset
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
}

/// Observer function that handles AmEntitySpawned events.
/// Adds collision components based on layer naming conventions.
///
/// 处理 AmEntitySpawned 事件的观察者函数。
/// 根据图层命名约定添加碰撞组件。
pub fn on_am_entity_spawned(
    trigger: Trigger<AmEntitySpawned>,
    mut commands: Commands,
    sprite_query: Query<&Sprite>,
) {
    let event = trigger.event();
    let layer_name = &event.layer_name;

    info!(
        "[AM Battle] Entity spawned: '{}' (type={:?})",
        layer_name, event.element_type
    );

    // Add AmBattleEntity marker to all AM entities
    commands.entity(event.entity).insert(AmBattleEntity);

    // #B prefix: Bullet - add collision and damage
    if layer_name.starts_with("#B") {
        // Get sprite size if available, otherwise use default
        let half_size = if let Ok(sprite) = sprite_query.get(event.entity) {
            if let Some(rect) = &sprite.rect {
                Vec2::new(rect.width() / 2.0, rect.height() / 2.0)
            } else {
                // Default bullet size
                Vec2::new(24.0, 24.0)
            }
        } else {
            Vec2::new(24.0, 24.0)
        };

        commands.entity(event.entity).insert((
            Bullet,
            TriggerCollider::Box { half_size },
            BulletDamage(1.0),
            BulletHitBehavior::persistent(),
            BulletLastHitTime::default(),
            BulletMotionState::new(Vec2::ZERO),
        ));

        info!(
            "  → Added bullet collision to '{}' (half_size={:?})",
            layer_name, half_size
        );
    }

    // #C prefix: Battle box boundary
    if layer_name.starts_with("#C") {
        // Get sprite size for the battle box bounds
        let (width, height) = if let Ok(sprite) = sprite_query.get(event.entity) {
            if let Some(rect) = &sprite.rect {
                (rect.width(), rect.height())
            } else if let Some(custom_size) = sprite.custom_size {
                (custom_size.x, custom_size.y)
            } else {
                // Default battle box size
                (565.0, 140.0)
            }
        } else {
            (565.0, 140.0)
        };

        commands.entity(event.entity).insert((
            BattleBox,
            AmBattleBoxBounds { width, height },
        ));

        info!(
            "  → Added BattleBox marker to '{}' (size={}x{})",
            layer_name, width, height
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
