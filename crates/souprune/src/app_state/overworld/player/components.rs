use crate::core::animation::components::SpriteAnimationClip;
use crate::core::basic_components::{Direction, Facing, Speed};
use bevy::math::Vec2;
use bevy::prelude::{Bundle, GlobalTransform, Sprite, Transform};

#[derive(Bundle)]
pub struct PlayerBundle {
    facing: Facing,
    speed: Speed,
    sprite: Sprite,
    anim: SpriteAnimationClip,
    transform: Transform,
    global_transform: GlobalTransform,
}

impl PlayerBundle {
    pub fn new(spawn_pos: Vec2, facing: Direction, anim: SpriteAnimationClip) -> Self {
        Self {
            facing: Facing { value: facing },
            speed: Speed { value: 100.0 },
            sprite: Sprite::default(),
            anim,
            transform: Transform::from_translation(spawn_pos.extend(0.0)),
            global_transform: GlobalTransform::default(),
        }
    }
}
