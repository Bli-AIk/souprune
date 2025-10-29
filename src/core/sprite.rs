use crate::extra::toml_asset_loader::{AnimationConfig, SpriteConfig, TomlAsset};
use bevy::asset::LoadedFolder;
use bevy::image::ImageSampler;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct ModuleSpriteRegistry {
    pub(crate) modules: HashMap<String, Handle<LoadedFolder>>,
}

impl ModuleSpriteRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn register_module(&mut self, module_name: String, folder_handle: Handle<LoadedFolder>) {
        self.modules.insert(module_name, folder_handle);
    }

    pub fn get_module(&self, module_name: &str) -> Option<&Handle<LoadedFolder>> {
        self.modules.get(module_name)
    }
}

/// 存储所有模块的 TOML 配置数据的资源
#[derive(Resource, Default)]
pub struct TomlConfigRegistry {
    pub animations: HashMap<String, AnimationConfig>,
    pub sprites: HashMap<String, SpriteConfig>,
    pub modules: HashMap<String, Vec<String>>, // module_name -> loaded_before
}

impl TomlConfigRegistry {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
            sprites: HashMap::new(),
            modules: HashMap::new(),
        }
    }

    /// 注册一个模块的 TOML 配置
    pub fn register_module_config(
        &mut self,
        module_name: &str,
        config: &crate::extra::toml_asset_loader::TomlConfig,
    ) {
        // 添加动画配置
        for anim in &config.animations {
            self.animations.insert(anim.name.clone(), anim.clone());
        }

        // 添加精灵配置
        for sprite in &config.sprites {
            self.sprites.insert(sprite.name.clone(), sprite.clone());
        }

        // 添加模块配置
        for module in &config.module {
            self.modules
                .insert(module_name.to_string(), module.loaded_before.clone());
        }

        info!(
            "已注册模块 '{}' 的配置: {} 个动画, {} 个精灵",
            module_name,
            config.animations.len(),
            config.sprites.len()
        );
    }

    /// 根据名称获取动画配置
    pub fn get_animation(&self, name: &str) -> Option<&AnimationConfig> {
        self.animations.get(name)
    }

    /// 根据名称获取精灵配置
    pub fn get_sprite(&self, name: &str) -> Option<&SpriteConfig> {
        self.sprites.get(name)
    }

    /// 获取模块的依赖列表
    pub fn get_module_dependencies(&self, module_name: &str) -> Option<&Vec<String>> {
        self.modules.get(module_name)
    }

    /// 列出所有可用的动画名称
    pub fn list_animations(&self) -> Vec<&String> {
        self.animations.keys().collect()
    }

    /// 列出所有可用的精灵名称
    pub fn list_sprites(&self) -> Vec<&String> {
        self.sprites.keys().collect()
    }
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
                    // 处理 TOML 配置文件，将数据存储到资源中
                    let toml_id = handle.id().typed_unchecked::<TomlAsset>();
                    if let Some(toml_asset) = toml_assets.get(toml_id) {
                        info!("正在注册模块 '{}' 的 TOML 配置: {}", module_name, path_str);
                        toml_registry.register_module_config(module_name, &toml_asset.config);
                    } else {
                        warn!("无法加载 TOML 文件: {}", path_str);
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
pub fn get_sprite_from_config(
    sprite_registry: &ModuleSpriteRegistry,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    loaded_folders: &Assets<LoadedFolder>,
    textures: &mut Assets<Image>,
    toml_assets: &Assets<TomlAsset>,
    toml_registry: &mut TomlConfigRegistry,
    module_name: &str,
    config_item_name: &str,
) -> Sprite {
    let handle = sprite_registry
        .get_module(module_name)
        .unwrap_or_else(|| panic!("{module_name} module not registered"));

    let loaded_folder = loaded_folders.get(handle).unwrap();

    let (texture_atlas_nearest, _nearest_sources, nearest_texture, index_map) =
        create_texture_atlas(
            loaded_folder,
            None,
            Some(ImageSampler::nearest()),
            textures,
            toml_assets,
            toml_registry,
            module_name,
        );

    let atlas_nearest_handle = texture_atlases.add(texture_atlas_nearest);

    // Get image path according to configuration
    let sprite_path = if let Some(sprite_config) = toml_registry.get_sprite(config_item_name) {
        sprite_config.path.clone()
    } else {
        panic!("Elf not found in configuration '{}'", config_item_name);
    };

    let sprite_index = *index_map.get(&sprite_path).unwrap_or_else(|| {
        panic!(
            "The path '{}' of sprite '{}' was not found in the gallery",
            config_item_name, sprite_path
        )
    });

    Sprite::from_atlas_image(
        nearest_texture,
        TextureAtlas {
            layout: atlas_nearest_handle.clone(),
            index: sprite_index,
        },
    )
}
