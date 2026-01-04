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
//! All effects have a 0.5 second alpha transition.
//!
//! 本模块实现 Overworld 的追逐战状态视觉效果。
//! 进入追逐战状态时，将应用以下效果：
//! - 全屏 0.5 透明度的黑色覆盖层
//! - 玩家精灵使用自定义着色器的红色1像素描边（仅描边，不含原始图像）
//! - 心形判定标记作为玩家的子实体附着
//! 所有效果都有 0.5 秒的透明度过渡。

use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::image::TextureAtlasLayout;
use bevy::prelude::*;
use bevy::sprite_render::MeshMaterial2d;
use bevy_smud::prelude::*;
use serde::Deserialize;
use std::fs;

use crate::app_state::overworld::character::components::PlayerControlled;
use crate::app_state::overworld::{OverworldEntity, OverworldState, OverworldUpdate};
use crate::config;
use crate::core::ui::PixelOutlineMaterial;

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

// ============================================================================
// Chase configuration structures (loaded from RON file)
// 追逐战配置结构（从 RON 文件加载）
// ============================================================================

const CHASE_CONFIG_PATH: &str = "overworld/chase_config.ron";

/// Configuration for the heart marker effect.
///
/// 心形判定标记效果配置。
#[derive(Debug, Clone, Deserialize)]
pub struct HeartMarkerConfig {
    /// Texture path for the heart marker
    pub texture_path: String,
    /// Offset from player center
    pub offset: Vec2Config,
    /// Z-offset from player
    pub z_offset: f32,
    /// Scale of the heart marker
    #[serde(default = "default_heart_scale")]
    pub scale: f32,
    /// Tint color (RGBA)
    pub color: ColorConfig,
    /// Fade duration in seconds
    pub fade_duration: f32,
}

fn default_heart_scale() -> f32 {
    0.5
}

impl Default for HeartMarkerConfig {
    fn default() -> Self {
        Self {
            texture_path: "textures/common/ui/dr_heart.png".to_string(),
            offset: Vec2Config { x: 0.0, y: -2.0 },
            z_offset: 101.0,
            scale: 0.5,
            color: ColorConfig {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            fade_duration: 0.5,
        }
    }
}

/// Configuration for the outline effect.
///
/// 描边效果配置。
#[derive(Debug, Clone, Deserialize)]
pub struct OutlineConfig {
    /// Outline color (RGB)
    pub color: ColorConfig,
    /// Fade duration in seconds
    pub fade_duration: f32,
}

impl Default for OutlineConfig {
    fn default() -> Self {
        Self {
            color: ColorConfig {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            fade_duration: 0.5,
        }
    }
}

/// Configuration for the dark overlay effect.
///
/// 黑色覆盖层效果配置。
#[derive(Debug, Clone, Deserialize)]
pub struct DarkOverlayConfig {
    /// Target alpha value
    pub target_alpha: f32,
    /// Fade duration in seconds
    pub fade_duration: f32,
}

impl Default for DarkOverlayConfig {
    fn default() -> Self {
        Self {
            target_alpha: 0.5,
            fade_duration: 0.5,
        }
    }
}

/// Simple Vec2 config for RON deserialization.
///
/// 用于 RON 反序列化的简单 Vec2 配置。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Vec2Config {
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
}

impl Vec2Config {
    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

/// Simple color config for RON deserialization.
///
/// 用于 RON 反序列化的简单颜色配置。
#[derive(Debug, Clone, Deserialize)]
pub struct ColorConfig {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    #[serde(default = "default_alpha")]
    pub a: f32,
}

fn default_alpha() -> f32 {
    1.0
}

/// Hitbox shape configuration.
///
/// 判定框形状配置。
#[derive(Debug, Clone, Deserialize)]
pub enum HitboxShapeConfig {
    Circle { radius: f32 },
    Box { half_width: f32, half_height: f32 },
}

impl Default for HitboxShapeConfig {
    fn default() -> Self {
        HitboxShapeConfig::Circle { radius: 4.0 }
    }
}

/// Hitbox configuration for damage detection.
///
/// 用于伤害检测的判定框配置。
#[derive(Debug, Clone, Deserialize)]
pub struct HitboxConfig {
    /// Shape of the hitbox
    pub shape: HitboxShapeConfig,
    /// Offset from player center
    #[serde(default)]
    pub offset: Vec2Config,
}

impl Default for HitboxConfig {
    fn default() -> Self {
        Self {
            shape: HitboxShapeConfig::default(),
            offset: Vec2Config { x: 0.0, y: -5.0 },
        }
    }
}

/// Damage UI configuration.
///
/// 受伤UI配置。
#[derive(Debug, Clone, Deserialize)]
pub struct DamageUIConfig {
    /// UI layout file path
    pub layout_path: String,
    /// Display duration in seconds
    pub display_duration: f32,
}

impl Default for DamageUIConfig {
    fn default() -> Self {
        Self {
            layout_path: "overworld/ui/damage_flash.ui_layout.ron".to_string(),
            display_duration: 0.5,
        }
    }
}

/// Complete chase configuration loaded from RON file.
///
/// 从 RON 文件加载的完整追逐战配置。
#[derive(Debug, Clone, Deserialize, Resource)]
pub struct ChaseConfig {
    pub heart_marker: HeartMarkerConfig,
    pub outline: OutlineConfig,
    pub dark_overlay: DarkOverlayConfig,
    #[serde(default)]
    pub hitbox: HitboxConfig,
    #[serde(default)]
    pub damage_ui: DamageUIConfig,
}

impl Default for ChaseConfig {
    fn default() -> Self {
        Self {
            heart_marker: HeartMarkerConfig::default(),
            outline: OutlineConfig::default(),
            dark_overlay: DarkOverlayConfig::default(),
            hitbox: HitboxConfig::default(),
            damage_ui: DamageUIConfig::default(),
        }
    }
}

impl ChaseConfig {
    /// Load chase configuration from RON file.
    ///
    /// 从 RON 文件加载追逐战配置。
    pub fn load() -> Self {
        if let Some(path) = config::resolve_path(CHASE_CONFIG_PATH) {
            if let Ok(contents) = fs::read_to_string(&path) {
                if let Ok(config) = ron::de::from_str(&contents) {
                    info!("Chase: Loaded config from {}", path.display());
                    return config;
                } else {
                    warn!(
                        "Chase: Failed to parse config at {}, using defaults",
                        path.display()
                    );
                }
            }
        }
        info!("Chase: Using default config");
        Self::default()
    }

    /// Get transition duration (uses heart marker fade duration as reference).
    ///
    /// 获取过渡持续时间（使用心形标记的渐变持续时间作为参考）。
    pub fn transition_duration(&self) -> f32 {
        self.heart_marker.fade_duration
    }
}

pub struct ChasePlugin;

impl Plugin for ChasePlugin {
    fn build(&self, app: &mut App) {
        // Load chase config at plugin build time
        let chase_config = ChaseConfig::load();

        app.insert_resource(chase_config)
            .init_resource::<ChaseTransition>()
            .init_resource::<DamageUIState>()
            .init_resource::<PlayerInvincibility>()
            .add_message::<ChasePlayerDamageEvent>()
            .add_systems(
                OnEnter(OverworldState::Chase),
                (
                    setup_chase_effect_root_system,
                    start_chase_transition_in_system,
                    setup_chase_hud_system,
                ),
            )
            .add_systems(
                OnExit(OverworldState::Chase),
                (start_chase_transition_out_system, cleanup_chase_hud_system),
            )
            .add_systems(
                Update,
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
                    .in_set(OverworldUpdate),
            );
    }
}

/// Setup the root entity for chase effect visualizers.
///
/// 设置追逐战效果可视化器的根实体。
fn setup_chase_effect_root_system(mut commands: Commands) {
    commands.spawn((
        OverworldEntity(),
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
    mut shaders: ResMut<Assets<Shader>>,
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
    let overlay_size = 10000.0;
    let sdf = shaders.add_sdf_expr(format!(
        "smud::sd_box(p, vec2<f32>({}, {}))",
        overlay_size / 2.0,
        overlay_size / 2.0
    ));

    commands.entity(root_entity).with_children(|parent| {
        parent.spawn((
            ChaseDarkOverlay,
            ChaseEffect {
                target_alpha: chase_config.dark_overlay.target_alpha,
            },
            SmudShape {
                color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                sdf,
                frame: Frame::Quad(overlay_size),
                fill: SIMPLE_FILL_HANDLE,
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 50.0), // High Z to be on top
            Name::new("ChaseDarkOverlay"),
        ));
    });

    info!("Chase: Spawned dark overlay");
}

/// Spawn player outline mesh using PixelOutlineMaterial.
///
/// 使用 PixelOutlineMaterial 生成玩家描边网格。
#[allow(clippy::type_complexity)]
fn spawn_player_outline_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PixelOutlineMaterial>>,
    images: Res<Assets<Image>>,
    atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    transition: Res<ChaseTransition>,
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
    let outline_size = sprite_size + Vec2::splat(2.0);
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
            Transform::from_translation(player_transform.translation + Vec3::new(0.0, 0.0, 100.0)),
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
#[allow(clippy::type_complexity)]
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
                    let outline_size = sprite_size + Vec2::splat(2.0);
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
    mut smud_effects: Query<(&mut SmudShape, &ChaseEffect)>,
    mut outline_query: Query<&MeshMaterial2d<PixelOutlineMaterial>, With<ChasePlayerOutline>>,
    mut materials: ResMut<Assets<PixelOutlineMaterial>>,
) {
    let duration = chase_config.transition_duration();
    let progress = if duration > 0.0 {
        transition.timer / duration
    } else {
        1.0
    };

    // Update SmudShape effects (dark overlay)
    for (mut shape, effect) in smud_effects.iter_mut() {
        let target_alpha = effect.target_alpha * progress;
        let mut color = shape.color.to_srgba();
        color.alpha = target_alpha;
        shape.color = color.into();
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

// ============================================================================
// Player Hitbox and Damage Detection Systems (experimental feature)
// 玩家判定框和伤害检测系统（experimental 特性）
// ============================================================================

/// Marker component for the player's damage hitbox in chase mode.
///
/// 追逐战模式下玩家伤害判定框的标记组件。
#[derive(Component)]
pub struct ChasePlayerHitbox;

/// Resource to track damage UI display timer.
///
/// 追踪受伤UI显示计时器的资源。
#[derive(Resource, Default)]
pub struct DamageUIState {
    /// Whether damage UI is currently showing
    pub showing: bool,
    /// Timer for auto-hide
    pub timer: f32,
}

/// Resource to track player invincibility state.
/// Used for both OW chase mode and Battle mode.
///
/// 追踪玩家无敌状态的资源。
/// 用于 OW 追逐战模式和战斗模式。
#[derive(Resource, Default)]
pub struct PlayerInvincibility {
    /// Whether player is currently invincible
    pub active: bool,
    /// Remaining invincibility time
    pub timer: f32,
    /// Flash timer for heart color toggle
    pub flash_timer: f32,
    /// Current flash state (true = normal color, false = flash color)
    pub flash_state: bool,
}

impl PlayerInvincibility {
    /// Start invincibility with the given duration.
    pub fn start(&mut self, duration: f32) {
        self.active = true;
        self.timer = duration;
        self.flash_timer = 0.0;
        self.flash_state = true;
    }

    /// Check if player is invincible.
    pub fn is_invincible(&self) -> bool {
        self.active && self.timer > 0.0
    }
}

/// Event fired when player takes damage in chase mode.
///
/// 玩家在追逐战模式下受伤时触发的事件。
#[derive(Clone)]
pub struct ChasePlayerDamageEvent {
    pub damage: f32,
}

impl Message for ChasePlayerDamageEvent {}

/// System to add TriggerCollider to player when entering chase state.
///
/// 进入追逐战状态时为玩家添加 TriggerCollider 的系统。
pub fn spawn_player_hitbox_system(
    mut commands: Commands,
    chase_config: Res<ChaseConfig>,
    transition: Res<ChaseTransition>,
    player_query: Query<Entity, (With<PlayerControlled>, Without<ChasePlayerHitbox>)>,
) {
    // Only spawn when transitioning in
    if !transition.transitioning_in {
        return;
    }

    let Ok(player_entity) = player_query.single() else {
        return;
    };

    // Create TriggerCollider based on hitbox config
    let trigger_collider = match &chase_config.hitbox.shape {
        HitboxShapeConfig::Circle { radius } => {
            crate::core::collision::TriggerCollider::Circle { radius: *radius }
        }
        HitboxShapeConfig::Box {
            half_width,
            half_height,
        } => crate::core::collision::TriggerCollider::Box {
            half_size: Vec2::new(*half_width, *half_height),
        },
    };

    commands.entity(player_entity).insert((
        ChasePlayerHitbox,
        trigger_collider,
        crate::core::collision::HitboxOffset(chase_config.hitbox.offset.to_vec2()),
    ));

    info!(
        "Chase: Added player hitbox with shape {:?}",
        chase_config.hitbox.shape
    );
}

/// System to remove TriggerCollider from player when exiting chase state.
///
/// 退出追逐战状态时从玩家移除 TriggerCollider 的系统。
pub fn cleanup_player_hitbox_system(
    mut commands: Commands,
    transition: Res<ChaseTransition>,
    player_query: Query<Entity, With<ChasePlayerHitbox>>,
) {
    // Only cleanup when transition out is complete
    if transition.active || transition.transitioning_in || transition.timer > 0.0 {
        return;
    }

    for player_entity in player_query.iter() {
        commands
            .entity(player_entity)
            .remove::<ChasePlayerHitbox>()
            .remove::<crate::core::collision::TriggerCollider>()
            .remove::<crate::core::collision::HitboxOffset>();
        info!("Chase: Removed player hitbox");
    }
}

/// System to detect bullet collision with player hitbox in chase mode.
///
/// 检测追逐战模式下弹幕与玩家判定框碰撞的系统。
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn chase_damage_detection_system(
    mut commands: Commands,
    chase_config: Res<ChaseConfig>,
    player_behavior: Res<crate::app_state::overworld::player::config::PlayerBehavior>,
    overworld_state: Res<State<OverworldState>>,
    asset_server: Res<AssetServer>,
    mut player_invincibility: ResMut<PlayerInvincibility>,
    mut player_data: ResMut<crate::core::data::PlayerData>,
    #[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))] audio: Res<
        bevy_kira_audio::Audio,
    >,
    player_query: Query<
        (
            &Transform,
            &crate::core::collision::TriggerCollider,
            Option<&crate::core::collision::HitboxOffset>,
            Option<&crate::app_state::overworld::character::components::StateWalking>,
            Option<&crate::app_state::overworld::character::components::StateRunning>,
        ),
        With<ChasePlayerHitbox>,
    >,
    mut bullet_query: Query<
        (
            Entity,
            &GlobalTransform,
            &crate::core::collision::TriggerCollider,
            &crate::core::danmaku::BulletDamage,
            &crate::core::danmaku::BulletHitBehavior,
            &mut crate::core::danmaku::BulletLastHitTime,
            &crate::core::danmaku::BulletMotionState,
        ),
        With<crate::core::danmaku::Bullet>,
    >,
    mut damage_events: MessageWriter<ChasePlayerDamageEvent>,
) {
    // Only run in chase state
    if *overworld_state.get() != OverworldState::Chase {
        return;
    }

    let Ok((player_transform, player_hitbox, hitbox_offset, is_walking, is_running)) =
        player_query.single()
    else {
        return;
    };

    let player_center = player_transform.translation.truncate()
        + hitbox_offset
            .map(|o| o.0)
            .unwrap_or(chase_config.hitbox.offset.to_vec2());

    // Check if player is moving
    let player_is_moving = is_walking.is_some() || is_running.is_some();

    // Check if player is invincible
    let is_invincible = player_invincibility.is_invincible();

    for (
        bullet_entity,
        bullet_transform,
        bullet_collider,
        bullet_damage,
        hit_behavior,
        mut last_hit_time,
        motion_state,
    ) in bullet_query.iter_mut()
    {
        let bullet_center = bullet_transform.translation().truncate();

        // Check collision between player hitbox and bullet collider
        if !check_trigger_collision(player_hitbox, player_center, bullet_collider, bullet_center) {
            continue;
        }

        // Check bullet's own invincibility frames (for persistent bullets)
        if hit_behavior.invincibility_duration > 0.0 {
            let time_since_last_hit = motion_state.elapsed - last_hit_time.0;
            if time_since_last_hit < hit_behavior.invincibility_duration {
                continue;
            }
        }

        // Check movement-based damage conditions
        let should_damage = if hit_behavior.damage_on_player_moving {
            // "Blue soul" style: only damage when player is moving
            player_is_moving
        } else if hit_behavior.damage_on_player_stationary {
            // "Orange soul" style: only damage when player is stationary
            !player_is_moving
        } else {
            // Default: always damage
            true
        };

        // If player is invincible, don't deal damage but still handle despawn
        if is_invincible {
            // Handle despawn behavior even during invincibility
            if hit_behavior.despawn_on_hit && should_damage {
                commands
                    .entity(bullet_entity)
                    .insert(crate::core::danmaku::DespawnBullet);
            }
            continue;
        }

        if should_damage {
            // Update last hit time
            last_hit_time.0 = motion_state.elapsed;

            // Apply damage to player HP (fixed integer damage)
            let damage = bullet_damage.0 as usize;
            if player_data.hp > damage {
                player_data.hp -= damage;
            } else {
                player_data.hp = 0;
            }

            // Fire damage event
            damage_events.write(ChasePlayerDamageEvent {
                damage: bullet_damage.0,
            });

            // Start player invincibility
            player_invincibility.start(player_behavior.invincibility.duration);

            // Play hurt sound
            #[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
            crate::core::audio::play_sound(&audio, &asset_server, "hurtsound.wav");
            #[cfg(feature = "firewheel")]
            crate::core::audio::play_sound(&mut commands, &asset_server, "hurtsound.wav");

            info!(
                "Chase: Player hit! Damage: {}, HP: {}/{}",
                damage, player_data.hp, player_data.hp_max
            );

            // Handle despawn behavior
            if hit_behavior.despawn_on_hit {
                commands
                    .entity(bullet_entity)
                    .insert(crate::core::danmaku::DespawnBullet);
            }

            // Only one bullet can deal damage per frame
            break;
        }
    }
}

/// Helper function to check collision between two trigger colliders.
///
/// 检查两个触发器碰撞体之间是否发生碰撞的辅助函数。
fn check_trigger_collision(
    a: &crate::core::collision::TriggerCollider,
    a_center: Vec2,
    b: &crate::core::collision::TriggerCollider,
    b_center: Vec2,
) -> bool {
    use crate::core::collision::TriggerCollider;

    match (a, b) {
        (TriggerCollider::Circle { radius: r1 }, TriggerCollider::Circle { radius: r2 }) => {
            let dist = a_center.distance(b_center);
            dist <= r1 + r2
        }
        (TriggerCollider::Box { half_size: hs1 }, TriggerCollider::Box { half_size: hs2 }) => {
            let diff = (a_center - b_center).abs();
            diff.x <= hs1.x + hs2.x && diff.y <= hs1.y + hs2.y
        }
        (TriggerCollider::Circle { radius }, TriggerCollider::Box { half_size })
        | (TriggerCollider::Box { half_size }, TriggerCollider::Circle { radius }) => {
            let (circle_center, box_center, box_half) =
                if matches!(a, TriggerCollider::Circle { .. }) {
                    (a_center, b_center, *half_size)
                } else {
                    (b_center, a_center, *half_size)
                };
            // Closest point on box to circle center
            let closest = Vec2::new(
                circle_center
                    .x
                    .clamp(box_center.x - box_half.x, box_center.x + box_half.x),
                circle_center
                    .y
                    .clamp(box_center.y - box_half.y, box_center.y + box_half.y),
            );
            circle_center.distance(closest) <= *radius
        }
    }
}

/// System to update player invincibility timer and heart flashing effect.
///
/// 更新玩家无敌时间和心形闪烁效果的系统。
pub fn update_player_invincibility_system(
    time: Res<Time>,
    chase_config: Res<ChaseConfig>,
    player_behavior: Res<crate::app_state::overworld::player::config::PlayerBehavior>,
    overworld_state: Res<State<OverworldState>>,
    mut player_invincibility: ResMut<PlayerInvincibility>,
    mut heart_markers: Query<&mut Sprite, With<ChaseHeartMarker>>,
) {
    // Only run in chase state
    if *overworld_state.get() != OverworldState::Chase {
        return;
    }

    if !player_invincibility.active {
        return;
    }

    let delta = time.delta_secs();
    let config = &player_behavior.invincibility;

    // Update invincibility timer
    player_invincibility.timer -= delta;

    if player_invincibility.timer <= 0.0 {
        // Invincibility ended - reset to normal color
        player_invincibility.active = false;
        player_invincibility.timer = 0.0;
        player_invincibility.flash_state = true;

        // Reset heart color to pure red
        for mut sprite in heart_markers.iter_mut() {
            let heart_config = &chase_config.heart_marker;
            sprite.color = Color::srgba(
                heart_config.color.r,
                heart_config.color.g,
                heart_config.color.b,
                heart_config.color.a,
            );
        }

        info!("Chase: Player invincibility ended");
        return;
    }

    // Update flash timer
    player_invincibility.flash_timer += delta;

    if player_invincibility.flash_timer >= config.flash_interval {
        player_invincibility.flash_timer = 0.0;
        player_invincibility.flash_state = !player_invincibility.flash_state;

        // Toggle heart color
        let color = if player_invincibility.flash_state {
            // Normal color (pure red)
            parse_hex_color_for_heart(&config.normal_color)
        } else {
            // Flash color (dark red)
            parse_hex_color_for_heart(&config.flash_color)
        };

        for mut sprite in heart_markers.iter_mut() {
            if let Some(c) = color {
                sprite.color = c;
            }
        }
    }
}

/// Parse hex color string for heart color.
fn parse_hex_color_for_heart(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::srgb_u8(r, g, b))
        }
        _ => None,
    }
}

/// System to handle damage UI display.
/// Now simplified since HP bar HUD provides primary damage feedback.
/// This only handles a brief screen tint effect on damage.
///
/// 处理受伤UI显示的系统。
/// 由于血条 HUD 提供了主要的受伤反馈，此系统已简化。
/// 仅处理受伤时的短暂屏幕色调效果。
pub fn damage_ui_display_system(
    mut commands: Commands,
    time: Res<Time>,
    mut damage_ui_state: ResMut<DamageUIState>,
    mut damage_events: MessageReader<ChasePlayerDamageEvent>,
    damage_ui_query: Query<Entity, With<DamageUIMarker>>,
    camera_query: Query<&Transform, With<Camera2d>>,
) {
    // Check for new damage events
    for _event in damage_events.read() {
        if !damage_ui_state.showing {
            // Get camera position to center the overlay
            let camera_pos = camera_query
                .single()
                .map(|t| t.translation)
                .unwrap_or(Vec3::ZERO);

            // Spawn a simple red overlay sprite without requiring a texture
            // The HP bar HUD now provides the primary damage feedback
            commands.spawn((
                Sprite {
                    color: Color::srgba(1.0, 0.0, 0.0, 0.3),
                    custom_size: Some(Vec2::new(640.0, 480.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(camera_pos.x, camera_pos.y, 500.0)),
                OverworldEntity(),
                DamageUIMarker,
                Name::new("DamageFlashOverlay"),
            ));
            damage_ui_state.showing = true;
            damage_ui_state.timer = 0.1; // Short flash duration
            info!("Chase: Showing damage flash overlay");
        } else {
            // Reset timer if already showing
            damage_ui_state.timer = 0.1;
        }
    }

    // Update timer and hide UI when done
    if damage_ui_state.showing {
        damage_ui_state.timer -= time.delta_secs();
        if damage_ui_state.timer <= 0.0 {
            // Despawn damage UI
            for entity in damage_ui_query.iter() {
                commands.entity(entity).despawn();
            }
            damage_ui_state.showing = false;
            info!("Chase: Hiding damage flash overlay");
        }
    }
}

/// Marker component for damage UI entities.
///
/// 受伤UI实体的标记组件。
#[derive(Component)]
pub struct DamageUIMarker;

/// Marker component for Chase HUD UI root entity.
///
/// 追逐战 HUD UI 根实体的标记组件。
#[derive(Component)]
pub struct ChaseHUDRoot;

/// System to setup Chase HUD when entering chase state.
/// Loads the damage_flash.ui_layout.ron which contains HP bar and HP text.
///
/// 进入追逐战状态时设置 HUD 的系统。
/// 加载包含血条和血量文字的 damage_flash.ui_layout.ron。
fn setup_chase_hud_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Chase: Setting up Chase HUD");

    // Load the chase HUD UI layout
    let handle = asset_server.load("overworld/ui/damage_flash.ui_layout.ron");

    // Insert the UI layout handle resource
    commands.insert_resource(crate::core::ui::UILayoutHandle {
        handle,
        last_modified: None,
    });

    // Spawn a root entity for the RON UI system to attach to
    commands.spawn((
        crate::core::ui::components::RonUI::new(
            crate::core::ui::components::UILayer::new("ChaseHUD"),
            0,
        ),
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        OverworldEntity(),
        ChaseHUDRoot,
        Name::new("ChaseHUD Root"),
    ));

    // Insert UILayoutWatcher with pending_reload = false to ensure clean state
    // This prevents double UI spawning from rebuild_reloaded_ui_system
    // 插入 pending_reload = false 的 UILayoutWatcher 以确保干净的状态
    // 这可以防止 rebuild_reloaded_ui_system 导致的双重 UI 生成
    commands.insert_resource(crate::core::ui::UILayoutWatcher::new());

    info!("Chase: Chase HUD setup complete");
}

/// System to cleanup Chase HUD when exiting chase state.
///
/// 退出追逐战状态时清理 HUD 的系统。
fn cleanup_chase_hud_system(
    mut commands: Commands,
    chase_hud_query: Query<Entity, With<ChaseHUDRoot>>,
    ron_driven_ui_query: Query<Entity, With<crate::core::ui::RonDrivenUI>>,
) {
    info!("Chase: Cleaning up Chase HUD");

    // Despawn the Chase HUD root and all RON-driven UI entities
    for entity in chase_hud_query.iter() {
        commands.entity(entity).despawn();
    }

    // Also despawn any RON-driven UI entities that may have been spawned
    for entity in ron_driven_ui_query.iter() {
        commands.entity(entity).despawn();
    }

    // Remove the UI layout handle resource
    commands.remove_resource::<crate::core::ui::UILayoutHandle>();
    commands.remove_resource::<crate::core::ui::UILayoutWatcher>();

    info!("Chase: Chase HUD cleanup complete");
}
