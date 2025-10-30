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
            .add_systems(Update, animate_sprite_system)
            .add_systems(Update, update_sprite_animation_system)
            .add_systems(Update, setup_sprite_animation_clip_system);
    }
}

#[derive(Component, Default)]
pub(crate) struct SpriteAnimationCurrentFrame {
    pub(crate) value: usize,
}

#[derive(Component)]
pub(crate) struct SpriteAnimationTimer {
    timer: f32,
    frame_duration: f32,
}

impl SpriteAnimationTimer {
    pub fn new(frame_duration: f32) -> Self {
        Self {
            timer: 0.0,
            frame_duration,
        }
    }

    pub fn tick(&mut self, delta_time: f32) -> bool {
        self.timer += delta_time;
        if self.timer >= self.frame_duration {
            self.timer -= self.frame_duration;
            true
        } else {
            false
        }
    }
}

#[derive(Component)]
pub(crate) struct SpriteAnimationClip {
    sprites: Vec<Sprite>,
    frame: usize,
    looping: bool,
    clip_name: String,
    #[allow(dead_code)]
    module_name: String,
}

impl SpriteAnimationClip {
    pub fn new(sprite_context: &mut SpriteLoadContext, module_name: &str, clip_name: &str) -> Self {
        let (sprites, looping) =
            sprite_context.get_sprite_animations_with_config(module_name, clip_name);
        Self {
            sprites,
            frame: 0,
            looping,
            clip_name: clip_name.to_string(),
            module_name: module_name.to_string(),
        }
    }

    pub fn get_current_sprite(&self) -> &Sprite {
        &self.sprites[self.frame]
    }

    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    pub fn is_looping(&self) -> bool {
        self.looping
    }

    pub fn clip_name(&self) -> &str {
        &self.clip_name
    }
}
