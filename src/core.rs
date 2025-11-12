mod basic_components;
pub(crate) mod input;
pub(crate) mod overworld;
pub(crate) mod setup;
pub(crate) mod sprite;

mod animation;
mod camera;

use crate::core::animation::AnimationPlugin;
use crate::core::camera::update_followable_camera_system;
use crate::extra::toml::config::TomlConfigRegistry;
use bevy::app::*;

/// CorePlugin is a global plugin that runs early in the app lifecycle.
/// It should be used to initialize global resources and register plugins
/// that need to be available before most systems run.
pub(crate) struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TomlConfigRegistry>()
            .add_plugins(AnimationPlugin);
    }
}

/// GlobalPlugin is a global plugin that runs late in the app lifecycle.
/// It should be used to register systems that need to run after most
/// initialization has completed.
pub(crate) struct GlobalPlugin;

impl Plugin for GlobalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_followable_camera_system);
    }
}
