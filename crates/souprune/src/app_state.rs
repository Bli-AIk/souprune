//! # app_state.rs
//!
//! ## 模块概述
//!
//! 定义应用程序状态、序列模式、模式作用域和动态子状态。
//! 全部由数据驱动，无硬编码模式枚举。

use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::prelude::*;

pub mod app_setup;
pub mod battle;
pub mod overworld;

/// 应用程序生命周期状态。Loading = 资源加载，Running = 序列运行中。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum AppState {
    #[default]
    Loading,
    Running,
}

/// 当前序列模式。由序列 RON 文件的 `mode` 字段声明，决定哪些系统集激活。
/// 替代原 `GameMode` 枚举——完全数据驱动。
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct SequenceMode(pub Option<String>);

impl SequenceMode {
    pub fn is(&self, mode: &str) -> bool {
        self.0.as_deref() == Some(mode)
    }
}

/// 判断当前模式是否匹配。用于系统集的 `run_if` 条件。
pub fn is_mode(mode: &'static str) -> impl Fn(Res<SequenceMode>) -> bool + Clone {
    move |res: Res<SequenceMode>| res.0.as_deref() == Some(mode)
}

/// 将实体与特定模式关联。模式切换时旧模式的实体被自动清理。
/// 替代原 `OverworldEntity` / `BattleEntity` 标记。
#[derive(Component, Clone, Debug)]
pub struct ModeScoped(pub String);

/// 模式变更事件。在 `SequenceMode` 资源变化时由检测系统自动发出。
#[derive(Message, Debug, Clone)]
pub struct ModeChanged {
    pub from: Option<String>,
    pub to: Option<String>,
}

/// 模式内的动态子状态。替代原 `OverworldSubState`，任何模式都可使用。
/// 模式切换时自动重置为 "Normal"。
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

/// 检测 `SequenceMode` 变化并发出 `ModeChanged` 事件。同时重置 `SequenceSubState`。
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

/// 模式切换时清理旧模式的 `ModeScoped` 实体。
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
