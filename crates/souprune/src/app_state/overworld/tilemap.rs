use crate::app_state::AppState::Overworld;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

mod systems;

pub(crate) struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Overworld), systems::setup_tilemap_system)
            .add_systems(
                Update,
                (
                    systems::initialize_tilemap_system,
                    systems::update_objects_order_with_player_system,
                ),
            );
    }
}

