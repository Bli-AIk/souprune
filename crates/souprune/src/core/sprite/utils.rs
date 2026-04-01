//! # utils.rs
//!
//! Texture atlas building and caching utilities.
//!
//! 纹理图集构建与缓存工具。

use crate::core::sprite::resources::ModuleSpriteRegistry;
use anyhow::{Result, anyhow};
use bevy::asset::{Assets, Handle, LoadedFolder, RenderAssetUsages};
use bevy::image::{
    Image, ImageSampler, TextureAtlasBuilder, TextureAtlasLayout, TextureAtlasSources,
};
use bevy::log::warn;
use bevy::math::UVec2;
use bevy::platform::collections::HashMap;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub fn get_or_create_texture_atlas(
    module_name: &str,
    sprite_registry: &mut ModuleSpriteRegistry,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    loaded_folders: &Assets<LoadedFolder>,
    textures: &mut Assets<Image>,
) -> Result<(
    Handle<TextureAtlasLayout>,
    Handle<Image>,
    HashMap<String, usize>,
)> {
    if let Some((atlas_layout, texture, index_map)) = sprite_registry.get_cached_atlas(module_name)
    {
        return Ok((atlas_layout.clone(), texture.clone(), index_map.clone()));
    }

    let handle = sprite_registry
        .get_module(module_name)
        .ok_or_else(|| anyhow!("{module_name} module not registered"))?;

    let loaded_folder = loaded_folders
        .get(handle)
        .ok_or_else(|| anyhow!("Failed to get loaded folder for module {module_name}"))?;

    let (texture_atlas_layout, _texture_atlas_sources, texture, index_map) = create_texture_atlas(
        loaded_folder,
        Some(UVec2::new(1, 1)),
        Some(ImageSampler::nearest()),
        textures,
    )?;

    let atlas_layout_handle = texture_atlases.add(texture_atlas_layout);

    sprite_registry.cache_atlas(
        module_name.to_string(),
        atlas_layout_handle.clone(),
        texture.clone(),
        index_map.clone(),
    );

    Ok((atlas_layout_handle, texture, index_map))
}

pub fn create_texture_atlas(
    folder: &LoadedFolder,
    padding: Option<UVec2>,
    sampling: Option<ImageSampler>,
    textures: &mut Assets<Image>,
) -> Result<(
    TextureAtlasLayout,
    TextureAtlasSources,
    Handle<Image>,
    HashMap<String, usize>,
)> {
    let mut texture_atlas_builder = TextureAtlasBuilder::default();
    texture_atlas_builder.padding(padding.unwrap_or_default());

    let mut index_map = HashMap::new();
    let mut added_count = 0;

    for handle in folder.handles.iter() {
        let Some(path) = handle.path() else {
            let id = handle.id().typed_unchecked::<Image>();
            let Some(texture) = textures.get(id) else {
                warn!(
                    "{} did not resolve to an `Image` asset.",
                    handle.path().map(|p| p.to_string()).unwrap_or_default()
                );
                continue;
            };
            texture_atlas_builder.add_texture(Some(id), texture);
            continue;
        };

        let path_str = path.to_string().replace('\\', "/");
        let is_image =
            path_str.ends_with(".png") || path_str.ends_with(".jpg") || path_str.ends_with(".jpeg");

        if !is_image {
            continue;
        }
        index_map.insert(path_str, added_count);
        added_count += 1;

        let id = handle.id().typed_unchecked::<Image>();
        let Some(texture) = textures.get(id) else {
            warn!(
                "{} did not resolve to an `Image` asset.",
                handle.path().map(|p| p.to_string()).unwrap_or_default()
            );
            continue;
        };

        texture_atlas_builder.add_texture(Some(id), texture);
    }

    let (texture_atlas_layout, texture_atlas_sources, texture) = texture_atlas_builder
        .build()
        .map_err(|e| anyhow!("Failed to build texture atlas: {e}"))?;
    let texture = textures.add(texture);

    let image = textures
        .get_mut(&texture)
        .ok_or_else(|| anyhow!("Failed to access built texture"))?;
    image.sampler = sampling.unwrap_or_default();

    Ok((
        texture_atlas_layout,
        texture_atlas_sources,
        texture,
        index_map,
    ))
}

pub fn get_or_create_missing_texture(
    sprite_registry: &mut ModuleSpriteRegistry,
    textures: &mut Assets<Image>,
) -> Handle<Image> {
    if let Some(handle) = &sprite_registry.missing_texture {
        return handle.clone();
    }

    let size = Extent3d {
        width: 16,
        height: 16,
        depth_or_array_layers: 1,
    };

    let mut data = Vec::with_capacity((size.width * size.height * 4) as usize);

    for y in 0..size.height {
        for x in 0..size.width {
            let is_purple = ((x / 8) + (y / 8)) % 2 == 0;
            if is_purple {
                data.extend_from_slice(&[255, 0, 255, 255]);
            } else {
                data.extend_from_slice(&[0, 0, 0, 255]);
            }
        }
    }

    let mut image = Image::new(
        size,
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();

    let handle = textures.add(image);
    sprite_registry.missing_texture = Some(handle.clone());

    handle
}
