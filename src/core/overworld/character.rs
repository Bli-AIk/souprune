use crate::core::animation::SpriteAnimationClip;
use crate::core::basic_components::*;
use crate::core::input::Action;
use crate::core::overworld::character::components::PlayerControlled;
use bevy::math::Vec2;
use bevy::prelude;
use bevy::prelude::{Bundle, GlobalTransform, Query, Sprite, Transform, With};
use leafwing_input_manager::action_state::ActionState;

pub(crate) mod components;
pub(crate) mod systems;

// CharacterBundle 应当为简化的 PlayerBundle，用于非玩家角色
#[derive(Bundle)]
pub struct PlayerBundle {
    position: Position,
    rotation: Rotation,
    facing: Facing,
    speed: Speed,
    sprite: Sprite,
    health: BasicAttributes,
    anim: SpriteAnimationClip,
    transform: Transform,
    global_transform: GlobalTransform,
}

impl PlayerBundle {
    pub fn new(spawn_pos: Vec2, facing: Direction, anim: SpriteAnimationClip) -> Self {
        Self {
            position: Position { value: spawn_pos },
            rotation: Rotation { angle: 0.0 },
            facing: Facing { value: facing },
            speed: Speed { value: 100.0 },
            sprite: Sprite::default(),
            health: BasicAttributes {
                hp_current: 20,
                hp_max: 20,
                atk: 10,
                def: 10,
            },
            anim,
            transform: Transform::from_translation(spawn_pos.extend(0.0)),
            global_transform: GlobalTransform::default(),
        }
    }
}

pub fn is_walking(
    query: Query<&ActionState<Action>, With<PlayerControlled>>,
) -> prelude::Result<(), ()> {
    let action_state = query.single().map_err(|_| ())?;
    if action_state.pressed(&Action::Left)
        || action_state.pressed(&Action::Right)
        || action_state.pressed(&Action::Up)
        || action_state.pressed(&Action::Down)
    {
        Ok(())
    } else {
        Err(())
    }
}

pub fn is_running(
    query: Query<&ActionState<Action>, With<PlayerControlled>>,
) -> prelude::Result<(), ()> {
    let action_state = query.single().map_err(|_| ())?;
    if action_state.pressed(&Action::Cancel) {
        Ok(())
    } else {
        Err(())
    }
}
