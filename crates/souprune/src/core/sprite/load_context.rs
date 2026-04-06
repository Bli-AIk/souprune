//! # load_context.rs
//!
//! Sprite/animation lookup using texture atlas + direct paths.
//!
//! 通过纹理图集 + 直接路径查找精灵和动画。

use crate::core::sprite::resources::ModuleSpriteRegistry;
use anyhow::{Result, anyhow};
use bevy::asset::{Assets, LoadedFolder};
use bevy::image::{Image, TextureAtlas, TextureAtlasLayout};
use bevy::prelude::Sprite;

pub struct SpriteLoadContext<'a> {
    sprite_registry: &'a mut ModuleSpriteRegistry,
    texture_atlases: &'a mut Assets<TextureAtlasLayout>,
    loaded_folders: &'a Assets<LoadedFolder>,
    textures: &'a mut Assets<Image>,
}

impl<'a> SpriteLoadContext<'a> {
    pub(crate) fn new(
        sprite_registry: &'a mut ModuleSpriteRegistry,
        texture_atlases: &'a mut Assets<TextureAtlasLayout>,
        loaded_folders: &'a Assets<LoadedFolder>,
        textures: &'a mut Assets<Image>,
    ) -> Self {
        Self {
            sprite_registry,
            texture_atlases,
            loaded_folders,
            textures,
        }
    }

    /// Load a single sprite by module-relative path.
    ///
    /// `relative_path` is relative to `assets/textures/{module}/`,
    /// e.g. `"characters/hero/walk/down/0.png"`.
    pub(crate) fn get_sprite(
        &mut self,
        module_name: &str,
        relative_path: &str,
        flip_x: bool,
        flip_y: bool,
    ) -> Result<Sprite> {
        let (atlas_layout_handle, texture, index_map) =
            crate::core::sprite::utils::get_or_create_texture_atlas(
                module_name,
                self.sprite_registry,
                self.texture_atlases,
                self.loaded_folders,
                self.textures,
            )?;

        let full_path = format!("assets/textures/{module_name}/{relative_path}");

        let sprite_index = *index_map.get(&full_path).ok_or_else(|| {
            anyhow!(
                "Sprite path '{}' not found in atlas for module '{}'",
                full_path,
                module_name
            )
        })?;

        let mut sprite = Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: atlas_layout_handle,
                index: sprite_index,
            },
        );

        sprite.flip_x = flip_x;
        sprite.flip_y = flip_y;

        Ok(sprite)
    }

    /// Load animation frames by module-relative directory path.
    ///
    /// `relative_path` is a directory prefix (e.g. `"characters/hero/walk/down"`)
    /// or a single file (e.g. `"characters/hero/walk/down/0.png"`).
    pub(crate) fn get_sprite_animations(
        &mut self,
        module_name: &str,
        relative_path: &str,
        flip_x: bool,
        flip_y: bool,
    ) -> Result<Vec<Sprite>> {
        let (atlas_layout_handle, texture, index_map) =
            crate::core::sprite::utils::get_or_create_texture_atlas(
                module_name,
                self.sprite_registry,
                self.texture_atlases,
                self.loaded_folders,
                self.textures,
            )?;

        let full_path = format!("assets/textures/{module_name}/{relative_path}");

        // Single file path
        if full_path.ends_with(".png")
            || full_path.ends_with(".jpg")
            || full_path.ends_with(".jpeg")
        {
            if let Some(&sprite_index) = index_map.get(&full_path) {
                let mut sprite = Sprite::from_atlas_image(
                    texture,
                    TextureAtlas {
                        layout: atlas_layout_handle,
                        index: sprite_index,
                    },
                );
                sprite.flip_x = flip_x;
                sprite.flip_y = flip_y;
                return Ok(vec![sprite]);
            }
            return Err(anyhow!(
                "Single file '{}' not found in atlas for module '{}'",
                full_path,
                module_name
            ));
        }

        // Directory: collect all matching files and sort
        let prefix = format!("{full_path}/");
        let mut matching_files: Vec<_> = index_map
            .iter()
            .filter(|(path, _)| path.starts_with(&prefix))
            .collect();

        if matching_files.is_empty() {
            return Err(anyhow!(
                "No files found under '{}' for module '{}'",
                full_path,
                module_name
            ));
        }

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
                sprite.flip_x = flip_x;
                sprite.flip_y = flip_y;
                sprite
            })
            .collect();

        Ok(sprites)
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

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum NaturalSortSegment {
    Text(String),
    Number(u64),
}
