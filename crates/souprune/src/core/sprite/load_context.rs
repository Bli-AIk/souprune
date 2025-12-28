//! # load_context.rs
//!
//! # load_context.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines `SpriteLoadContext`, a helper struct that encapsulates access to various assets and registries needed for loading sprites and animations dynamically.
//!
//! 定义 `SpriteLoadContext`，这是一个辅助结构体，封装了对动态加载精灵和动画所需的各种资产和注册表的访问。

use crate::core::sprite::resources::ModuleSpriteRegistry;
use crate::extra::toml::TomlAsset;
use crate::extra::toml::config::TomlConfigRegistry;
use anyhow::{Result, anyhow};
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

    pub(crate) fn get_sprite(
        &mut self,
        module_name: &str,
        config_item_name: &str,
    ) -> Result<Sprite> {
        let (atlas_layout_handle, texture, index_map) =
            crate::core::sprite::utils::get_or_create_texture_atlas(
                module_name,
                self.sprite_registry,
                self.texture_atlases,
                self.loaded_folders,
                self.textures,
                self.toml_assets,
                self.toml_registry,
            )?;

        let (sprite_path, flip_x, flip_y) =
            if let Some(sprite_config) = self.toml_registry.get_sprite(config_item_name) {
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

    pub(crate) fn get_sprite_animations(
        &mut self,
        module_name: &str,
        config_item_name: &str,
    ) -> Result<Vec<Sprite>> {
        let (atlas_layout_handle, texture, index_map) =
            crate::core::sprite::utils::get_or_create_texture_atlas(
                module_name,
                self.sprite_registry,
                self.texture_atlases,
                self.loaded_folders,
                self.textures,
                self.toml_assets,
                self.toml_registry,
            )?;

        let (config_path, flip_x, flip_y) =
            if let Some(sprite_config) = self.toml_registry.get_animation(config_item_name) {
                (
                    sprite_config.path.clone(),
                    sprite_config.flip_x,
                    sprite_config.flip_y,
                )
            } else {
                return Err(anyhow!(
                    "Animation not found in configuration '{}'",
                    config_item_name
                ));
            };

        // Check if path points to a single file
        //
        // 检查路径是否指向单个文件
        if config_path.ends_with(".png")
            || config_path.ends_with(".jpg")
            || config_path.ends_with(".jpeg")
        {
            // Single file: Find matching files directly
            //
            // 单个文件：直接查找匹配文件
            if let Some(&sprite_index) = index_map.get(&config_path) {
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

                Ok(vec![sprite])
            } else {
                Err(anyhow!(
                    "Single file '{}' not found for animation '{}'",
                    config_path,
                    config_item_name
                ))
            }
        } else {
            // Directory: collect all matching files and sort them
            //
            // 目录：收集所有匹配文件并进行排序
            let mut matching_files: Vec<_> = index_map
                .iter()
                .filter(|(path, _)| path.starts_with(&config_path))
                .collect();

            if matching_files.is_empty() {
                return Err(anyhow!(
                    "No files found in directory '{}' for animation '{}'",
                    config_path,
                    config_item_name
                ));
            }

            // Sort by filename using natural (numeric) ordering to ensure correct frame order
            // e.g., frame_2.png comes before frame_10.png
            //
            // 使用自然（数字）排序以确保正确的帧顺序
            // 例如：frame_2.png 排在 frame_10.png 之前
            matching_files.sort_by(|(path_a, _), (path_b, _)| {
                natural_sort_key(path_a).cmp(&natural_sort_key(path_b))
            });

            let sprites = matching_files
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
                    //
                    // 对每一帧应用翻转设置
                    sprite.flip_x = flip_x;
                    sprite.flip_y = flip_y;

                    sprite
                })
                .collect();

            Ok(sprites)
        }
    }

    pub(crate) fn get_sprite_animations_with_config(
        &mut self,
        module_name: &str,
        config_item_name: &str,
    ) -> Result<(Vec<Sprite>, bool)> {
        let sprites = self.get_sprite_animations(module_name, config_item_name)?;
        let looping =
            if let Some(animation_config) = self.toml_registry.get_animation(config_item_name) {
                animation_config.looping
            } else {
                true
            };
        Ok((sprites, looping))
    }

    pub(crate) fn get_animation_frame_duration(&self, config_item_name: &str) -> f32 {
        if let Some(animation_config) = self.toml_registry.get_animation(config_item_name) {
            animation_config.frame_duration
        } else {
            0.15
        }
    }

    pub(crate) fn get_missing_sprite(&mut self) -> Sprite {
        let texture = crate::core::sprite::utils::get_or_create_missing_texture(
            self.sprite_registry,
            self.textures,
        );
        Sprite {
            image: texture,
            ..Default::default()
        }
    }
}

/// Natural sort key: splits a string into segments of digits and non-digits,
/// allowing numeric comparison of digit sequences.
///
/// 自然排序键：将字符串分割成数字和非数字段，
/// 允许对数字序列进行数值比较。
fn natural_sort_key(s: &str) -> Vec<NaturalSortSegment> {
    let mut result = Vec::new();
    let mut current_str = String::new();
    let mut in_number = false;

    for c in s.chars() {
        let is_digit = c.is_ascii_digit();
        if is_digit != in_number && !current_str.is_empty() {
            if in_number {
                result.push(NaturalSortSegment::Number(current_str.parse().unwrap_or(0)));
            } else {
                result.push(NaturalSortSegment::Text(current_str.clone()));
            }
            current_str.clear();
        }
        current_str.push(c);
        in_number = is_digit;
    }

    if !current_str.is_empty() {
        if in_number {
            result.push(NaturalSortSegment::Number(current_str.parse().unwrap_or(0)));
        } else {
            result.push(NaturalSortSegment::Text(current_str));
        }
    }

    result
}

/// Segment type for natural sorting
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum NaturalSortSegment {
    Text(String),
    Number(u64),
}
