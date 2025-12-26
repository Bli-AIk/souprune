//! # procedural_textures.rs
//!
//! Procedurally generated textures for UI elements.
//! 程序生成的UI元素纹理。

use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// Resource that holds handles to procedurally generated textures.
///
/// 保存程序生成纹理的句柄资源。
#[derive(Resource)]
pub struct ProceduralTextures {
    pub white_pixel: Handle<Image>,
}

/// Initialize procedural textures.
///
/// 初始化程序生成的纹理。
pub fn init_procedural_textures(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // Create 1x1 white pixel texture
    let mut white_pixel = Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[255, 255, 255, 255], // RGBA white
        TextureFormat::Rgba8UnormSrgb,
        Default::default(), // Use default RenderAssetUsages
    );

    let white_pixel_handle = images.add(white_pixel);

    commands.insert_resource(ProceduralTextures {
        white_pixel: white_pixel_handle,
    });

    info!("Procedural textures initialized");
}
