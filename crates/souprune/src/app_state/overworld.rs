use crate::app_state::AppState;
use crate::core::camera::Followable;
use crate::core::input::PlayerInputSettings;
use crate::core::sprite::params::SpriteParams;
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use seldom_state::trigger::*;

pub(crate) mod character;
mod player;
pub(crate) mod tilemap;

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
    player::spawn_overworld_player(&mut commands, &mut sprite_params, &player_input);
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
