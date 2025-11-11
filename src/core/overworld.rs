use crate::AppState;
use crate::core::animation::SpriteAnimationClip;
use crate::core::basic_components::Direction;
use crate::core::input::{Action, PlayerInputSettings};
use crate::core::overworld::character::components::*;
use crate::core::overworld::player::PlayerPlugin;
use crate::core::sprite::params::SpriteParams;
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use character::systems::*;
use leafwing_input_manager::action_state::*;
use seldom_state::machine::*;
use seldom_state::trigger::IntoTrigger;

mod character;
mod player;
mod tilemap;

pub(crate) struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Overworld), setup_overworld_system)
            .add_systems(
                Update,
                (
                    update_walking_system,
                    update_running_system,
                    tilemap::filter_prototype_layers_and_set_z_order,
                ),
            )
            .add_plugins(PlayerPlugin);
    }
}

fn setup_overworld_system(
    mut commands: Commands,
    mut sprite_params: SpriteParams,
    player_input: Res<PlayerInputSettings>,
    asset_server: Res<AssetServer>,
) {
    setup_overworld_player(&mut commands, &mut sprite_params, &player_input);
    tilemap::setup_tilemap(&mut commands, &asset_server);
}

fn setup_overworld_player(
    commands: &mut Commands,
    sprite_params: &mut SpriteParams,
    player_input: &Res<PlayerInputSettings>,
) {
    use player::*;
    commands.spawn((
        StateIdle,
        PlayerControlled,
        StateMachine::default()
            .trans::<StateIdle, _>(is_player_walking, StateWalking)
            .trans::<StateWalking, _>(is_player_walking.not(), StateIdle)
            .trans::<StateRunning, _>(is_player_walking.not(), StateIdle)
            .trans::<StateWalking, _>(is_player_running, StateRunning)
            .trans::<StateRunning, _>(is_player_running.not(), StateWalking),
        player_input.get_merged_map(),
        ActionState::<Action>::default(),
        PlayerBundle::new(
            Vec2::new(0.0, 0.0),
            Direction::Down,
            SpriteAnimationClip::new(
                &mut sprite_params.create_sprite_context(),
                "overworld",
                "frisk_walk_down",
            ),
        ),
    ));
}
