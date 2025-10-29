mod components;
pub(crate) mod input;
pub(crate) mod overworld;
pub(crate) mod setup;
pub(crate) mod sprite;

mod animation;
mod systems;

use crate::core::sprite::TomlConfigRegistry;
use bevy::app::*;
use systems::*;

pub(crate) struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TomlConfigRegistry>()
            .add_systems(Update, update_transform_sync_system);
    }
}
