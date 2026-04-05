//! Generic runtime mode and state types shared across the engine.
//!
//! 通用运行时模式与状态类型，供整个引擎共享。

use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::prelude::*;

/// Application lifecycle state. Loading = resources loading, Running = gameplay active.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum AppState {
    #[default]
    Loading,
    Running,
}

/// Current sequence mode. Declared by sequence assets to decide which systems should run.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct SequenceMode(pub Option<String>);

impl SequenceMode {
    pub fn is(&self, mode: &str) -> bool {
        self.0.as_deref() == Some(mode)
    }
}

/// Helper for `run_if` checks against the current mode.
pub fn is_mode(mode: &'static str) -> impl Fn(Res<SequenceMode>) -> bool + Clone {
    move |res: Res<SequenceMode>| res.0.as_deref() == Some(mode)
}

/// Marks an entity as belonging to a specific mode for cleanup when the mode changes.
#[derive(Component, Clone, Debug)]
pub struct ModeScoped(pub String);

/// Message emitted when the active mode changes.
#[derive(Message, Debug, Clone)]
pub struct ModeChanged {
    pub from: Option<String>,
    pub to: Option<String>,
}

/// Dynamic sub-state within the active mode.
#[derive(Debug, Clone, PartialEq, Eq, Hash, States)]
pub struct SequenceSubState(pub String);

impl Default for SequenceSubState {
    fn default() -> Self {
        Self("Normal".to_string())
    }
}

impl SequenceSubState {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn is(&self, name: &str) -> bool {
        self.0 == name
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

/// Detect `SequenceMode` changes and emit `ModeChanged`. Also resets `SequenceSubState`.
pub fn detect_mode_changes(
    mode: Res<SequenceMode>,
    mut previous: Local<Option<String>>,
    mut events: MessageWriter<ModeChanged>,
    mut next_sub_state: ResMut<NextState<SequenceSubState>>,
) {
    if mode.is_changed() {
        let from = previous.clone();
        let to = mode.0.clone();
        if from != to {
            events.write(ModeChanged {
                from: from.clone(),
                to: to.clone(),
            });
            next_sub_state.set(SequenceSubState::default());
            *previous = to;
        }
    }
}

/// Despawn entities from the previous mode when a mode change occurs.
pub fn cleanup_mode_scoped_entities(
    mut commands: Commands,
    mut events: MessageReader<ModeChanged>,
    query: Query<(Entity, &ModeScoped)>,
) {
    for event in events.read() {
        let Some(ref from) = event.from else { continue };
        for (entity, scoped) in query.iter() {
            if scoped.0 == *from {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn cleanup_entities_system<T: Component>(
    mut commands: Commands,
    query: Query<Entity, With<T>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Registry of mode names known to the engine.
///
/// Presets register modes here during plugin initialization.
/// The engine core uses this to validate mode transitions without
/// knowing specific mode names.
///
/// 引擎已知模式名注册表。Preset 在初始化时注册模式。
/// 引擎核心通过此注册表验证模式切换，而无需了解具体模式名。
#[derive(Resource, Default, Debug)]
pub struct ModeRegistry {
    /// Set of registered mode names.
    modes: std::collections::HashSet<String>,
}

impl ModeRegistry {
    /// Register a mode name. Call during plugin initialization.
    pub fn register(&mut self, mode: impl Into<String>) {
        self.modes.insert(mode.into());
    }

    /// Check if a mode is registered.
    pub fn is_registered(&self, mode: &str) -> bool {
        self.modes.contains(mode)
    }

    /// Get all registered mode names.
    pub fn modes(&self) -> impl Iterator<Item = &str> {
        self.modes.iter().map(|s| s.as_str())
    }
}
