use crate::app_state::AppState;
use crate::core::camera::Followable;
use bevy::app::{App, Plugin};
use bevy::prelude::*;

pub(crate) mod character;
pub(crate) mod player;
pub(crate) mod tilemap;
pub(crate) mod ui;

/// Overworld substates
///
/// Overworld 子状态
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub(crate) enum OverworldState {
    #[default]
    Normal,
    Backpack,
    Cutscene,
}

pub(crate) struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<OverworldState>()
            .add_plugins((
                tilemap::TilemapPlugin,
                player::PlayerPlugin,
                character::CharacterPlugin,
                ui::OverworldUIPlugin,
            ))
            .add_systems(
                OnEnter(AppState::Overworld),
                create_overworld_entities_system,
            )
            .add_systems(Update, bind_camera_target_system)
            .add_systems(
                OnEnter(OverworldState::Backpack),
                player::force_player_idle_on_state_change_system,
            )
            .add_systems(
                OnEnter(OverworldState::Cutscene),
                player::force_player_idle_on_state_change_system,
            );
    }
}

fn create_overworld_entities_system(mut spawn_events: MessageWriter<player::SpawnPlayerRequest>) {
    spawn_events.write(player::SpawnPlayerRequest);
}

fn bind_camera_target_system(
    mut camera: Query<&mut Followable, With<Camera2d>>,
    player: Query<Entity, Added<character::components::PlayerControlled>>,
) {
    for player_entity in player.iter() {
        for mut followable in camera.iter_mut() {
            followable.target = Some(player_entity);
        }
    }
}
