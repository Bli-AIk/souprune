use bevy::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Component)]
pub(crate) struct Speed {
    pub value: f32,
}

#[derive(Component)]
pub(crate) struct Facing {
    pub value: Direction,
}

#[derive(Default, Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    #[default]
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}
impl Direction {
    pub fn as_vec2(&self) -> Vec2 {
        match self {
            Direction::Up => Vec2::Y,
            Direction::Down => -Vec2::Y,
            Direction::Left => -Vec2::X,
            Direction::Right => Vec2::X,
            Direction::UpLeft => Vec2::new(-1.0, 1.0).normalize(),
            Direction::UpRight => Vec2::new(1.0, 1.0).normalize(),
            Direction::DownLeft => Vec2::new(-1.0, -1.0).normalize(),
            Direction::DownRight => Vec2::new(1.0, -1.0).normalize(),
        }
    }
}
