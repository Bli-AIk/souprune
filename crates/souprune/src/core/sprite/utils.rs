use crate::core::sprite::resources::ModuleSpriteRegistry;
use crate::extra::toml::TomlAsset;
use crate::extra::toml::config::TomlConfigRegistry;
use anyhow::{Result, anyhow};
use bevy::asset::{Assets, Handle, LoadedFolder};
use bevy::image::{
    Image, ImageSampler, TextureAtlas, TextureAtlasBuilder, TextureAtlasLayout, TextureAtlasSources,
};
use bevy::log::{info, warn};
use bevy::math::{UVec2, Vec3};
use bevy::platform::collections::HashMap;
use bevy::prelude::{Commands, Sprite, Transform, default};

pub fn get_or_create_texture_atlas(
    module_name: &str,
    sprite_registry: &mut ModuleSpriteRegistry,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    loaded_folders: &Assets<LoadedFolder>,
    textures: &mut Assets<Image>,
    toml_assets: &Assets<TomlAsset>,
    toml_registry: &mut TomlConfigRegistry,
) -> Result<(
    Handle<TextureAtlasLayout>,
    Handle<Image>,
    HashMap<String, usize>,
)> {
    // Check cache
    //
    // 检查缓存
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
        toml_assets,
        toml_registry,
        module_name,
    )?;

    let atlas_layout_handle = texture_atlases.add(texture_atlas_layout);

    // Cache results
    //
    // 缓存结果
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
    toml_assets: &Assets<TomlAsset>,
    toml_registry: &mut TomlConfigRegistry,
    module_name: &str,
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
        if let Some(path) = handle.path() {
            let path_str = path.to_string();
            if !path_str.ends_with(".png")
                && !path_str.ends_with(".jpg")
                && !path_str.ends_with(".jpeg")
            {
                if path_str.ends_with(".toml") {
                    let toml_id = handle.id().typed_unchecked::<TomlAsset>();
                    if let Some(toml_asset) = toml_assets.get(toml_id) {
                        info!(
                            "Registering TOML configuration for module '{}': {}",
                            module_name, path_str
                        );
                        toml_registry.register_module_config(module_name, &toml_asset.config);
                    } else {
                        warn!("Unable to load TOML file: {}", path_str);
                    }
                }

                continue;
            }
            index_map.insert(path_str, added_count);
            added_count += 1;
        }

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

#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn create_sprite_from_atlas(
    commands: &mut Commands,
    translation: (f32, f32, f32),
    atlas_texture: Handle<Image>,
    atlas_sources: TextureAtlasSources,
    atlas_handle: Handle<TextureAtlasLayout>,
    vendor_handle: &Handle<Image>,
) {
    // This unwrap is on getting the handle from sources. If it fails, it panics.
    // However, this is more internal logic. If the atlas was built correctly, this should work.
    // For now, replacing with expect is a slight improvement, or better:
    //
    // 此 unwrap 是用于从源获取句柄。如果失败，则会恐慌。
    // 但是，这是更内部的逻辑。如果图集构建正确，这应该可以工作。
    // 暂时替换为 expect 是轻微的改进，或者更好：
    if let Some(texture_atlas_handle) = atlas_sources.handle(atlas_handle, vendor_handle) {
        commands.spawn((
            Transform {
                translation: Vec3::new(translation.0, translation.1, translation.2),
                ..default()
            },
            Sprite::from_atlas_image(atlas_texture, texture_atlas_handle),
        ));
    } else {
        warn!("Failed to get texture atlas handle for sprite spawning");
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub fn get_sprite_from_config(
    sprite_registry: &mut ModuleSpriteRegistry,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    loaded_folders: &Assets<LoadedFolder>,
    textures: &mut Assets<Image>,
    toml_assets: &Assets<TomlAsset>,
    toml_registry: &mut TomlConfigRegistry,
    module_name: &str,
    config_item_name: &str,
) -> Result<Sprite> {
    let (atlas_layout_handle, texture, index_map) = get_or_create_texture_atlas(
        module_name,
        sprite_registry,
        texture_atlases,
        loaded_folders,
        textures,
        toml_assets,
        toml_registry,
    )?;

    // Get image path and flip settings according to configuration
    //
    // 根据配置获取图像路径和翻转设置
    let (sprite_path, flip_x, flip_y) =
        if let Some(sprite_config) = toml_registry.get_sprite(config_item_name) {
            (
                sprite_config.path.clone(),
                sprite_config.flip_x,
                sprite_config.flip_y,
            )
        } else {
            return Err(anyhow!(
                "Sprite not found in configuration '{}'",
                config_item_name
            ));
        };

    let sprite_index = *index_map.get(&sprite_path).ok_or_else(|| {
        anyhow!(
            "The path '{}' of sprite '{}' was not found in the gallery",
            config_item_name,
            sprite_path
        )
    })?;

    let mut sprite = Sprite::from_atlas_image(
        texture,
        TextureAtlas {
            layout: atlas_layout_handle,
            index: sprite_index,
        },
    );

    // Apply flip settings
    //
    // 应用翻转设置
    sprite.flip_x = flip_x;
    sprite.flip_y = flip_y;

    Ok(sprite)
}
