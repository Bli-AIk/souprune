//! # custom_sprite_material.rs
//!
//! Custom Material2d for sprite rendering with shader support.
//! 支持着色器的精灵渲染自定义 Material2d。
//!
//! Generic sprite material system for shader-based effects.
//!
//! 用于着色器效果的通用精灵材质系统。

use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};

/// Custom sprite material with shader parameters.
///
/// 带有着色器参数的自定义精灵材质。
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CustomSpriteMaterial {
    /// Shader uniform parameters (vec4).
    ///
    /// 着色器 uniform 参数 (vec4)。
    #[uniform(0)]
    pub color_params: LinearRgba,

    /// Base texture.
    ///
    /// 基础纹理。
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl Material2d for CustomSpriteMaterial {
    fn fragment_shader() -> ShaderRef {
        // HP bar shader with UV-based gradient
        "shared/shaders/hp_bar_sprite.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

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
        "shared/shaders/pixel_outline.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
