//! # custom_sprite_material.rs
//!
//! Custom Material2d for sprite rendering with shader support.
//! 支持着色器的精灵渲染自定义 Material2d。
//!
//! This module contains PixelOutlineMaterial for chase state highlight effects.
//! HP bar materials have been replaced by the DynamicMaterial2d system.
//!
//! 该模块包含追逐状态高亮效果的 PixelOutlineMaterial。
//! HP 条材质已被 DynamicMaterial2d 系统取代。

use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};

/// Pixel outline material for chase state highlight effect.
/// Uses a shader that creates 1-pixel red outline around opaque pixels.
///
/// 追逐战状态高亮效果的像素描边材质。
/// 使用着色器在不透明像素周围创建1像素红色描边。
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PixelOutlineMaterial {
    /// Outline parameters (rgba): rgb = outline color, a = outline alpha.
    ///
    /// 描边参数 (rgba): rgb = 描边颜色, a = 描边透明度。
    #[uniform(0)]
    pub params: LinearRgba,

    /// UV rect for atlas sprites (x=min_u, y=min_v, z=max_u, w=max_v).
    /// Use (0,0,1,1) for full texture.
    ///
    /// 图集精灵的 UV 矩形 (x=min_u, y=min_v, z=max_u, w=max_v)。
    /// 完整纹理使用 (0,0,1,1)。
    #[uniform(1)]
    pub uv_rect: Vec4,

    /// Flip flags (x=flip_x, y=flip_y, z=unused, w=unused).
    /// 0.0 = no flip, 1.0 = flip.
    ///
    /// 翻转标志 (x=flip_x, y=flip_y, z=未使用, w=未使用)。
    /// 0.0 = 不翻转, 1.0 = 翻转。
    #[uniform(2)]
    pub flip: Vec4,

    /// Base texture (the sprite to outline).
    ///
    /// 基础纹理（要描边的精灵）。
    #[texture(3)]
    #[sampler(4)]
    pub texture: Handle<Image>,
}

impl Material2d for PixelOutlineMaterial {
    fn fragment_shader() -> ShaderRef {
        // TODO: Refactor to use a more generic shader system that allows
        // user-configurable shader paths through RON configuration.
        // This hardcoded path should be replaced with a data-driven approach
        // similar to DynamicMaterial2d's material.shader field.
        //
        // TODO: 重构为更通用的着色器系统，允许用户通过 RON 配置指定着色器路径。
        // 此硬编码路径应替换为类似 DynamicMaterial2d 的 material.shader 字段的数据驱动方式。
        "assets/shaders/pixel_outline.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
