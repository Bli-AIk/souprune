use crate::AppState;
use crate::core::animation::SpriteAnimationClip;
use crate::core::basic_components::Direction;
use crate::core::input::{Action, PlayerInputSettings};
use crate::core::overworld::character::components::*;
use crate::core::sprite::params::SpriteParams;
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use character::systems::*;
use leafwing_input_manager::action_state::*;
use seldom_state::machine::*;
use seldom_state::trigger::IntoTrigger;
use crate::core::overworld::player::PlayerPlugin;

mod character;
mod player;

pub(crate) struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Overworld), setup_overworld_system)
            .add_systems(
                Update,
                (
                    update_walking_system,
                    update_running_system,
                ),
            )
        .add_plugins(PlayerPlugin);
    }
}

fn setup_overworld_system(
    mut commands: Commands,
    mut sprite_params: SpriteParams,
    player_input: Res<PlayerInputSettings>,
) {
    let sprite = sprite_params
        .create_sprite_context()
        .get_sprite("overworld", "chest_box");
    commands.spawn((
        sprite,
        Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
    ));

    use player::*;
    commands.spawn((
        Idle,
        PlayerControlled,
        StateMachine::default()
            .trans::<Idle, _>(is_player_walking, Walking)
            .trans::<Walking, _>(is_player_walking.not(), Idle)
            .trans::<Running, _>(is_player_walking.not(), Idle)
            .trans::<Walking, _>(is_player_running, Running)
            .trans::<Running, _>(is_player_running.not(), Walking),
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
