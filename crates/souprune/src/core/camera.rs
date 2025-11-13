pub(crate) mod components;
mod systems;

pub(crate) use components::*;

use crate::core::camera::systems::*;
use bevy::app::{App, Plugin, Update};

pub(crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_followable_camera_system);
    }
}
