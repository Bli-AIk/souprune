//! Provides a safe wrapper around the WIT host-api imports.
//! Mod developers use `Context` to interact with the game engine
//! (logging, input, kinematics) without touching raw WIT types.
//!
//! 对 WIT host-api 导入的安全封装。
//! 模组开发者使用 `Context` 与引擎交互（日志、输入、运动学），
//! 无需接触原始 WIT 类型。

use crate::Action;
use crate::souprune::plugin::host_api;

/// Context is the mod's window into the engine.
/// It wraps the WIT host-api imports in an ergonomic Rust API.
///
/// Context 是模组与引擎交互的窗口。
/// 将 WIT host-api 导入封装为符合 Rust 习惯的 API。
pub struct Context {
    _private: (),
}

impl Context {
    /// Create a new context. Called by the SDK macro before each lifecycle callback.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Print a log message to the engine console.
    pub fn log(&self, msg: &str) {
        host_api::log(0, msg);
    }

    /// Access input state.
    pub fn input(&self) -> InputHelper<'_> {
        InputHelper { _ctx: self }
    }

    /// Access kinematics control.
    pub fn kinematics(&self) -> KinematicsHelper<'_> {
        KinematicsHelper { _ctx: self }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

/// Input module wrapper.
pub struct InputHelper<'a> {
    _ctx: &'a Context,
}

impl InputHelper<'_> {
    /// Check if a semantic action is currently pressed.
    pub fn pressed(&self, action: Action) -> bool {
        host_api::is_action_pressed(action.to_wit())
    }
}

/// Kinematics module wrapper.
pub struct KinematicsHelper<'a> {
    _ctx: &'a Context,
}

impl KinematicsHelper<'_> {
    /// Set the entity's velocity.
    pub fn set_velocity(&self, x: f32, y: f32) {
        host_api::set_velocity(host_api::Vec2 { x, y });
    }
}

/// Simple 2D vector for SDK use.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0001 {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            Self::ZERO
        }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}
