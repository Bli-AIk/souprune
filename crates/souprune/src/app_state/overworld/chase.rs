//! # chase.rs
//!
//! # chase.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module implements the chase state visual effects for the Overworld.
//! When entering chase state, the following effects are applied:
//! - Full screen dark overlay with 0.5 alpha
//! - Red 1-pixel outline on player sprite using custom shader (outline only, no original image)
//! - Heart marker (judgment indicator) attached to player as child entity
//!
//! All effects have a 0.5 second alpha transition.
//!
//! 本模块实现 Overworld 的追逐战状态视觉效果。
//! 进入追逐战状态时，将应用以下效果：
//! - 全屏 0.5 透明度的黑色覆盖层
//! - 玩家精灵使用自定义着色器的红色1像素描边（仅描边，不含原始图像）
//! - 心形判定标记作为玩家的子实体附着
//!
//! 所有效果都有 0.5 秒的透明度过渡。

use bevy::image::TextureAtlasLayout;
use bevy::prelude::*;
use bevy::sprite_render::MeshMaterial2d;
use bevy_alight_motion::sdf_material::SdfMaterial;

use crate::app_state::overworld::OverworldUpdate;
use crate::app_state::overworld::character::components::PlayerControlled;
use crate::app_state::{ModeScoped, SequenceSubState};
use crate::config;
use crate::core::state_config::LoadedStateConfig;
use crate::core::view::PixelOutlineMaterial;
use crate::core::view::sdf_shape::ViewSdfShape;

pub use super::chase_config::*;
pub use super::chase_damage::*;

/// Marker component for entities that should be highlighted during chase state.
///
/// 标记组件，用于在追逐战状态下需要高亮的实体。
#[derive(Component, Debug, Default)]
pub struct ChaseHighlight;

/// Component for chase effect entities that need alpha transition.
///
/// 需要透明度过渡的追逐战效果实体组件。
#[derive(Component, Debug)]
pub struct ChaseEffect {
    /// Target alpha value
    pub target_alpha: f32,
}

impl Default for ChaseEffect {
    fn default() -> Self {
        Self { target_alpha: 0.5 }
    }
}

/// Marker for the chase dark overlay entity.
///
/// 追逐战黑色覆盖层实体的标记。
#[derive(Component)]
pub struct ChaseDarkOverlay;

/// Marker for the player outline entity, also stores current sprite size for change detection.
///
/// 玩家描边实体的标记，同时存储当前精灵尺寸用于变化检测。
#[derive(Component)]
pub struct ChasePlayerOutline {
    /// Current sprite size (used to detect when mesh needs updating)
    pub current_size: Vec2,
}

/// Marker for the heart marker (judgment indicator) entity.
///
/// 心形判定标记实体的标记组件。
#[derive(Component)]
pub struct ChaseHeartMarker;

/// Root entity for organizing chase effect visualizers.
///
/// 用于组织追逐战效果可视化器的根实体。
#[derive(Component)]
pub struct ChaseEffectRoot;

/// Resource to track chase state transition.
///
/// 追踪追逐战状态过渡的资源。
#[derive(Resource, Default)]
pub struct ChaseTransition {
    /// Whether currently in chase state
    pub active: bool,
    /// Transition timer (0.0 to 0.5 seconds)
    pub timer: f32,
    /// Whether transitioning in or out
    pub transitioning_in: bool,
    /// Whether cleanup has been done
    pub cleanup_done: bool,
}

pub struct ChasePlugin;

impl Plugin for ChasePlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        // Initialize chase resources (config will be loaded dynamically)
        app.init_resource::<ChaseEnabled>()
            .init_resource::<ChaseStateName>()
            .init_resource::<ChaseStateTracker>()
            .init_resource::<ChaseTransition>()
            .init_resource::<DamageUIState>()
            .init_resource::<PlayerInvincibility>()
            .init_resource::<ChaseConfigLoaded>()
            .add_message::<ChasePlayerDamageEvent>()
            // Load chase config dynamically when state config becomes available
            // Must run before FRETriggerSet so chase state actions can work
            .add_systems(
                schedule,
                load_chase_config_system
                    .run_if(|loaded: Res<ChaseConfigLoaded>| !loaded.0)
                    .before(super::FRETriggerSet),
            )
            // Dynamic state change detection (replaces OnEnter/OnExit)
            .add_systems(
                schedule,
                (
                    detect_chase_state_enter_system,
                    detect_chase_state_exit_system,
                )
                    .chain()
                    .before(update_chase_transition_system)
                    .in_set(OverworldUpdate)
                    .run_if(chase_enabled),
            )
            .add_systems(
                schedule,
                (
                    update_chase_transition_system,
                    spawn_chase_dark_overlay_system,
                    spawn_player_outline_system,
                    spawn_heart_marker_system,
                    spawn_player_hitbox_system,
                    update_player_outline_system,
                    update_chase_effect_alpha_system,
                    update_heart_marker_alpha_system,
                    chase_damage_detection_system,
                    update_player_invincibility_system,
                    damage_ui_display_system,
                    cleanup_chase_effects_system,
                    cleanup_player_hitbox_system,
                )
                    .chain()
                    .in_set(OverworldUpdate)
                    .run_if(chase_enabled),
            );
    }
}

/// Resource to track if chase config has been loaded (to avoid repeated loading attempts).
#[derive(Resource, Default)]
pub struct ChaseConfigLoaded(pub bool);

/// Run condition: check if chase is enabled
fn chase_enabled(enabled: Res<ChaseEnabled>) -> bool {
    enabled.0
}

/// Check if current state is a chase state (has chase_config in its definition).
pub fn is_in_chase_state(
    current_state: &SequenceSubState,
    chase_state_name: &ChaseStateName,
) -> bool {
    chase_state_name
        .0
        .as_ref()
        .is_some_and(|name| current_state.0 == *name)
}

/// System to detect when entering chase state.
fn detect_chase_state_enter_system(
    mut commands: Commands,
    current_state: Res<State<SequenceSubState>>,
    chase_state_name: Res<ChaseStateName>,
    mut tracker: ResMut<ChaseStateTracker>,
    mut transition: ResMut<ChaseTransition>,
    chase_config: Option<Res<ChaseConfig>>,
    asset_server: Res<AssetServer>,
) {
    let in_chase = is_in_chase_state(&current_state, &chase_state_name);

    if in_chase && !tracker.was_in_chase {
        // Just entered chase state
        tracker.was_in_chase = true;

        // Setup chase effect root
        commands.spawn((
            ModeScoped("overworld".to_string()),
            ChaseEffectRoot,
            Transform::default(),
            Visibility::default(),
        ));
        info!("Chase: Created effect root entity");

        // Start transition in
        transition.active = true;
        transition.timer = 0.0;
        transition.transitioning_in = true;
        transition.cleanup_done = false;
        info!("Chase: Starting transition IN");

        // Setup HUD
        if let Some(config) = chase_config {
            // Load HUD layout only if path is specified
            // 仅当路径指定时才加载 HUD 布局
            let layout_path = &config.damage_ui.layout_path;
            if !layout_path.is_empty() {
                let _handle: Handle<crate::core::view::layout::ViewLayoutAsset> =
                    asset_server.load(layout_path.clone());
                info!("Chase: Setup HUD from {}", layout_path);
            } else {
                warn!("Chase: No damage UI layout path configured");
            }
        }
    }
}

/// System to detect when exiting chase state.
fn detect_chase_state_exit_system(
    current_state: Res<State<SequenceSubState>>,
    chase_state_name: Res<ChaseStateName>,
    mut tracker: ResMut<ChaseStateTracker>,
    mut transition: ResMut<ChaseTransition>,
) {
    let in_chase = is_in_chase_state(&current_state, &chase_state_name);

    if !in_chase && tracker.was_in_chase {
        // Just exited chase state
        tracker.was_in_chase = false;

        // Start transition out
        transition.transitioning_in = false;
        transition.timer = 0.0;
        info!("Chase: Starting transition OUT");
    }
}

/// System to load chase configuration from SoupruneConfig.
/// This reads the chase_config path from mod.toml and loads the config.
/// Also finds which state has chase_config in states.ron.
///
/// 从 SoupruneConfig 加载追逐战配置的系统。
/// 从 mod.toml 读取 chase_config 路径并加载配置。
/// 同时查找 states.ron 中哪个状态有 chase_config。
fn load_chase_config_system(
    mut commands: Commands,
    souprune_config: Res<config::SoupruneConfig>,
    state_config: Res<LoadedStateConfig>,
    state_config_loaded: Res<crate::core::state_config::StateConfigLoaded>,
    mut chase_enabled: ResMut<ChaseEnabled>,
    mut chase_state_name: ResMut<ChaseStateName>,
    mut chase_loaded: ResMut<ChaseConfigLoaded>,
) {
    // Wait for state config to be actually loaded from file (not just default)
    if !state_config_loaded.0 {
        return;
    }

    // Mark as loaded to prevent repeated attempts
    chase_loaded.0 = true;

    // First try to find chase state from states.ron
    for (state_name, state_def) in &state_config.0.states {
        if state_def.chase_config.is_some() {
            chase_state_name.0 = Some(state_name.clone());
            info!("Chase: Found chase state '{}' in states.ron", state_name);

            // Load config from the path specified in state definition
            if let Some(path) = &state_def.chase_config
                && let Some(config) = ChaseConfig::load_from_path(Some(path.as_str()))
            {
                info!("Chase: Enabled with config from {}", path);
                commands.insert_resource(config);
                chase_enabled.0 = true;
                return;
            }
        }
    }

    // Fallback: try chase_config from mod.toml (legacy support)
    let chase_config_path = souprune_config.game.chase_config.as_deref();

    if let Some(config) = ChaseConfig::load_from_path(chase_config_path) {
        info!(
            "Chase: Enabled with config from mod.toml: {:?}",
            chase_config_path
        );
        commands.insert_resource(config);
        chase_enabled.0 = true;
        // Assume "Chase" state name for legacy config
        if chase_state_name.0.is_none() {
            chase_state_name.0 = Some("Chase".to_string());
        }
    } else if chase_config_path.is_some() {
        // Path was specified but config couldn't be loaded
        warn!("Chase: Disabled - config file could not be loaded");
        commands.insert_resource(ChaseConfig::default());
        chase_enabled.0 = false;
    } else {
        // No path specified - chase is intentionally disabled
        info!("Chase: Disabled - no chase_config specified");
        commands.insert_resource(ChaseConfig::default());
        chase_enabled.0 = false;
    }
}

/// Setup the root entity for chase effect visualizers.
///
/// 设置追逐战效果可视化器的根实体。
fn setup_chase_effect_root_system(mut commands: Commands) {
    commands.spawn((
        ModeScoped("overworld".to_string()),
        ChaseEffectRoot,
        Name::new("ChaseEffectRoot"),
        Transform::default(),
        Visibility::default(),
    ));
    info!("Chase: Created effect root entity");
}

/// Start transition into chase state.
///
/// 开始进入追逐战状态的过渡。
fn start_chase_transition_in_system(mut transition: ResMut<ChaseTransition>) {
    transition.active = true;
    transition.timer = 0.0;
    transition.transitioning_in = true;
    transition.cleanup_done = false;
    info!("Chase: Starting transition IN");
}

/// Start transition out of chase state.
///
/// 开始退出追逐战状态的过渡。
fn start_chase_transition_out_system(
    mut transition: ResMut<ChaseTransition>,
    chase_config: Res<ChaseConfig>,
) {
    transition.transitioning_in = false;
    transition.timer = chase_config.transition_duration();
    info!("Chase: Starting transition OUT");
}

/// Update chase transition timer.
///
/// 更新追逐战过渡计时器。
fn update_chase_transition_system(
    time: Res<Time>,
    mut transition: ResMut<ChaseTransition>,
    chase_config: Res<ChaseConfig>,
) {
    if !transition.active && !transition.transitioning_in && transition.timer <= 0.0 {
        return;
    }

    let duration = chase_config.transition_duration();
    if transition.transitioning_in {
        transition.timer = (transition.timer + time.delta_secs()).min(duration);
    } else {
        transition.timer = (transition.timer - time.delta_secs()).max(0.0);
        if transition.timer <= 0.0 {
            transition.active = false;
        }
    }
}

/// Spawn the dark overlay when entering chase state.
///
/// 进入追逐战状态时生成黑色覆盖层。
fn spawn_chase_dark_overlay_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut sdf_materials: ResMut<Assets<SdfMaterial>>,
    transition: Res<ChaseTransition>,
    chase_config: Res<ChaseConfig>,
    chase_root: Query<Entity, With<ChaseEffectRoot>>,
    existing_overlay: Query<Entity, With<ChaseDarkOverlay>>,
) {
    // Only spawn if transitioning in and overlay doesn't exist
    if !transition.transitioning_in || !existing_overlay.is_empty() {
        return;
    }

    let Ok(root_entity) = chase_root.single() else {
        return;
    };

    // Create a very large box to cover the screen
    let overlay_size = chase_config.dark_overlay.overlay_size;
    let shape = ViewSdfShape::new(overlay_size, overlay_size, Color::srgba(0.0, 0.0, 0.0, 0.0));
    let mesh = meshes.add(shape.create_mesh());
    let material = sdf_materials.add(shape.to_material());

    commands.entity(root_entity).with_children(|parent| {
        parent.spawn((
            ChaseDarkOverlay,
            ChaseEffect {
                target_alpha: chase_config.dark_overlay.target_alpha,
            },
            shape,
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, chase_config.dark_overlay.z_offset),
            Name::new("ChaseDarkOverlay"),
        ));
    });

    info!("Chase: Spawned dark overlay");
}

/// Spawn player outline mesh using PixelOutlineMaterial.
///
/// 使用 PixelOutlineMaterial 生成玩家描边网格。
fn spawn_player_outline_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PixelOutlineMaterial>>,
    images: Res<Assets<Image>>,
    atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    transition: Res<ChaseTransition>,
    chase_config: Res<ChaseConfig>,
    chase_root: Query<Entity, With<ChaseEffectRoot>>,
    player_query: Query<(Entity, &Sprite, &Transform), With<PlayerControlled>>,
    existing_outline: Query<Entity, With<ChasePlayerOutline>>,
) {
    // Only spawn if transitioning in and outline doesn't exist
    if !transition.transitioning_in || !existing_outline.is_empty() {
        return;
    }

    let Ok(root_entity) = chase_root.single() else {
        return;
    };

    let Ok((_player_entity, sprite, player_transform)) = player_query.single() else {
        return;
    };

    // Get the sprite's texture
    let texture = sprite.image.clone();

    // Get texture dimensions
    let Some(image) = images.get(&texture) else {
        return;
    };
    let tex_size = image.size().as_vec2();

    // Calculate UV rect based on TextureAtlas or sprite.rect
    let (uv_rect, sprite_size) = if let Some(ref atlas) = sprite.texture_atlas {
        // Using TextureAtlas - get rect from layout
        if let Some(layout) = atlas_layouts.get(&atlas.layout) {
            let rect = layout.textures[atlas.index];
            let uv = Vec4::new(
                rect.min.x as f32 / tex_size.x,
                rect.min.y as f32 / tex_size.y,
                rect.max.x as f32 / tex_size.x,
                rect.max.y as f32 / tex_size.y,
            );
            let size = Vec2::new(rect.width() as f32, rect.height() as f32);
            (uv, size)
        } else {
            (Vec4::new(0.0, 0.0, 1.0, 1.0), tex_size)
        }
    } else if let Some(rect) = sprite.rect {
        // Using sprite.rect
        let uv = Vec4::new(
            rect.min.x / tex_size.x,
            rect.min.y / tex_size.y,
            rect.max.x / tex_size.x,
            rect.max.y / tex_size.y,
        );
        let size = Vec2::new(rect.width(), rect.height());
        (uv, size)
    } else {
        // Full texture
        (Vec4::new(0.0, 0.0, 1.0, 1.0), tex_size)
    };

    // Get flip flags from sprite
    let flip = Vec4::new(
        if sprite.flip_x { 1.0 } else { 0.0 },
        if sprite.flip_y { 1.0 } else { 0.0 },
        0.0,
        0.0,
    );

    // Create a quad mesh for the outline (slightly larger to accommodate outline)
    let outline_size = sprite_size + Vec2::splat(chase_config.outline.padding);
    let mesh = meshes.add(Rectangle::new(outline_size.x, outline_size.y));

    // Create the outline material with red color and 0 alpha initially
    let material = materials.add(PixelOutlineMaterial {
        params: LinearRgba::new(1.0, 0.0, 0.0, 0.0), // Red, 0 alpha
        uv_rect,
        flip,
        texture,
    });

    // Spawn outline entity as child of chase root, positioned at player
    commands.entity(root_entity).with_children(|parent| {
        parent.spawn((
            ChasePlayerOutline {
                current_size: sprite_size,
            },
            ChaseEffect { target_alpha: 1.0 },
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_translation(
                player_transform.translation + Vec3::new(0.0, 0.0, chase_config.outline.z_offset),
            ),
            Name::new("ChasePlayerOutline"),
        ));
    });

    info!("Chase: Spawned player outline");
}

/// Spawn heart marker (judgment indicator) as child of player entity.
///
/// 在玩家实体下生成心形判定标记作为子实体。
fn spawn_heart_marker_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    transition: Res<ChaseTransition>,
    chase_config: Res<ChaseConfig>,
    player_query: Query<Entity, With<PlayerControlled>>,
    existing_markers: Query<Entity, With<ChaseHeartMarker>>,
) {
    // Only spawn if transitioning in and marker doesn't exist
    if !transition.transitioning_in || !existing_markers.is_empty() {
        return;
    }

    let Ok(player_entity) = player_query.single() else {
        return;
    };

    let config = &chase_config.heart_marker;

    // Skip if no texture path configured
    // 如果未配置纹理路径则跳过
    if config.texture_path.is_empty() {
        warn!("Chase: No heart marker texture path configured");
        return;
    }

    let offset = config.offset.to_vec2();

    // Load the heart texture
    let texture: Handle<Image> = asset_server.load(&config.texture_path);

    // Spawn heart marker as child of player so it follows automatically
    commands.entity(player_entity).with_children(|parent| {
        parent.spawn((
            ChaseHeartMarker,
            Sprite {
                image: texture,
                // Start with 0 alpha for fade in
                color: Color::srgba(config.color.r, config.color.g, config.color.b, 0.0),
                ..default()
            },
            Transform::from_xyz(offset.x, offset.y, config.z_offset)
                .with_scale(Vec3::splat(config.scale)),
            Name::new("ChaseHeartMarker"),
        ));
    });

    info!("Chase: Spawned heart marker");
}

/// Update player outline position and texture to follow player sprite.
///
/// 更新玩家描边位置和纹理以跟随玩家精灵。
fn update_player_outline_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut outline_query: Query<
        (
            Entity,
            &mut Transform,
            &mut ChasePlayerOutline,
            &MeshMaterial2d<PixelOutlineMaterial>,
        ),
        Without<PlayerControlled>,
    >,
    player_query: Query<(&Transform, &Sprite), With<PlayerControlled>>,
    images: Res<Assets<Image>>,
    atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    mut materials: ResMut<Assets<PixelOutlineMaterial>>,
    transition: Res<ChaseTransition>,
    chase_config: Res<ChaseConfig>,
) {
    if !transition.active {
        return;
    }

    let Ok((player_transform, sprite)) = player_query.single() else {
        return;
    };

    for (entity, mut outline_transform, mut outline_marker, material_handle) in
        outline_query.iter_mut()
    {
        // Update position
        outline_transform.translation.x = player_transform.translation.x;
        outline_transform.translation.y = player_transform.translation.y;

        // Update texture, UV rect, and flip if changed
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.texture = sprite.image.clone();

            // Update flip flags
            material.flip = Vec4::new(
                if sprite.flip_x { 1.0 } else { 0.0 },
                if sprite.flip_y { 1.0 } else { 0.0 },
                0.0,
                0.0,
            );

            // Update UV rect and mesh size based on TextureAtlas or sprite.rect
            if let Some(image) = images.get(&sprite.image) {
                let tex_size = image.size().as_vec2();
                let (new_uv_rect, sprite_size) = if let Some(ref atlas) = sprite.texture_atlas {
                    if let Some(layout) = atlas_layouts.get(&atlas.layout) {
                        let rect = layout.textures[atlas.index];
                        let uv = Vec4::new(
                            rect.min.x as f32 / tex_size.x,
                            rect.min.y as f32 / tex_size.y,
                            rect.max.x as f32 / tex_size.x,
                            rect.max.y as f32 / tex_size.y,
                        );
                        let size = Vec2::new(rect.width() as f32, rect.height() as f32);
                        (uv, size)
                    } else {
                        (Vec4::new(0.0, 0.0, 1.0, 1.0), tex_size)
                    }
                } else if let Some(rect) = sprite.rect {
                    let uv = Vec4::new(
                        rect.min.x / tex_size.x,
                        rect.min.y / tex_size.y,
                        rect.max.x / tex_size.x,
                        rect.max.y / tex_size.y,
                    );
                    let size = Vec2::new(rect.width(), rect.height());
                    (uv, size)
                } else {
                    (Vec4::new(0.0, 0.0, 1.0, 1.0), tex_size)
                };

                material.uv_rect = new_uv_rect;

                // Only update mesh if sprite size changed (avoid creating new mesh every frame)
                // 只在精灵尺寸变化时更新 mesh（避免每帧创建新 mesh）
                if (outline_marker.current_size - sprite_size).length() > 0.01 {
                    outline_marker.current_size = sprite_size;
                    let outline_size = sprite_size + Vec2::splat(chase_config.outline.padding);
                    let new_mesh = meshes.add(Rectangle::new(outline_size.x, outline_size.y));
                    commands.entity(entity).insert(Mesh2d(new_mesh));
                }
            }
        }
    }
}

/// Update alpha values for all chase effects based on transition progress.
///
/// 根据过渡进度更新所有追逐战效果的透明度。
fn update_chase_effect_alpha_system(
    transition: Res<ChaseTransition>,
    chase_config: Res<ChaseConfig>,
    sdf_effects: Query<(&ViewSdfShape, &MeshMaterial2d<SdfMaterial>, &ChaseEffect)>,
    mut sdf_materials: ResMut<Assets<SdfMaterial>>,
    mut outline_query: Query<&MeshMaterial2d<PixelOutlineMaterial>, With<ChasePlayerOutline>>,
    mut materials: ResMut<Assets<PixelOutlineMaterial>>,
) {
    let duration = chase_config.transition_duration();
    let progress = if duration > 0.0 {
        transition.timer / duration
    } else {
        1.0
    };

    // Update SDF shape effects (dark overlay)
    for (shape, mat_handle, effect) in sdf_effects.iter() {
        let target_alpha = effect.target_alpha * progress;
        if let Some(material) = sdf_materials.get_mut(&mat_handle.0) {
            // Update the material color with new alpha
            let mut new_shape = shape.clone();
            let mut color = shape.color.to_srgba();
            color.alpha = target_alpha;
            new_shape.color = color.into();
            *material = new_shape.to_material();
        }
    }

    // Update outline material alpha
    for material_handle in outline_query.iter_mut() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.params.alpha = progress;
        }
    }
}

/// Update heart marker alpha based on transition progress.
///
/// 根据过渡进度更新心形标记的透明度。
fn update_heart_marker_alpha_system(
    transition: Res<ChaseTransition>,
    chase_config: Res<ChaseConfig>,
    player_invincibility: Res<PlayerInvincibility>,
    mut heart_markers: Query<&mut Sprite, With<ChaseHeartMarker>>,
) {
    // Skip if player is invincible (let invincibility system handle color)
    // 如果玩家处于无敌状态，跳过（让无敌系统处理颜色）
    if player_invincibility.is_invincible() {
        return;
    }

    // Only update during active transition
    // 仅在活动过渡期间更新
    if !transition.active {
        return;
    }

    let duration = chase_config.transition_duration();
    let progress = if duration > 0.0 {
        transition.timer / duration
    } else {
        1.0
    };

    let config = &chase_config.heart_marker;
    for mut sprite in heart_markers.iter_mut() {
        sprite.color = Color::srgba(
            config.color.r,
            config.color.g,
            config.color.b,
            config.color.a * progress,
        );
    }
}

/// Cleanup chase effects when transition out is complete.
///
/// 当退出过渡完成时清理追逐战效果。
fn cleanup_chase_effects_system(
    mut commands: Commands,
    mut transition: ResMut<ChaseTransition>,
    chase_root: Query<Entity, With<ChaseEffectRoot>>,
    overlays: Query<Entity, With<ChaseDarkOverlay>>,
    outlines: Query<Entity, With<ChasePlayerOutline>>,
    heart_markers: Query<Entity, With<ChaseHeartMarker>>,
) {
    // Only cleanup when transition out is complete and cleanup hasn't been done
    if transition.active || transition.transitioning_in || transition.timer > 0.0 {
        return;
    }

    // Skip if already cleaned up
    if transition.cleanup_done {
        return;
    }

    // Despawn all chase effect entities
    for entity in overlays.iter() {
        commands.entity(entity).despawn();
    }
    for entity in outlines.iter() {
        commands.entity(entity).despawn();
    }
    for entity in heart_markers.iter() {
        commands.entity(entity).despawn();
    }

    // Despawn root
    if let Ok(root) = chase_root.single() {
        commands.entity(root).despawn();
    }

    transition.cleanup_done = true;
    info!("Chase: Cleaned up all effects");
}
