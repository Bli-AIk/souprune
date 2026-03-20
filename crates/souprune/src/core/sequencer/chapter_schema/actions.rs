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
