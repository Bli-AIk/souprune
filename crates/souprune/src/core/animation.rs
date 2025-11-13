pub(crate) mod component;
mod systems;

use crate::core::animation::systems::*;
use bevy::app::{App, Plugin, Update};
pub(crate) struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_sprite_animation_system)
            .add_systems(Update, animate_sprite_system)
            .add_systems(Update, update_sprite_animation_system)
            .add_systems(Update, setup_sprite_animation_clip_system);
    }
}
