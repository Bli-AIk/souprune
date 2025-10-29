mod core_components;
pub(crate) mod input;
pub(crate) mod overworld;
pub(crate) mod setup;
pub(crate) mod sprite;

mod animation;
mod core_systems;

use bevy::app::*;
use core_systems::*;

pub(crate) struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_transform_sync_system);
    }
}
