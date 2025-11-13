use crate::core::sprite::ModuleSpriteRegistry;
use crate::extra::toml::TomlAsset;
use crate::extra::toml::config::TomlConfigRegistry;
use bevy::asset::{Assets, LoadedFolder};
use bevy::image::{Image, TextureAtlas, TextureAtlasLayout};
use bevy::prelude::Sprite;

pub struct SpriteLoadContext<'a> {
    sprite_registry: &'a mut ModuleSpriteRegistry,
    texture_atlases: &'a mut Assets<TextureAtlasLayout>,
    loaded_folders: &'a Assets<LoadedFolder>,
    textures: &'a mut Assets<Image>,
    toml_assets: &'a Assets<TomlAsset>,
    toml_registry: &'a mut TomlConfigRegistry,
}

impl<'a> SpriteLoadContext<'a> {
    pub(crate) fn new(
        sprite_registry: &'a mut ModuleSpriteRegistry,
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
        let (atlas_layout_handle, texture, index_map) =
            crate::core::sprite::utils::get_or_create_texture_atlas(
                module_name,
                self.sprite_registry,
                self.texture_atlases,
                self.loaded_folders,
                self.textures,
                self.toml_assets,
                self.toml_registry,
            );

        let (sprite_path, flip_x, flip_y) =
            if let Some(sprite_config) = self.toml_registry.get_sprite(config_item_name) {
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

    pub(crate) fn get_sprite_animations(
        &mut self,
        module_name: &str,
        config_item_name: &str,
    ) -> Vec<Sprite> {
        let (atlas_layout_handle, texture, index_map) =
            crate::core::sprite::utils::get_or_create_texture_atlas(
                module_name,
                self.sprite_registry,
                self.texture_atlases,
                self.loaded_folders,
                self.textures,
                self.toml_assets,
                self.toml_registry,
            );

        let (config_path, flip_x, flip_y) =
            if let Some(sprite_config) = self.toml_registry.get_animation(config_item_name) {
                (
                    sprite_config.path.clone(),
                    sprite_config.flip_x,
                    sprite_config.flip_y,
                )
            } else {
                panic!(
                    "Animation not found in configuration '{}'",
                    config_item_name
                );
            };

        // Check if path points to a single file
        if config_path.ends_with(".png")
            || config_path.ends_with(".jpg")
            || config_path.ends_with(".jpeg")
        {
            // Single file: Find matching files directly
            if let Some(&sprite_index) = index_map.get(&config_path) {
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

                vec![sprite]
            } else {
                panic!(
                    "Single file '{}' not found for animation '{}'",
                    config_path, config_item_name
                );
            }
        } else {
            // Directory: collect all matching files and sort them
            let mut matching_files: Vec<_> = index_map
                .iter()
                .filter(|(path, _)| path.starts_with(&config_path))
                .collect();

            if matching_files.is_empty() {
                panic!(
                    "No files found in directory '{}' for animation '{}'",
                    config_path, config_item_name
                );
            }

            // Sort by filename to ensure correct frame order
            matching_files.sort_by(|(path_a, _), (path_b, _)| path_a.cmp(path_b));

            matching_files
                .into_iter()
                .map(|(_, &sprite_index)| {
                    let mut sprite = Sprite::from_atlas_image(
                        texture.clone(),
                        TextureAtlas {
                            layout: atlas_layout_handle.clone(),
                            index: sprite_index,
                        },
                    );

                    // Apply flip settings to each frame
                    sprite.flip_x = flip_x;
                    sprite.flip_y = flip_y;

                    sprite
                })
                .collect()
        }
    }

    pub(crate) fn get_sprite_animations_with_config(
        &mut self,
        module_name: &str,
        config_item_name: &str,
    ) -> (Vec<Sprite>, bool) {
        let sprites = self.get_sprite_animations(module_name, config_item_name);
        let looping =
            if let Some(animation_config) = self.toml_registry.get_animation(config_item_name) {
                animation_config.looping
            } else {
                true
            };
        (sprites, looping)
    }

    pub(crate) fn get_animation_frame_duration(&self, config_item_name: &str) -> f32 {
        if let Some(animation_config) = self.toml_registry.get_animation(config_item_name) {
            animation_config.frame_duration
        } else {
            0.15
        }
    }
}
