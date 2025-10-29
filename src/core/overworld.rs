use crate::AppState;
use crate::core::components::Direction;
use crate::core::input::{Action, PlayerInputSettings};
use crate::core::overworld::character::components::*;
use crate::core::sprite::*;
use crate::extra::toml_asset_loader::TomlAsset;
use bevy::app::{App, Plugin};
use bevy::asset::LoadedFolder;
use bevy::prelude::*;
use character::CharacterBundle;
use character::systems::*;
use leafwing_input_manager::action_state::*;
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

#[allow(clippy::too_many_arguments)]
fn setup_overworld_system(
    mut commands: Commands,
    sprite_registry: Res<ModuleSpriteRegistry>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut textures: ResMut<Assets<Image>>,
    toml_assets: Res<Assets<TomlAsset>>,
    mut toml_registry: ResMut<TomlConfigRegistry>,
    player_input: Res<PlayerInputSettings>,
) {
    let sprite = get_sprite_from_config(
        &sprite_registry,
        &mut texture_atlases,
        &loaded_folders,
        &mut textures,
        &toml_assets,
        &mut toml_registry,
        "overworld",
        "chest_box",
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
