//! Provides a safe wrapper around the WIT host-api imports.
//! Mod developers use `Context` to interact with the host framework
//! (logging, input, kinematics) without touching raw WIT types.
//!
//! 对 WIT host-api 导入的安全封装。
//! 模组开发者使用 `Context` 与宿主框架交互（日志、输入、运动学），
//! 无需接触原始 WIT 类型。

use crate::Action;
use crate::souprune::plugin::host_api;

/// Typed fact value mirroring the WIT `fact-value` variant.
/// Mod code uses this instead of raw strings.
#[derive(Debug, Clone, PartialEq)]
pub enum FactValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Text(String),
}

impl FactValue {
    pub(crate) fn from_wit(v: host_api::FactValue) -> Self {
        match v {
            host_api::FactValue::IntVal(n) => Self::Int(n),
            host_api::FactValue::FloatVal(f) => Self::Float(f),
            host_api::FactValue::BoolVal(b) => Self::Bool(b),
            host_api::FactValue::TextVal(s) => Self::Text(s),
        }
    }
}

/// Context is the mod's window into the host framework.
/// It wraps the WIT host-api imports in an ergonomic Rust API.
///
/// Context 是模组与宿主框架交互的窗口。
/// 将 WIT host-api 导入封装为符合 Rust 习惯的 API。
pub struct Context {
    _private: (),
}

impl Context {
    /// Create a new context. Called by the SDK macro before each lifecycle callback.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Print an info-level log message to the host console.
    pub fn log(&self, msg: &str) {
        host_api::log(0, msg);
    }

    /// Print a warning-level log message to the host console.
    pub fn warn(&self, msg: &str) {
        host_api::log(1, msg);
    }

    /// Print an error-level log message to the host console.
    pub fn error(&self, msg: &str) {
        host_api::log(2, msg);
    }

    /// Access input state.
    pub fn input(&self) -> InputHelper {
        InputHelper
    }

    /// Access kinematics control.
    pub fn kinematics(&self) -> KinematicsHelper {
        KinematicsHelper
    }

    /// Get the current entity's world position.
    pub fn entity_position(&self) -> Vec2 {
        let pos = host_api::get_entity_position();
        Vec2::new(pos.x, pos.y)
    }

    /// Get the frame delta time in seconds.
    pub fn delta_time(&self) -> f32 {
        host_api::delta_time()
    }

    /// Get a FRE fact by key as a typed value. Returns None if not set.
    pub fn get_fact(&self, key: &str) -> Option<FactValue> {
        host_api::get_fact(key).map(FactValue::from_wit)
    }

    /// Get a FRE fact as bool. Returns None if not set or not bool.
    pub fn get_fact_bool(&self, key: &str) -> Option<bool> {
        match self.get_fact(key)? {
            FactValue::Bool(b) => Some(b),
            _ => None,
        }
    }

    /// Get a FRE fact as i64. Returns None if not set or not int.
    pub fn get_fact_int(&self, key: &str) -> Option<i64> {
        match self.get_fact(key)? {
            FactValue::Int(n) => Some(n),
            _ => None,
        }
    }

    /// Get a FRE fact as f64. Returns None if not set or not float.
    pub fn get_fact_float(&self, key: &str) -> Option<f64> {
        match self.get_fact(key)? {
            FactValue::Float(f) => Some(f),
            _ => None,
        }
    }

    /// Get a FRE fact as string. Returns None if not set or not text.
    pub fn get_fact_string(&self, key: &str) -> Option<String> {
        match self.get_fact(key)? {
            FactValue::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Set a FRE fact to a bool value.
    pub fn set_fact_bool(&self, key: &str, value: bool) {
        host_api::set_fact(key, &host_api::FactValue::BoolVal(value));
    }

    /// Set a FRE fact to an int value.
    pub fn set_fact_int(&self, key: &str, value: i64) {
        host_api::set_fact(key, &host_api::FactValue::IntVal(value));
    }

    /// Set a FRE fact to a float value.
    pub fn set_fact_float(&self, key: &str, value: f64) {
        host_api::set_fact(key, &host_api::FactValue::FloatVal(value));
    }

    /// Set a FRE fact to a string value.
    pub fn set_fact_string(&self, key: &str, value: &str) {
        host_api::set_fact(key, &host_api::FactValue::TextVal(value.to_string()));
    }

    /// Emit a FRE event by name. Applied after the current callback returns.
    pub fn emit_event(&self, event: &str) {
        host_api::emit_event(event);
    }

    /// Get the position of a named entity by tag. Returns None if not found.
    pub fn get_entity_position_by_tag(&self, tag: &str) -> Option<Vec2> {
        host_api::get_entity_position_by_tag(tag).map(|pos| Vec2::new(pos.x, pos.y))
    }

    /// Spawn a bullet emitter with the given pattern ID at a position.
    /// Returns an opaque handle that can be used to despawn it later.
    pub fn spawn_emitter(&self, pattern_id: &str, x: f32, y: f32) -> u64 {
        host_api::spawn_emitter(pattern_id, host_api::Vec2 { x, y })
    }

    /// Despawn a previously spawned emitter by handle.
    pub fn despawn_emitter(&self, handle: u64) {
        host_api::despawn_emitter(handle);
    }

    /// Open a named view (must be defined in a `.view_layout.ron` file).
    pub fn open_view(&self, view_id: &str) {
        host_api::open_view(view_id);
    }

    /// Close the currently active view (if any).
    pub fn close_view(&self) {
        host_api::close_view();
    }

    /// Play a sound effect by asset key.
    pub fn play_sound(&self, sound_key: &str) {
        host_api::play_sound(sound_key);
    }

    /// Get the current mode name. Returns None if no mode is active.
    pub fn get_current_mode(&self) -> Option<String> {
        host_api::get_current_mode()
    }

    /// Get the current sub-state name within the active mode.
    pub fn get_current_sub_state(&self) -> String {
        host_api::get_current_sub_state()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

/// Input query helper (zero-sized, stateless).
pub struct InputHelper;

impl InputHelper {
    /// Check if a semantic action is currently pressed.
    pub fn pressed(&self, action: Action) -> bool {
        host_api::is_action_pressed(action.to_wit())
    }

    /// Check if a semantic action was just pressed this frame.
    pub fn just_pressed(&self, action: Action) -> bool {
        host_api::is_action_just_pressed(action.to_wit())
    }
}

/// Kinematics control helper (zero-sized, stateless).
pub struct KinematicsHelper;

impl KinematicsHelper {
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
