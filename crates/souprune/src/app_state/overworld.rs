use crate::app_state::AppState;
use crate::app_state::overworld::character::components::*;
use crate::app_state::overworld::player::PlayerPlugin;
use crate::app_state::overworld::tilemap::TilemapPlugin;
use crate::core::animation::components::SpriteAnimationClip;
use crate::core::basic_components::Direction;
use crate::core::camera::Followable;
use crate::core::input::{Action, PlayerInputSettings};
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
        app.add_plugins((TilemapPlugin, PlayerPlugin))
            .add_systems(
                OnEnter(AppState::Overworld),
                (create_overworld_entities_system, bind_camera_target_system).chain(),
            )
            .add_systems(Update, (update_walking_system, update_running_system));
    }
}

fn create_overworld_entities_system(
    mut commands: Commands,
    mut sprite_params: SpriteParams,
    player_input: Res<PlayerInputSettings>,
) {
    spawn_overworld_player(&mut commands, &mut sprite_params, &player_input);
}

fn spawn_overworld_player(
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
fn bind_camera_target_system(
    mut camera: Query<&mut Followable, With<Camera2d>>,
    player: Query<Entity, With<PlayerControlled>>,
) {
    if let Ok(player_entity) = player.single() {
        for mut followable in camera.iter_mut() {
            followable.target = Some(player_entity);
        }
    }
}
