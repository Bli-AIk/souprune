//! # danmaku/systems.rs
//!
//! Runtime systems for the timeline-based danmaku system.
//! Implements the core state-agnostic runtime.

use super::DanmakuSpawnContext;
use super::components::*;
use super::danmaku_schema::*;
use super::target::BulletTarget;
use crate::config::load_config;
use crate::core::animation::components::{SpriteAnimationClip, SpriteAnimationTimer};
use crate::core::collision::TriggerCollider;
use crate::core::mod_system::{
    ActiveDanmakuStack, DanmakuRegistry, LoadedMods, SpawnPatternRegistry,
};
use crate::core::mode::ModeScoped;
use crate::core::sprite::params::SpriteParams;
use crate::core::visual::{
    DEFAULT_FRAME_DURATION, ResolvedVisual, get_asset_path, resolve_visual_path,
};
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use std::collections::HashMap;

mod lifecycle;
mod motion;
mod performance_loading;
mod timeline;

pub use lifecycle::{cleanup_dead_bullets, cleanup_empty_containers, update_bullet_lifetime};
pub use motion::update_bullet_motion;
pub use performance_loading::{process_play_performance_events, spawn_performance_players};
pub use timeline::advance_performance_timeline;
