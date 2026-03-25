//! # reveal.rs
//!
//! # reveal.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module implements a tile reveal effect for the tilemap.
//! Tiles are spawned as pure white sprites that animate from small to large, creating a
//! ripple effect from a center point. After the sprite animation, the original tile is shown
//! and gradually fades from white to normal B&W rendering using TileColor.
//!
//! 本模块实现了瓦片地图的揭示效果。
//! 瓦片作为纯白色精灵生成，从小到大播放动画，从中心点向外产生涟漪效果。
//! 精灵动画完成后，显示原始瓦片并通过 TileColor 从纯白渐变到正常黑白渲染。
//!
//! ## Fade Value Semantics
//!
//! ## 淡入值语义
//!
//! TileColor.r is used as fade value (passed to shader via in.color.r):
//! - 0.0 = pure black
//! - 0.5 = normal black & white rendering
//! - 1.0 = pure white
//!
//! TileColor.r 用作淡入值（通过 in.color.r 传递给着色器）：
//! - 0.0 = 纯黑
//! - 0.5 = 正常黑白渲染
//! - 1.0 = 纯白
//!
//! ## Experimental Feature
//!
//! ## 实验功能
//!
//! This module only runs when the `experimental` feature is enabled.
//!
//! 本模块仅在启用 `experimental` feature 时运行。

mod animation;
mod setup;

use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};
use bevy_ecs_tiled::prelude::*;
use bevy_tween::interpolate::Interpolator;
use bevy_tween::prelude::*;
use std::collections::HashMap;

// ========== FADE CONFIGURATION ==========
// 淡入配置

/// Number of eighth notes to fade from 1.0 (white) to 0.5 (normal B&W).
/// 32 eighth notes = 8 quarter notes (beats) = 2 bars in 4/4 time.
/// 从 1.0（白色）淡入到 0.5（正常黑白）所需的八分音符数。
/// 32 个八分音符 = 8 个四分音符（拍）= 4/4 拍中的 2 小节。
pub const FADE_EIGHTH_NOTES: u32 = 32;

/// Target fade value (normal B&W rendering).
/// 目标淡入值（正常黑白渲染）。
pub const FADE_TARGET: f32 = 0.5;

/// Initial fade value (pure white).
/// 初始淡入值（纯白）。
pub const FADE_INITIAL: f32 = 1.0;

// ========== BLACK/WHITE THRESHOLD ==========
// This constant is used for the BlackWhiteMaterial (if used for sprite shader).
// Currently sprites use pure white, so this is only kept for reference.
// Lower value = more white, higher value = more black.
// 此常量用于 BlackWhiteMaterial（如果用于精灵着色器）。
// 目前精灵使用纯白色，因此这里仅保留供参考。
// 较低的值 = 更多白色，较高的值 = 更多黑色。
#[allow(dead_code)]
pub const BLACK_WHITE_THRESHOLD: f32 = 0.125;

/// Marker component for revealed tile sprites.
/// These sprites are now pure white for the ripple effect.
///
/// 已揭示的瓦片精灵的标记组件。
/// 这些精灵现在是纯白色，用于涟漪效果。
#[derive(Component)]
pub struct RevealedTileSprite {
    /// Manhattan distance from the reveal origin.
    ///
    /// 距离揭示原点的曼哈顿距离。
    pub manhattan_distance: u32,
    /// Tile position (x, y) for matching with original tiles.
    ///
    /// 瓦片位置 (x, y)，用于与原始瓦片匹配。
    pub tile_pos: (u32, u32),
    /// Direction from origin to this tile.
    ///
    /// 从原点到此瓦片的方向。
    pub direction: RippleDirection,
}

/// Component to track tile fade state.
/// Attached to original TiledTile entities after their sprite animation completes.
///
/// 跟踪瓦片淡入状态的组件。
/// 在精灵动画完成后附加到原始 TiledTile 实体。
#[derive(Component)]
pub struct TileFadeState {
    /// Current fade value (1.0 = white, 0.5 = normal, 0.0 = black).
    ///
    /// 当前淡入值（1.0 = 白色，0.5 = 正常，0.0 = 黑色）。
    pub fade: f32,
    /// Random offset for stepped fade (0-31 eighth notes).
    /// This creates a "noise" effect as tiles don't all transition at once.
    ///
    /// 步进淡入的随机偏移（0-31 个八分音符）。
    /// 这会产生"噪点"效果，因为瓦片不会同时转换。
    pub random_offset: u32,
    /// Count of eighth notes processed since this tile started fading.
    ///
    /// 自此瓦片开始淡入以来处理的八分音符数。
    pub eighth_notes_elapsed: u32,
}

/// Marker component for tiles that are currently animating.
///
/// 当前正在动画的瓦片的标记组件。
#[derive(Component)]
pub struct AnimatingTile {
    /// Timer to track when animation is complete.
    ///
    /// 跟踪动画完成的计时器。
    pub animation_timer: Timer,
}

/// Marker component for tiles that have finished animating.
///
/// 已完成动画的瓦片的标记组件。
#[derive(Component)]
pub struct AnimationComplete;

/// Direction for ripple expansion.
/// 涟漪扩展方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RippleDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Marker component for the reveal tiles parent container.
/// All RevealTile entities are children of this entity for cleaner hierarchy.
///
/// 揭示瓦片父容器的标记组件。
/// 所有 RevealTile 实体都是此实体的子实体，使层级结构更整洁。
#[derive(Component)]
pub struct RevealTilesRoot;

impl RippleDirection {
    /// Get all four directions.
    /// 获取所有四个方向。
    pub fn all() -> [RippleDirection; 4] {
        [
            RippleDirection::Up,
            RippleDirection::Down,
            RippleDirection::Left,
            RippleDirection::Right,
        ]
    }

    /// Determine the primary direction from a tile delta relative to the origin.
    /// For tiles at the origin (0, 0), defaults to Up.
    fn from_delta(dx: i32, dy: i32) -> Self {
        if dx == 0 && dy == 0 {
            return RippleDirection::Up;
        }
        if dx.abs() >= dy.abs() {
            if dx > 0 {
                RippleDirection::Right
            } else {
                RippleDirection::Left
            }
        } else if dy > 0 {
            RippleDirection::Up
        } else {
            RippleDirection::Down
        }
    }
}

/// Resource to track the reveal animation state.
///
/// 跟踪揭示动画状态的资源。
#[derive(Resource)]
pub struct TileRevealState {
    /// Origin position for the ripple effect (world coordinates).
    ///
    /// 涟漪效果的原点位置（世界坐标）。
    pub origin: Vec2,

    /// Current step of the reveal (Manhattan distance threshold).
    ///
    /// 当前揭示步骤（曼哈顿距离阈值）。
    pub current_step: u32,

    /// Maximum Manhattan distance to reveal.
    ///
    /// 要揭示的最大曼哈顿距离。
    pub max_distance: u32,

    /// Whether the reveal animation triggering is complete.
    ///
    /// 揭示动画触发是否完成。
    pub all_triggered: bool,

    /// Whether initialization is done.
    ///
    /// 初始化是否完成。
    pub initialized: bool,

    /// Whether all original tiles have been made visible initially.
    ///
    /// 是否已将所有原始瓦片初始设置为不可见。
    pub tiles_hidden: bool,

    /// Directions that have been used in the current step.
    /// A direction can only be used again after all four directions have been used.
    ///
    /// 当前步骤中已使用的方向。
    /// 只有在四个方向都使用后，才能再次使用某个方向。
    pub used_directions: Vec<RippleDirection>,

    /// Pending tiles to reveal for the current step, organized by direction.
    /// This stores (direction, entity) pairs for tiles that should be revealed next.
    ///
    /// 当前步骤待揭示的瓦片，按方向组织。
    /// 存储 (方向, 实体) 对，表示下一个要揭示的瓦片。
    pub pending_tiles_by_direction: HashMap<RippleDirection, Vec<Entity>>,
}

impl Default for TileRevealState {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            current_step: 0,
            max_distance: 0,
            all_triggered: false,
            initialized: false,
            tiles_hidden: false,
            used_directions: Vec::new(),
            pending_tiles_by_direction: HashMap::new(),
        }
    }
}

/// Resource to store tilemap texture for black/white rendering.
///
/// 存储瓦片地图纹理以用于黑白渲染的资源。
#[derive(Resource, Default)]
pub struct TilemapTextureCache {
    pub textures: Vec<Handle<Image>>,
    pub tile_size: Vec2,
}

/// Black and white shader material for tiles.
/// Converts tile textures to pure black (#000000) and white (#FFFFFF).
///
/// 瓦片的黑白着色器材质。
/// 将瓦片纹理转换为纯黑色 (#000000) 和纯白色 (#FFFFFF)。
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BlackWhiteMaterial {
    // ========== BLACK/WHITE THRESHOLD ==========
    // Threshold for black/white conversion.
    // Lower value = more white, higher value = more black.
    // Pixels with luminance ABOVE this become WHITE, BELOW become BLACK.
    // 黑白转换的阈值。
    // 较低的值 = 更多白色，较高的值 = 更多黑色。
    // 亮度高于此值的像素变为白色，低于的变为黑色。
    #[uniform(0)]
    pub threshold: f32,

    /// Base texture.
    ///
    /// 基础纹理。
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,

    /// UV rect for this tile in the atlas (x, y, width, height).
    ///
    /// 此瓦片在图集中的 UV 矩形 (x, y, width, height)。
    #[uniform(3)]
    pub uv_rect: Vec4,
}

impl Material2d for BlackWhiteMaterial {
    fn fragment_shader() -> ShaderRef {
        // TODO: Refactor to use a more generic shader system that allows
        // user-configurable shader paths through RON configuration.
        // This hardcoded path should be replaced with a data-driven approach
        // similar to DynamicMaterial2d's material.shader field.
        //
        // TODO: 重构为更通用的着色器系统，允许用户通过 RON 配置指定着色器路径。
        // 此硬编码路径应替换为类似 DynamicMaterial2d 的 material.shader 字段的数据驱动方式。
        "shaders/black_white_tile.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Black and white material for bevy_ecs_tilemap.
/// Uses a custom fragment shader to convert tilemap to pure black and white.
///
/// bevy_ecs_tilemap 的黑白材质。
/// 使用自定义片段着色器将瓦片地图转换为纯黑白。
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct BlackWhiteTilemapMaterial {}

impl MaterialTilemap for BlackWhiteTilemapMaterial {
    fn fragment_shader() -> ShaderRef {
        // TODO: Refactor to use a more generic shader system that allows
        // user-configurable shader paths through RON configuration.
        //
        // TODO: 重构为更通用的着色器系统，允许用户通过 RON 配置指定着色器路径。
        "shaders/black_white_tilemap.wgsl".into()
    }
}

/// Marker component for tilemaps that have been given the black/white material.
///
/// 已应用黑白材质的瓦片地图的标记组件。
#[derive(Component)]
pub struct HasBlackWhiteMaterial;

/// Plugin for the tile reveal effect.
///
/// 瓦片揭示效果的插件。
pub struct TileRevealPlugin;

impl Plugin for TileRevealPlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        app.add_plugins(Material2dPlugin::<BlackWhiteMaterial>::default())
            .add_plugins(MaterialTilemapPlugin::<BlackWhiteTilemapMaterial>::default())
            .init_resource::<TileRevealState>()
            .init_resource::<TilemapTextureCache>()
            .add_tween_systems(
                PostUpdate,
                bevy_tween::tween::component_tween_system::<ScaleInterpolator>(),
            )
            .add_systems(
                schedule,
                (
                    setup::apply_black_white_tilemap_material_system,
                    setup::cache_tilemap_textures_system,
                    setup::initialize_tile_colors_system,
                    setup::hide_all_original_tiles_system,
                    setup::create_tile_sprites_system,
                    animation::collect_pending_tiles_system,
                    animation::update_reveal_animation_system,
                    animation::check_animation_complete_system,
                    animation::cleanup_completed_sprites_system,
                    animation::update_tile_fade_system,
                )
                    .chain()
                    .in_set(super::super::OverworldUpdate),
            );
    }
}

/// Scale interpolator for tile reveal animation.
///
/// 瓦片揭示动画的缩放插值器。
#[derive(Debug, Clone)]
pub struct ScaleInterpolator {
    pub start: Vec3,
    pub end: Vec3,
}

impl Interpolator for ScaleInterpolator {
    type Item = Transform;

    fn interpolate(&self, item: &mut Self::Item, value: f32, _previous_value: f32) {
        item.scale = self.start.lerp(self.end, value);
    }
}
