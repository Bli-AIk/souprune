use crate::core::resource_plugin::{
    create_sprite_from_atlas, create_texture_atlas, OverWorldCharacterSpriteFolder,
};
use crate::AppState;
use bevy::app::{App, Plugin};
use bevy::asset::LoadedFolder;
use bevy::image::ImageSampler;
use bevy::prelude::*;

pub(crate) struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Overworld), setup_overworld_system);
    }
}

fn setup_overworld_system(
    mut commands: Commands,
    rpg_sprite_handles: Res<OverWorldCharacterSpriteFolder>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut textures: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2d);

    let loaded_folder = loaded_folders.get(&rpg_sprite_handles.0).unwrap();

    let (texture_atlas_nearest, nearest_sources, nearest_texture) = create_texture_atlas(
        loaded_folder,
        None,
        Some(ImageSampler::nearest()),
        &mut textures,
    );
    let atlas_nearest_handle = texture_atlases.add(texture_atlas_nearest);

    let frisk_handle: Handle<Image> = asset_server
        .get_handle("textures/overworld/characters/frisk/walk/frisk-walk-down-1.png")
        .unwrap();

    create_sprite_from_atlas(
        &mut commands,
        (0.0, 0.0, 0.0),
        nearest_texture,
        nearest_sources,
        atlas_nearest_handle,
        &frisk_handle,
    );
}
