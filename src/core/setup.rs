use crate::AppState;
use bevy::app::{App, Plugin, Update};
use bevy::asset::LoadedFolder;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct OverWorldCharacterSpriteFolder(pub(crate) Handle<LoadedFolder>);

pub(crate) struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Setup), load_textures_system)
            .add_systems(
                Update,
                check_textures_system.run_if(in_state(AppState::Setup)),
            );
    }
}

fn load_textures_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(OverWorldCharacterSpriteFolder(
        asset_server.load_folder("textures/overworld/characters"),
    ));
}

fn check_textures_system(
    mut next_state: ResMut<NextState<AppState>>,
    rpg_sprite_folder: Res<OverWorldCharacterSpriteFolder>,
    mut events: MessageReader<AssetEvent<LoadedFolder>>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(&rpg_sprite_folder.0) {
            next_state.set(AppState::Overworld);
        }
    }
}
