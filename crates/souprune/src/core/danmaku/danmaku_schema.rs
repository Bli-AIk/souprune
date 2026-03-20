//! Runtime helpers around the shared danmaku schema.
//!
//! `.performance.ron` 的权威数据结构定义位于 `souprune_schema::danmaku`。
//! 本模块只保留运行时资产包装、helper 和事件资源。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub use souprune_schema::danmaku::{
    AimedConfig, BulletBehavior, BulletPrototype, ColliderShape, ColorTint, Easing, EdgeSide,
    HitBehaviorPreset, LinearConfig, OrbitalConfig, SineConfig, SpawnPattern, TimelineEvent,
    TweenConfig,
};

pub type TweenTarget = souprune_schema::danmaku::DanmakuTweenTarget;
pub type WasmCall = (String, HashMap<String, f32>);

/// Runtime asset wrapper for `.performance.ron`.
#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct DanmakuPerformance(pub souprune_schema::danmaku::DanmakuPerformance);

impl Deref for DanmakuPerformance {
    type Target = souprune_schema::danmaku::DanmakuPerformance;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DanmakuPerformance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<souprune_schema::danmaku::DanmakuPerformance> for DanmakuPerformance {
    fn from(value: souprune_schema::danmaku::DanmakuPerformance) -> Self {
        Self(value)
    }
}

pub fn color_tint_to_color(color_tint: &ColorTint) -> Option<Color> {
    if !color_tint.hex.is_empty() {
        parse_hex_color(&color_tint.hex)
    } else if let Some((r, g, b, a)) = color_tint.rgba {
        Some(Color::srgba(r, g, b, a))
    } else {
        None
    }
}

pub fn behavior_to_wasm_call(behavior: &BulletBehavior) -> WasmCall {
    match behavior {
        BulletBehavior::Linear(config) => (
            "builtin.linear".into(),
            HashMap::from([
                ("dir_x".into(), config.dir.0),
                ("dir_y".into(), config.dir.1),
                ("speed".into(), config.speed),
            ]),
        ),
        BulletBehavior::Orbital(config) => (
            "builtin.orbital".into(),
            HashMap::from([
                ("angular_velocity".into(), config.angular_velocity),
                ("radial_velocity".into(), config.radial_velocity),
            ]),
        ),
        BulletBehavior::Sine(config) => (
            "builtin.sine".into(),
            HashMap::from([
                ("axis_x".into(), config.axis.0),
                ("axis_y".into(), config.axis.1),
                ("amplitude".into(), config.amplitude),
                ("frequency".into(), config.frequency),
                ("phase".into(), config.phase),
            ]),
        ),
        BulletBehavior::Tween(config) => (
            "builtin.tween".into(),
            HashMap::from([
                ("target".into(), config.target as u32 as f32),
                ("duration".into(), config.duration),
                ("ease".into(), config.ease as u32 as f32),
                ("range_start".into(), config.range.0),
                ("range_end".into(), config.range.1),
                ("delay".into(), config.delay),
            ]),
        ),
        BulletBehavior::Stationary() => ("builtin.stationary".into(), HashMap::new()),
        BulletBehavior::Aimed(config) => (
            "builtin.aimed".into(),
            HashMap::from([
                ("speed".into(), config.speed),
                ("angle_offset".into(), config.angle_offset),
            ]),
        ),
        BulletBehavior::Custom { id, props } => (id.clone(), props.clone()),
    }
}

pub fn spawn_pattern_to_wasm_call(pattern: &SpawnPattern) -> WasmCall {
    match pattern {
        SpawnPattern::Single => ("builtin.single".into(), HashMap::new()),
        SpawnPattern::RingGenerator {
            count,
            radius,
            start_angle,
        } => (
            "builtin.ring".into(),
            HashMap::from([
                ("count".into(), *count as f32),
                ("radius".into(), *radius),
                ("start_angle".into(), *start_angle),
            ]),
        ),
        SpawnPattern::LineGenerator {
            count,
            spacing,
            direction,
        } => (
            "builtin.line".into(),
            HashMap::from([
                ("count".into(), *count as f32),
                ("spacing".into(), *spacing),
                ("dir_x".into(), direction.0),
                ("dir_y".into(), direction.1),
            ]),
        ),
        SpawnPattern::EdgeGenerator {
            count,
            side,
            spacing,
            margin,
        } => (
            "builtin.edge".into(),
            HashMap::from([
                ("count".into(), *count as f32),
                ("side".into(), *side as u32 as f32),
                ("spacing".into(), *spacing),
                ("margin".into(), *margin),
            ]),
        ),
        SpawnPattern::CustomGenerator { id, params } => (id.clone(), params.clone()),
    }
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            Some(Color::srgb_u8(r, g, b))
        }
        4 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            let a = u8::from_str_radix(&hex[3..4], 16).ok()? * 17;
            Some(Color::srgba_u8(r, g, b, a))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::srgb_u8(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Color::srgba_u8(r, g, b, a))
        }
        _ => None,
    }
}

/// Resource tracking pending performance loads.
#[derive(Resource, Default)]
pub struct PendingPerformanceLoads {
    pub pending: Vec<(Handle<DanmakuPerformance>, PlayPerformanceEvent)>,
}

/// Event to play a danmaku performance.
#[derive(Message, Clone)]
pub struct PlayPerformanceEvent {
    pub performance_path: String,
    pub position: Vec2,
}

impl PlayPerformanceEvent {
    pub fn new(performance_path: impl Into<String>) -> Self {
        Self {
            performance_path: performance_path.into(),
            position: Vec2::ZERO,
        }
    }

    pub fn at_position(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }
}
