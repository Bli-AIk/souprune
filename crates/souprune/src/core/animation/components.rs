//! ECS components for sprite animation.
//!
//! 精灵动画 ECS 组件。

use crate::core::sprite::load_context::SpriteLoadContext;
use anyhow::Result;
use bevy::prelude::{Component, Sprite};

#[derive(Component, Default)]
pub(crate) struct SpriteAnimationCurrentFrame {
    pub(crate) value: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_entry_identity_includes_flip_flags() {
        let clip = SpriteAnimationClip {
            sprites: vec![Sprite::default()],
            frame: 0,
            looping: true,
            clip_path: "characters/frisk/walk/side".to_string(),
            flip_x: false,
            flip_y: false,
            frame_duration: 0.15,
        };

        assert!(clip.matches_entry("characters/frisk/walk/side", false, false));
        assert!(!clip.matches_entry("characters/frisk/walk/side", true, false));
        assert!(!clip.matches_entry("characters/frisk/walk/side", false, true));
    }
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
    /// Path used as identity to detect animation switches.
    clip_path: String,
    flip_x: bool,
    flip_y: bool,
    pub(crate) frame_duration: f32,
}

impl SpriteAnimationClip {
    /// Create a new animation clip from a module-relative path.
    ///
    /// `relative_path` — e.g. `"characters/hero/walk/down"` (directory) or single file.
    pub fn new(
        sprite_context: &mut SpriteLoadContext,
        module_name: &str,
        relative_path: &str,
        flip_x: bool,
        flip_y: bool,
        looping: bool,
        frame_duration: f32,
    ) -> Result<Self> {
        let sprites =
            sprite_context.get_sprite_animations(module_name, relative_path, flip_x, flip_y)?;
        Ok(Self {
            sprites,
            frame: 0,
            looping,
            clip_path: relative_path.to_string(),
            flip_x,
            flip_y,
            frame_duration,
        })
    }

    /// Fallback animation clip with a single missing-texture sprite.
    pub fn fallback(
        sprite_context: &mut SpriteLoadContext,
        relative_path: &str,
        frame_duration: f32,
    ) -> Self {
        Self {
            sprites: vec![sprite_context.get_missing_sprite()],
            frame: 0,
            looping: false,
            clip_path: relative_path.to_string(),
            flip_x: false,
            flip_y: false,
            frame_duration,
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

    pub fn clip_path(&self) -> &str {
        &self.clip_path
    }

    pub fn matches_entry(&self, relative_path: &str, flip_x: bool, flip_y: bool) -> bool {
        self.clip_path == relative_path && self.flip_x == flip_x && self.flip_y == flip_y
    }
}
