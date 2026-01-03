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
//! - Red 1-pixel outline on player sprite using custom shader
//! All effects have a 0.5 second alpha transition.
//!
//! 本模块实现 Overworld 的追逐战状态视觉效果。
//! 进入追逐战状态时，将应用以下效果：
//! - 全屏 0.5 透明度的黑色覆盖层
//! - 玩家精灵使用自定义着色器的红色1像素描边
//! 所有效果都有 0.5 秒的透明度过渡。

use bevy::image::TextureAtlasLayout;
use bevy::prelude::*;
use bevy::sprite_render::MeshMaterial2d;
use bevy_smud::prelude::*;

use crate::app_state::overworld::character::components::PlayerControlled;
use crate::app_state::overworld::{OverworldEntity, OverworldState, OverworldUpdate};
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

/// Marker for the player outline entity.
///
/// 玩家描边实体的标记。
#[derive(Component)]
pub struct ChasePlayerOutline;

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

const TRANSITION_DURATION: f32 = 0.5;
const OVERLAY_TARGET_ALPHA: f32 = 0.5;

pub struct ChasePlugin;

impl Plugin for ChasePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChaseTransition>()
            .add_systems(
                OnEnter(OverworldState::Chase),
                (
                    setup_chase_effect_root_system,
                    start_chase_transition_in_system,
                ),
            )
            .add_systems(
                OnExit(OverworldState::Chase),
                start_chase_transition_out_system,
            )
            .add_systems(
                Update,
                (
                    update_chase_transition_system,
                    spawn_chase_dark_overlay_system,
                    spawn_player_outline_system,
                    update_player_outline_system,
                    update_chase_effect_alpha_system,
                    cleanup_chase_effects_system,
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
fn start_chase_transition_out_system(mut transition: ResMut<ChaseTransition>) {
    transition.transitioning_in = false;
    transition.timer = TRANSITION_DURATION;
    info!("Chase: Starting transition OUT");
}

/// Update chase transition timer.
///
/// 更新追逐战过渡计时器。
fn update_chase_transition_system(time: Res<Time>, mut transition: ResMut<ChaseTransition>) {
    if !transition.active && !transition.transitioning_in && transition.timer <= 0.0 {
        return;
    }

    if transition.transitioning_in {
        transition.timer = (transition.timer + time.delta_secs()).min(TRANSITION_DURATION);
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
                target_alpha: OVERLAY_TARGET_ALPHA,
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
            ChasePlayerOutline,
            ChaseEffect { target_alpha: 1.0 },
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_translation(player_transform.translation + Vec3::new(0.0, 0.0, 100.0)),
            Name::new("ChasePlayerOutline"),
        ));
    });

    info!("Chase: Spawned player outline");
}

/// Update player outline position and texture to follow player sprite.
///
/// 更新玩家描边位置和纹理以跟随玩家精灵。
#[allow(clippy::type_complexity)]
fn update_player_outline_system(
    mut outline_query: Query<
        (&mut Transform, &MeshMaterial2d<PixelOutlineMaterial>),
        (With<ChasePlayerOutline>, Without<PlayerControlled>),
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

    for (mut outline_transform, material_handle) in outline_query.iter_mut() {
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

            // Update UV rect based on TextureAtlas or sprite.rect
            if let Some(image) = images.get(&sprite.image) {
                let tex_size = image.size().as_vec2();
                material.uv_rect = if let Some(ref atlas) = sprite.texture_atlas {
                    if let Some(layout) = atlas_layouts.get(&atlas.layout) {
                        let rect = layout.textures[atlas.index];
                        Vec4::new(
                            rect.min.x as f32 / tex_size.x,
                            rect.min.y as f32 / tex_size.y,
                            rect.max.x as f32 / tex_size.x,
                            rect.max.y as f32 / tex_size.y,
                        )
                    } else {
                        Vec4::new(0.0, 0.0, 1.0, 1.0)
                    }
                } else if let Some(rect) = sprite.rect {
                    Vec4::new(
                        rect.min.x / tex_size.x,
                        rect.min.y / tex_size.y,
                        rect.max.x / tex_size.x,
                        rect.max.y / tex_size.y,
                    )
                } else {
                    Vec4::new(0.0, 0.0, 1.0, 1.0)
                };
            }
        }
    }
}

/// Update alpha values for all chase effects based on transition progress.
///
/// 根据过渡进度更新所有追逐战效果的透明度。
fn update_chase_effect_alpha_system(
    transition: Res<ChaseTransition>,
    mut smud_effects: Query<(&mut SmudShape, &ChaseEffect)>,
    mut outline_query: Query<&MeshMaterial2d<PixelOutlineMaterial>, With<ChasePlayerOutline>>,
    mut materials: ResMut<Assets<PixelOutlineMaterial>>,
) {
    let progress = transition.timer / TRANSITION_DURATION;

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

/// Cleanup chase effects when transition out is complete.
///
/// 当退出过渡完成时清理追逐战效果。
fn cleanup_chase_effects_system(
    mut commands: Commands,
    mut transition: ResMut<ChaseTransition>,
    chase_root: Query<Entity, With<ChaseEffectRoot>>,
    overlays: Query<Entity, With<ChaseDarkOverlay>>,
    outlines: Query<Entity, With<ChasePlayerOutline>>,
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

    // Despawn root
    if let Ok(root) = chase_root.single() {
        commands.entity(root).despawn();
    }

    transition.cleanup_done = true;
    info!("Chase: Cleaned up all effects");
}
