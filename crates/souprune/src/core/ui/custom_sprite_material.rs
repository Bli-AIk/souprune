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
