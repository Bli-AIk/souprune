pub(crate) mod components;
mod systems;

use crate::core::animation::systems::*;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::IntoScheduleConfigs;

pub(crate) struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                sync_sprite_animation_system,
                animate_sprite_system,
                update_sprite_animation_system,
                setup_sprite_animation_clip_system,
            )
                .chain(),
        );
    }
}
