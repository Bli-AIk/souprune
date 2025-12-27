//! # patterns.rs
//!
//! ## Module Overview
//!
//! Defines bullet pattern types and the pattern registry.
//!
//! 定义弹幕模式类型和模式注册表。

use bevy::prelude::*;
use std::collections::HashMap;

/// Event to spawn a bullet pattern.
///
/// 生成弹幕模式的事件。
#[derive(bevy::ecs::message::Message)]
pub struct SpawnPatternEvent {
    pub pattern_id: String,
    pub position: Vec2,
}

impl SpawnPatternEvent {
    pub fn new(pattern_id: impl Into<String>) -> Self {
        Self {
            pattern_id: pattern_id.into(),
            position: Vec2::ZERO,
        }
    }

    pub fn at_position(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }
}

/// Built-in pattern types.
///
/// 内置弹幕模式类型。
#[derive(Debug, Clone)]
pub enum PatternType {
    /// Circle of pellets that converge on player (Flowey style)
    FloweyPelletsCircle {
        count: usize,
        radius: f32,
        converge_speed: f32,
        lifetime: f32,
    },
    /// Sweeping spears from one side (Undyne style)
    UndyneSpearSweep {
        count: usize,
        direction: SpearDirection,
        speed: f32,
        spacing: f32,
        lifetime: f32,
    },
}

/// Direction for spear attacks.
///
/// 矛攻击的方向。
#[derive(Debug, Clone, Copy)]
pub enum SpearDirection {
    FromLeft,
    FromRight,
    FromTop,
    FromBottom,
}

impl SpearDirection {
    pub fn to_vec2(self) -> Vec2 {
        match self {
            SpearDirection::FromLeft => Vec2::new(1.0, 0.0),
            SpearDirection::FromRight => Vec2::new(-1.0, 0.0),
            SpearDirection::FromTop => Vec2::new(0.0, -1.0),
            SpearDirection::FromBottom => Vec2::new(0.0, 1.0),
        }
    }

    pub fn start_offset(self, screen_margin: f32) -> Vec2 {
        match self {
            SpearDirection::FromLeft => Vec2::new(-screen_margin, 0.0),
            SpearDirection::FromRight => Vec2::new(screen_margin, 0.0),
            SpearDirection::FromTop => Vec2::new(0.0, screen_margin),
            SpearDirection::FromBottom => Vec2::new(0.0, -screen_margin),
        }
    }

    /// Get the rotation angle in radians for the spear sprite.
    pub fn rotation_angle(self) -> f32 {
        match self {
            SpearDirection::FromLeft => 0.0,
            SpearDirection::FromRight => std::f32::consts::PI,
            SpearDirection::FromTop => -std::f32::consts::FRAC_PI_2,
            SpearDirection::FromBottom => std::f32::consts::FRAC_PI_2,
        }
    }
}

/// Registry for bullet patterns.
///
/// 弹幕模式注册表。
#[derive(Resource)]
pub struct PatternRegistry {
    patterns: HashMap<String, PatternType>,
}

impl Default for PatternRegistry {
    fn default() -> Self {
        let mut registry = Self {
            patterns: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }
}

impl PatternRegistry {
    fn register_defaults(&mut self) {
        // Register Flowey pellet circle pattern
        self.patterns.insert(
            "flowey_pellets_circle".to_string(),
            PatternType::FloweyPelletsCircle {
                count: 12,
                radius: 120.0,
                converge_speed: 80.0,
                lifetime: 5.0,
            },
        );

        // Register Undyne spear sweep pattern
        self.patterns.insert(
            "undyne_spear_sweep".to_string(),
            PatternType::UndyneSpearSweep {
                count: 5,
                direction: SpearDirection::FromLeft,
                speed: 200.0,
                spacing: 30.0,
                lifetime: 3.0,
            },
        );
    }

    pub fn get(&self, id: &str) -> Option<&PatternType> {
        self.patterns.get(id)
    }

    pub fn register(&mut self, id: impl Into<String>, pattern: PatternType) {
        self.patterns.insert(id.into(), pattern);
    }
}
