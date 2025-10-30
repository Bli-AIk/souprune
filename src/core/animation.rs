mod systems;

use crate::core::animation::systems::*;
use crate::core::sprite::load_context::SpriteLoadContext;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::Component;
use bevy::sprite::Sprite;

pub(crate) struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_sprite_animation_system)
            .add_systems(Update, update_sprite_animation_system)
            .add_systems(Update, setup_sprite_animation_clip_system);
    }
}

#[derive(Component, Default)]
pub(crate) struct SpriteAnimationCurrentFrame {
    value: usize,
}

#[derive(Component)]
pub(crate) struct SpriteAnimationClip {
    sprites: Vec<Sprite>,
    frame: usize,
}

impl SpriteAnimationClip {
    pub fn new(sprite_context: &mut SpriteLoadContext, module_name: &str, clip_name: &str) -> Self {
        Self {
            sprites: sprite_context.get_sprite_animations(module_name, clip_name),
            frame: 0,
        }
    }
    pub fn get_current_sprite(&self) -> &Sprite {
        &self.sprites[self.frame]
    }
}
