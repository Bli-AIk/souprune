use crate::core::sprite::load_context::SpriteLoadContext;
use crate::core::sprite::resources::ModuleSpriteRegistry;
use crate::extra::toml::TomlAsset;
use crate::extra::toml::config::TomlConfigRegistry;
use bevy::asset::{Assets, LoadedFolder};
use bevy::ecs::system::SystemParam;
use bevy::image::{Image, TextureAtlasLayout};
use bevy::prelude::{Res, ResMut};

#[derive(SystemParam)]
pub struct SpriteParams<'w> {
    sprite_registry: ResMut<'w, ModuleSpriteRegistry>,
    texture_atlases: ResMut<'w, Assets<TextureAtlasLayout>>,
    loaded_folders: Res<'w, Assets<LoadedFolder>>,
    textures: ResMut<'w, Assets<Image>>,
    toml_assets: Res<'w, Assets<TomlAsset>>,
    toml_registry: ResMut<'w, TomlConfigRegistry>,
}

impl<'w> SpriteParams<'w> {
    pub(crate) fn create_sprite_context(&mut self) -> SpriteLoadContext<'_> {
        SpriteLoadContext::new(
            &mut self.sprite_registry,
            &mut self.texture_atlases,
            &self.loaded_folders,
            &mut self.textures,
            &self.toml_assets,
            &mut self.toml_registry,
        )
    }
}
