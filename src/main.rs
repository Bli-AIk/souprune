use bevy::asset::LoadedFolder;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_state::<AppState>()
        .add_systems(OnEnter(AppState::Setup), load_textures)
        .run();
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
    Setup,
    Finished,
}

#[derive(Resource, Default)]
struct OverWorldCharacterSpriteFolder(Handle<LoadedFolder>);

fn load_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(OverWorldCharacterSpriteFolder(asset_server.load_folder("textures/overworld/characters")));
}