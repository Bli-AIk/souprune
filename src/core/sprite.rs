use crate::extra::toml::TomlAsset;
use crate::extra::toml::config::TomlConfigRegistry;
use bevy::asset::LoadedFolder;
use bevy::image::ImageSampler;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

pub mod load_context;
pub mod params;

// 类型别名来简化复杂类型
type AtlasCacheData = (
    Handle<TextureAtlasLayout>,
    Handle<Image>,
    HashMap<String, usize>,
);
type ModuleAtlasCache = HashMap<String, AtlasCacheData>;

#[derive(Resource, Default)]
pub(crate) struct ModuleSpriteRegistry {
    pub(crate) modules: HashMap<String, Handle<LoadedFolder>>,
    // 缓存每个模块的TextureAtlas相关数据，避免重复创建
    pub(crate) atlas_cache: ModuleAtlasCache,
}

impl ModuleSpriteRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            atlas_cache: HashMap::new(),
        }
    }

    pub fn register_module(&mut self, module_name: String, folder_handle: Handle<LoadedFolder>) {
        self.modules.insert(module_name, folder_handle);
    }

    pub fn get_module(&self, module_name: &str) -> Option<&Handle<LoadedFolder>> {
        self.modules.get(module_name)
    }

    pub fn get_cached_atlas(&self, module_name: &str) -> Option<&AtlasCacheData> {
        self.atlas_cache.get(module_name)
    }

    pub fn cache_atlas(
        &mut self,
        module_name: String,
        atlas_layout: Handle<TextureAtlasLayout>,
        texture: Handle<Image>,
        index_map: HashMap<String, usize>,
    ) {
        self.atlas_cache
            .insert(module_name, (atlas_layout, texture, index_map));
    }
}

pub fn get_or_create_texture_atlas(
    module_name: &str,
    sprite_registry: &mut ModuleSpriteRegistry,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    loaded_folders: &Assets<LoadedFolder>,
    textures: &mut Assets<Image>,
    toml_assets: &Assets<TomlAsset>,
    toml_registry: &mut TomlConfigRegistry,
) -> (
    Handle<TextureAtlasLayout>,
    Handle<Image>,
    HashMap<String, usize>,
) {
    // Check cache
    if let Some((atlas_layout, texture, index_map)) = sprite_registry.get_cached_atlas(module_name)
    {
        return (atlas_layout.clone(), texture.clone(), index_map.clone());
    }

    let handle = sprite_registry
        .get_module(module_name)
        .unwrap_or_else(|| panic!("{module_name} module not registered"));

    let loaded_folder = loaded_folders.get(handle).unwrap();

    let (texture_atlas_layout, _texture_atlas_sources, texture, index_map) = create_texture_atlas(
        loaded_folder,
        None,
        Some(ImageSampler::nearest()),
        textures,
        toml_assets,
        toml_registry,
        module_name,
    );

    let atlas_layout_handle = texture_atlases.add(texture_atlas_layout);

    // Cache results
    sprite_registry.cache_atlas(
        module_name.to_string(),
        atlas_layout_handle.clone(),
        texture.clone(),
        index_map.clone(),
    );

    (atlas_layout_handle, texture, index_map)
}

pub fn create_texture_atlas(
    folder: &LoadedFolder,
    padding: Option<UVec2>,
    sampling: Option<ImageSampler>,
    textures: &mut Assets<Image>,
    toml_assets: &Assets<TomlAsset>,
    toml_registry: &mut TomlConfigRegistry,
    module_name: &str,
) -> (
    TextureAtlasLayout,
    TextureAtlasSources,
    Handle<Image>,
    HashMap<String, usize>,
) {
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
                handle.path().unwrap()
            );
            continue;
        };

        texture_atlas_builder.add_texture(Some(id), texture);
    }

    let (texture_atlas_layout, texture_atlas_sources, texture) =
        texture_atlas_builder.build().unwrap();
    let texture = textures.add(texture);

    let image = textures.get_mut(&texture).unwrap();
    image.sampler = sampling.unwrap_or_default();

    (
        texture_atlas_layout,
        texture_atlas_sources,
        texture,
        index_map,
    )
}

#[allow(dead_code)]
pub fn create_sprite_from_atlas(
    commands: &mut Commands,
    translation: (f32, f32, f32),
    atlas_texture: Handle<Image>,
    atlas_sources: TextureAtlasSources,
    atlas_handle: Handle<TextureAtlasLayout>,
    vendor_handle: &Handle<Image>,
) {
    commands.spawn((
        Transform {
            translation: Vec3::new(translation.0, translation.1, translation.2),
            ..default()
        },
        Sprite::from_atlas_image(
            atlas_texture,
            atlas_sources.handle(atlas_handle, vendor_handle).unwrap(),
        ),
    ));
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
) -> Sprite {
    let (atlas_layout_handle, texture, index_map) = get_or_create_texture_atlas(
        module_name,
        sprite_registry,
        texture_atlases,
        loaded_folders,
        textures,
        toml_assets,
        toml_registry,
    );

    // Get image path and flip settings according to configuration
    let (sprite_path, flip_x, flip_y) =
        if let Some(sprite_config) = toml_registry.get_sprite(config_item_name) {
            (
                sprite_config.path.clone(),
                sprite_config.flip_x,
                sprite_config.flip_y,
            )
        } else {
            panic!("Sprite not found in configuration '{}'", config_item_name);
        };

    let sprite_index = *index_map.get(&sprite_path).unwrap_or_else(|| {
        panic!(
            "The path '{}' of sprite '{}' was not found in the gallery",
            config_item_name, sprite_path
        )
    });

    let mut sprite = Sprite::from_atlas_image(
        texture,
        TextureAtlas {
            layout: atlas_layout_handle,
            index: sprite_index,
        },
    );

    // Apply flip settings
    sprite.flip_x = flip_x;
    sprite.flip_y = flip_y;

    sprite
}
