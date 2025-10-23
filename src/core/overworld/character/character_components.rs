use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct PlayerControlled;

#[derive(Clone, Component)]
#[component(storage = "SparseSet")]
pub(crate) struct Idle;

#[derive(Clone, Component)]
#[component(storage = "SparseSet")]
pub(crate) struct Walking;

#[derive(Clone, Component)]
#[component(storage = "SparseSet")]
pub(crate) struct Running;
