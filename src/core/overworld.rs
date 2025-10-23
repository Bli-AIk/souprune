use crate::AppState;
use crate::core::core_components::Direction;
use crate::core::input::{Action, PlayerInputSettings};
use crate::core::overworld::character::character_components::*;
use crate::core::resource::*;
use bevy::app::{App, Plugin};
use bevy::asset::LoadedFolder;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use character::CharacterBundle;
use character::character_systems::*;
use leafwing_input_manager::action_state::*;
use leafwing_input_manager::prelude::*;
use seldom_state::machine::*;
use seldom_state::trigger::IntoTrigger;
mod character;

pub(crate) struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Overworld), setup_overworld_system)
            .add_systems(
                Update,
                (
                    update_idle_system,
                    update_walking_system,
                    update_running_system,
                ),
            );
    }
}

fn setup_overworld_system(
    mut commands: Commands,
    rpg_sprite_handles: Res<OverWorldCharacterSpriteFolder>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut textures: ResMut<Assets<Image>>,
    player_input: Res<PlayerInputSettings>,
) {
    let loaded_folder = loaded_folders.get(&rpg_sprite_handles.0).unwrap();

    let (texture_atlas_nearest, nearest_sources, nearest_texture) = create_texture_atlas(
        loaded_folder,
        None,
        Some(ImageSampler::nearest()),
        &mut textures,
    );
    let atlas_nearest_handle = texture_atlases.add(texture_atlas_nearest);

    let frisk_handle: Handle<Image> = asset_server
        .get_handle("textures/overworld/characters/frisk/walk/down/1.png")
        .unwrap();

    let sprite = Sprite::from_atlas_image(
        nearest_texture,
        nearest_sources
            .handle(atlas_nearest_handle, &frisk_handle)
            .unwrap(),
    );

    commands.spawn((
        Idle,
        PlayerControlled,
        StateMachine::default()
            .trans::<Idle, _>(is_walking, Walking)
            .trans::<Walking, _>(is_walking.not(), Idle)
            .trans::<Running, _>(is_walking.not(), Idle)
            .trans::<Walking, _>(is_running, Running)
            .trans::<Running, _>(is_running.not(), Walking)
            .set_trans_logging(true),
        player_input.get_merged_map(),
        ActionState::<Action>::default(),
        CharacterBundle::new(Vec2::new(0.0, 0.0), Direction::Down, sprite.clone()),
    ));
}

fn is_walking(query: Query<&ActionState<Action>, With<PlayerControlled>>) -> Result<(), ()> {
    let action_state = query.single().map_err(|_| ())?;
    if action_state.pressed(&Action::Left)
        || action_state.pressed(&Action::Right)
        || action_state.pressed(&Action::Up)
        || action_state.pressed(&Action::Down)
    {
        Ok(())
    } else {
        Err(())
    }
}

fn is_running(query: Query<&ActionState<Action>, With<PlayerControlled>>) -> Result<(), ()> {
    let action_state = query.single().map_err(|_| ())?;
    if action_state.pressed(&Action::Cancel) {
        Ok(())
    } else {
        Err(())
    }
}
