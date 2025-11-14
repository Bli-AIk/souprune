use bevy::prelude::*;

#[derive(Component, Default)]
pub(crate) struct Followable {
    pub(crate) target: Option<Entity>,
}
