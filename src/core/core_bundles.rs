use crate::core::core_components::*;
use bevy::prelude::*;
#[derive(Bundle)]
pub struct CharacterBundle {
    position: Position,
    rotation: Rotation,
    facing: Facing,
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
            position: Position(spawn_pos),
            rotation: Rotation(0.0),
            facing: Facing(facing),
            sprite,
            health: Health {
                current: 20,
                max: 20,
            },
            anim: AnimationState {
                clip: "idle_down".into(),
            },
            state_timer: StateTimer(0.0),
            transform: Transform::from_translation(spawn_pos.extend(0.0)),
            global_transform: GlobalTransform::default(),
        }
    }
}
