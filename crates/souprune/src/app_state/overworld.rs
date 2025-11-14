use crate::app_state::AppState;
use crate::core::animation::components::SpriteAnimationClip;
use crate::core::basic_components::Direction;
use crate::core::camera::Followable;
use crate::core::collision::Rect2DCollider;
use crate::core::input::{Action, PlayerInputSettings};
use crate::core::sprite::params::SpriteParams;
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use leafwing_input_manager::action_state::*;
use seldom_state::machine::*;
use seldom_state::trigger::*;

pub(crate) mod character;
mod player;
mod tilemap;

pub(crate) struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            tilemap::TilemapPlugin,
            player::PlayerPlugin,
            character::CharacterPlugin,
        ))
        .add_systems(
            OnEnter(AppState::Overworld),
            (create_overworld_entities_system, bind_camera_target_system).chain(),
        );
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
    use character::components::*;
    use player::components::PlayerBundle;
    use player::utils::{is_player_running, is_player_walking};
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
        Rect2DCollider::new(Vec2::new(5.0, 5.0), Vec2::new(0.0, -5.0)),
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
    player: Query<Entity, With<character::components::PlayerControlled>>,
) {
    if let Ok(player_entity) = player.single() {
        for mut followable in camera.iter_mut() {
            followable.target = Some(player_entity);
        }
    }
}
