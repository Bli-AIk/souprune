use super::super::layout::ViewLayoutAsset;
use crate::app_state::overworld::OverworldSubState;
use crate::core::input::Action;
use bevy::prelude::*;
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Resource)]
pub struct ViewLayoutHandle {
    pub handle: Handle<ViewLayoutAsset>,
    pub last_modified: Option<SystemTime>,
    /// Layout asset path (e.g., "battle/view/undertale.view_layout.ron")
    ///
    /// 布局资源路径（例如 "battle/view/undertale.view_layout.ron"）
    pub path: String,
}

#[derive(Component)]
pub struct RonDrivenView;

#[derive(Resource, Default)]
pub struct ViewLayoutWatcher {
    pub(crate) timer: Timer,
    pub pending_reload: bool,
}

impl ViewLayoutWatcher {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            pending_reload: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct ViewGlobalTriggerConfig {
    pub triggers: HashMap<Action, Vec<GlobalTriggerRule>>,
}

#[derive(Clone)]
pub struct GlobalTriggerRule {
    pub target_state: OverworldSubState,
    pub sound: Option<String>,
    pub allowed_states: Vec<OverworldSubState>,
}

#[derive(Component)]
pub struct ViewGenerated;
