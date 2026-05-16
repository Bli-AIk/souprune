//! Generic runtime mode and state types shared across the framework.
//!
//! 通用运行时模式与状态类型，供整个框架共享。

use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::prelude::*;
use std::collections::HashMap;

use crate::config::{ModeConfig, ModePrimitiveConfig};

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

/// Registry of project-declared runtime modes.
///
/// Projects register modes through `mod.toml`; core systems query this resource
/// to decide which primitive systems are active without hardcoding mode names.
///
/// 项目声明的运行模式注册表。
/// 项目通过 `mod.toml` 注册 mode；core 系统查询此资源来决定哪些 primitive 系统激活，
/// 而不在框架内硬编码 mode 名。
#[derive(Resource, Default, Debug, Clone)]
pub struct ModeRegistry {
    modes: HashMap<String, ModeConfig>,
}

impl ModeRegistry {
    /// Replace all registered modes from project configuration.
    ///
    /// 使用项目配置替换所有已注册 mode。
    pub fn set_modes(&mut self, modes: HashMap<String, ModeConfig>) {
        self.modes = modes;
    }

    /// Check if a mode is registered.
    pub fn is_registered(&self, mode: &str) -> bool {
        self.modes.contains_key(mode)
    }

    /// Get a mode declaration.
    ///
    /// 获取 mode 声明。
    pub fn mode(&self, mode: &str) -> Option<&ModeConfig> {
        self.modes.get(mode)
    }

    /// Check whether a mode enables a primitive.
    ///
    /// 检查 mode 是否启用了某个 primitive。
    pub fn mode_has_primitive(&self, mode: &str, primitive: ModePrimitiveConfig) -> bool {
        self.mode(mode)
            .is_some_and(|mode_config| mode_config.has_primitive(primitive))
    }

    /// Get all registered mode names.
    pub fn modes(&self) -> impl Iterator<Item = &str> {
        self.modes.keys().map(|s| s.as_str())
    }
}

/// Helper for `run_if` checks against the current mode's primitive set.
///
/// 根据当前 mode 的 primitive 集合生成 `run_if` 检查。
pub fn current_mode_has_primitive(
    primitive: ModePrimitiveConfig,
) -> impl Fn(Res<SequenceMode>, Res<ModeRegistry>) -> bool + Clone {
    move |mode: Res<SequenceMode>, registry: Res<ModeRegistry>| {
        mode.0
            .as_deref()
            .is_some_and(|mode_name| registry.mode_has_primitive(mode_name, primitive))
    }
}

/// Helper for mode enter run conditions filtered by primitive.
///
/// 根据 primitive 过滤 mode enter 事件的 `run_if` 辅助函数。
pub fn on_entering_mode_with_primitive(
    primitive: ModePrimitiveConfig,
) -> impl FnMut(MessageReader<ModeChanged>, Res<ModeRegistry>) -> bool + Clone {
    move |mut events: MessageReader<ModeChanged>, registry: Res<ModeRegistry>| {
        events.read().any(|event| {
            event
                .to
                .as_deref()
                .is_some_and(|mode_name| registry.mode_has_primitive(mode_name, primitive))
        })
    }
}

/// Helper for mode exit run conditions filtered by primitive.
///
/// 根据 primitive 过滤 mode exit 事件的 `run_if` 辅助函数。
pub fn on_exiting_mode_with_primitive(
    primitive: ModePrimitiveConfig,
) -> impl FnMut(MessageReader<ModeChanged>, Res<ModeRegistry>) -> bool + Clone {
    move |mut events: MessageReader<ModeChanged>, registry: Res<ModeRegistry>| {
        events.read().any(|event| {
            event
                .from
                .as_deref()
                .is_some_and(|mode_name| registry.mode_has_primitive(mode_name, primitive))
        })
    }
}
