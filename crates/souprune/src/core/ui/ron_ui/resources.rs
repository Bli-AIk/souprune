use super::super::layout::UILayoutAsset;
use crate::app_state::overworld::OverworldState;
use crate::core::input::Action;
use bevy::prelude::*;
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Resource)]
pub struct UILayoutHandle {
    pub handle: Handle<UILayoutAsset>,
    pub last_modified: Option<SystemTime>,
}

#[derive(Component)]
pub struct RonDrivenUI;

#[derive(Resource, Default)]
pub struct UILayoutWatcher {
    pub(crate) timer: Timer,
    pub pending_reload: bool,
}

impl UILayoutWatcher {
    pub(crate) fn new() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            pending_reload: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct UIGlobalTriggerConfig {
    pub triggers: HashMap<Action, Vec<GlobalTriggerRule>>,
}

#[derive(Clone)]
pub struct GlobalTriggerRule {
    pub target_state: OverworldState,
    pub sound: Option<String>,
    pub allowed_states: Vec<OverworldState>,
}

#[derive(Component)]
pub struct UIGenerated;
