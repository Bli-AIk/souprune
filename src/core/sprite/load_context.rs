use crate::core::sprite::{ModuleSpriteRegistry, create_texture_atlas};
use crate::extra::toml::TomlAsset;
use crate::extra::toml::config::TomlConfigRegistry;
use bevy::asset::{Assets, LoadedFolder};
use bevy::image::{Image, ImageSampler, TextureAtlas, TextureAtlasLayout};
use bevy::prelude::Sprite;

pub struct SpriteLoadContext<'a> {
    sprite_registry: &'a ModuleSpriteRegistry,
    texture_atlases: &'a mut Assets<TextureAtlasLayout>,
    loaded_folders: &'a Assets<LoadedFolder>,
    textures: &'a mut Assets<Image>,
    toml_assets: &'a Assets<TomlAsset>,
    toml_registry: &'a mut TomlConfigRegistry,
}

impl<'a> SpriteLoadContext<'a> {
    pub(crate) fn new(
        sprite_registry: &'a ModuleSpriteRegistry,
        texture_atlases: &'a mut Assets<TextureAtlasLayout>,
        loaded_folders: &'a Assets<LoadedFolder>,
        textures: &'a mut Assets<Image>,
        toml_assets: &'a Assets<TomlAsset>,
        toml_registry: &'a mut TomlConfigRegistry,
    ) -> Self {
        Self {
            sprite_registry,
            texture_atlases,
            loaded_folders,
            textures,
            toml_assets,
            toml_registry,
        }
    }

    pub(crate) fn get_sprite(&mut self, module_name: &str, config_item_name: &str) -> Sprite {
        let handle = self
            .sprite_registry
            .get_module(module_name)
            .unwrap_or_else(|| panic!("{module_name} module not registered"));

        let loaded_folder = self.loaded_folders.get(handle).unwrap();

        let (texture_atlas_nearest, _nearest_sources, nearest_texture, index_map) =
            create_texture_atlas(
                loaded_folder,
                None,
                Some(ImageSampler::nearest()),
                self.textures,
                self.toml_assets,
                self.toml_registry,
                module_name,
            );

        let atlas_nearest_handle = self.texture_atlases.add(texture_atlas_nearest);

        let sprite_path =
            if let Some(sprite_config) = self.toml_registry.get_sprite(config_item_name) {
                sprite_config.path.clone()
            } else {
                panic!("Sprite not found in configuration '{}'", config_item_name);
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

    pub(crate) fn get_sprite_animations(
        &mut self,
        module_name: &str,
        config_item_name: &str,
    ) -> Vec<Sprite> {
        let handle = self
            .sprite_registry
            .get_module(module_name)
            .unwrap_or_else(|| panic!("{module_name} module not registered"));

        let loaded_folder = self.loaded_folders.get(handle).unwrap();

        let (texture_atlas_nearest, _nearest_sources, nearest_texture, index_map) =
            create_texture_atlas(
                loaded_folder,
                None,
                Some(ImageSampler::nearest()),
                self.textures,
                self.toml_assets,
                self.toml_registry,
                module_name,
            );

        let atlas_nearest_handle = self.texture_atlases.add(texture_atlas_nearest);

        let folder_path =
            if let Some(sprite_config) = self.toml_registry.get_animation(config_item_name) {
                sprite_config.path.clone()
            } else {
                panic!(
                    "Sprite folder not found in configuration '{}'",
                    config_item_name
                );
            };

        index_map
            .iter()
            .filter(|(path, _)| path.starts_with(&folder_path))
            .map(|(_, &sprite_index)| {
                Sprite::from_atlas_image(
                    nearest_texture.clone(),
                    TextureAtlas {
                        layout: atlas_nearest_handle.clone(),
                        index: sprite_index,
                    },
                )
            })
            .collect()
    }
}
