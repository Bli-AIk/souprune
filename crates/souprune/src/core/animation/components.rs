use crate::core::sprite::load_context::SpriteLoadContext;
use anyhow::Result;
use bevy::prelude::{Component, Sprite};

#[derive(Component, Default)]
pub(crate) struct SpriteAnimationCurrentFrame {
    pub(crate) value: usize,
}

#[derive(Component)]
pub(crate) struct SpriteAnimationTimer {
    pub(crate) timer: f32,
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

    pub fn reset(&mut self) {
        self.timer = 0.0;
    }
}

#[derive(Component)]
pub(crate) struct SpriteAnimationClip {
    sprites: Vec<Sprite>,
    pub(crate) frame: usize,
    looping: bool,
    clip_name: String,
    #[allow(dead_code)]
    module_name: String,
}

impl SpriteAnimationClip {
    pub fn new(
        sprite_context: &mut SpriteLoadContext,
        module_name: &str,
        clip_name: &str,
    ) -> Result<Self> {
        let (sprites, looping) =
            sprite_context.get_sprite_animations_with_config(module_name, clip_name)?;
        Ok(Self {
            sprites,
            frame: 0,
            looping,
            clip_name: clip_name.to_string(),
            module_name: module_name.to_string(),
        })
    }

    /// Creates a fallback animation clip with a single default sprite.
    /// Used when the requested animation fails to load, preventing repeated load attempts.
    pub fn fallback(module_name: &str, clip_name: &str) -> Self {
        Self {
            sprites: vec![Sprite::default()],
            frame: 0,
            looping: false,
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
