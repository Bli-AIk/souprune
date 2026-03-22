//! Defines the action enums embedded in sequence chapters for camera, UI, and player changes.
//!
//! 定义序列章节中会直接携带的相机、UI 与玩家动作枚举。
//!
//! These enums are part of the sequence asset schema rather than runtime
//! execution state. They describe the declarative operations a chapter wants to
//! perform so loaders, editors, and runtime executors can all speak the same
//! serialized language.
//!
//! 这些枚举属于序列资产 schema，而不是运行时执行状态。它们描述的是章节希望
//! 进行的声明式操作，使加载器、编辑器和运行时执行器都能围绕同一种序列化语言工作。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum CameraAction {
    SetPosition(Vec2),
    SetZoom(f32),
    Shake { duration: f32, intensity: f32 },
    FollowPlayer(bool),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum UIAction {
    LoadLayout(String),
    Show(String),
    Hide(String),
    SetText { id: String, content: String },
    SetVariable { name: String, value: String },
    PlayAnimation { id: String, clip: String },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PlayerAction {
    SetMode(Vec<String>),
    Spawn {
        config_path: String,
        #[serde(default)]
        position: Option<Vec2>,
    },
    Teleport(Vec2),
    SetActive(bool),
    Despawn,
}
