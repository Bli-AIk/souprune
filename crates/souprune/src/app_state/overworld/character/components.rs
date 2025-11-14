use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct PlayerControlled;

#[derive(Clone, Component)]
#[component(storage = "SparseSet")]
pub(crate) struct StateIdle;

#[derive(Clone, Component)]
#[component(storage = "SparseSet")]
pub(crate) struct StateWalking;

#[derive(Clone, Component)]
#[component(storage = "SparseSet")]
pub(crate) struct StateRunning;
