use bevy::prelude::*;

// TODO:去掉 Position 和 Rotation 组件，直接使用 Transform 组件
#[derive(Component)]
pub(crate) struct Position {
    pub value: Vec2,
}

#[derive(Component)]
pub(crate) struct Rotation {
    pub angle: f32,
}
#[derive(Component)]
pub(crate) struct Speed {
    pub value: f32,
}

#[derive(Component)]
pub(crate) struct Facing {
    pub value: Direction,
}
#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct BasicAttributes {
    pub(crate) hp_current: usize,
    pub(crate) hp_max: usize,
    pub(crate) atk: usize,
    pub(crate) def: usize,
}

#[derive(Default)]
pub(crate) enum Direction {
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
