use super::super::layout::ViewLayoutAsset;
use crate::app_state::overworld::OverworldState;
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

/// Backwards compatibility alias
///
/// 向后兼容别名
pub type UILayoutHandle = ViewLayoutHandle;

#[derive(Component)]
pub struct RonDrivenView;

/// Backwards compatibility alias
///
/// 向后兼容别名
pub type RonDrivenUI = RonDrivenView;

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

/// Backwards compatibility alias
///
/// 向后兼容别名
pub type UILayoutWatcher = ViewLayoutWatcher;

#[derive(Resource, Default)]
pub struct ViewGlobalTriggerConfig {
    pub triggers: HashMap<Action, Vec<GlobalTriggerRule>>,
}

/// Backwards compatibility alias
///
/// 向后兼容别名
pub type UIGlobalTriggerConfig = ViewGlobalTriggerConfig;

#[derive(Clone)]
pub struct GlobalTriggerRule {
    pub target_state: OverworldState,
    pub sound: Option<String>,
    pub allowed_states: Vec<OverworldState>,
}

#[derive(Component)]
pub struct ViewGenerated;

/// Backwards compatibility alias
///
/// 向后兼容别名
pub type UIGenerated = ViewGenerated;
