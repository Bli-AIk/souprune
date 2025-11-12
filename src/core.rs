mod basic_components;
pub(crate) mod input;
pub(crate) mod overworld;
pub(crate) mod setup;
pub(crate) mod sprite;

mod animation;
mod camera;
mod systems;

use crate::core::animation::AnimationPlugin;
use crate::core::camera::*;
use crate::extra::toml::config::TomlConfigRegistry;
use bevy::app::*;
use systems::*;

pub(crate) struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TomlConfigRegistry>()
            .add_systems(Update, (update_transform_sync_system,))
            .add_plugins(AnimationPlugin);
    }
}
