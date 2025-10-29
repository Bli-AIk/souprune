use crate::core::components::*;
use bevy::math::Vec2;
use bevy::prelude::{Bundle, GlobalTransform, Sprite, Transform};

pub(crate) mod components;
pub(crate) mod systems;

#[derive(Bundle)]
pub struct CharacterBundle {
    position: Position,
    rotation: Rotation,
    facing: Facing,
    speed: Speed,
    sprite: Sprite,
    health: Health,
    anim: AnimationState,
    state_timer: StateTimer,
    transform: Transform,
    global_transform: GlobalTransform,
}

impl CharacterBundle {
    pub fn new(spawn_pos: Vec2, facing: Direction, sprite: Sprite) -> Self {
        Self {
            position: Position { value: spawn_pos },
            rotation: Rotation { angle: 0.0 },
            facing: Facing { value: facing },
            speed: Speed { value: 50.0 },
            sprite,
            health: Health {
                current: 20,
                max: 20,
            },
            anim: AnimationState {
                clip: "idle_down".into(),
            },
            state_timer: StateTimer { value: 0.0 },
            transform: Transform::from_translation(spawn_pos.extend(0.0)),
            global_transform: GlobalTransform::default(),
        }
    }
}
