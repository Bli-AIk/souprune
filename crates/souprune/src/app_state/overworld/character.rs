use bevy::app::{App, Plugin};
use bevy::prelude::*;

pub(crate) mod components;
pub(crate) mod systems;

pub struct CharacterPlugin;
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::update_walking_system,
                systems::update_running_system,
            ),
        );
    }
}
