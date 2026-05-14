//! Provides a safe wrapper around the WIT host-api imports.
//! Mod developers use `Context` to interact with the host framework
//! (logging, input, kinematics) without touching raw WIT types.
//!
//! 对 WIT host-api 导入的安全封装。
//! 模组开发者使用 `Context` 与宿主框架交互（日志、输入、运动学），
//! 无需接触原始 WIT 类型。

use crate::souprune::plugin::host_api;
use crate::{Action, ColliderShape, ConstraintHandle, RegionHandle};

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
    ///
    /// 创建新的上下文。由 SDK 宏在每个生命周期回调前调用。
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Print an info-level log message to the host console.
    ///
    /// 向宿主控制台输出 info 级日志。
    pub fn log(&self, msg: &str) {
        host_api::log(0, msg);
    }

    /// Print a warning-level log message to the host console.
    ///
    /// 向宿主控制台输出 warning 级日志。
    pub fn warn(&self, msg: &str) {
        host_api::log(1, msg);
    }

    /// Print an error-level log message to the host console.
    ///
    /// 向宿主控制台输出 error 级日志。
    pub fn error(&self, msg: &str) {
        host_api::log(2, msg);
    }

    /// Access input state.
    ///
    /// 访问输入状态。
    pub fn input(&self) -> InputHelper {
        InputHelper
    }

    /// Access kinematics control.
    ///
    /// 访问运动学控制。
    pub fn kinematics(&self) -> KinematicsHelper {
        KinematicsHelper
    }

    /// Access host-side collision region and movement constraint primitives.
    ///
    /// 访问宿主侧碰撞区域与移动约束 primitive。
    pub fn collision(&self) -> CollisionHelper {
        CollisionHelper
    }

    /// Get the current entity's world position.
    ///
    /// 获取当前实体的世界坐标。
    pub fn entity_position(&self) -> Vec2 {
        let pos = host_api::get_entity_position();
        Vec2::new(pos.x, pos.y)
    }

    /// Get the frame delta time in seconds.
    ///
    /// 获取当前帧间隔秒数。
    pub fn delta_time(&self) -> f32 {
        host_api::delta_time()
    }

    /// Get a FRE fact by key as a typed value. Returns None if not set.
    ///
    /// 按 key 获取类型化 FRE fact。未设置时返回 None。
    pub fn get_fact(&self, key: &str) -> Option<FactValue> {
        host_api::get_fact(key).map(FactValue::from_wit)
    }

    /// Get a FRE fact as bool. Returns None if not set or not bool.
    ///
    /// 按 bool 读取 FRE fact。未设置或类型不匹配时返回 None。
    pub fn get_fact_bool(&self, key: &str) -> Option<bool> {
        match self.get_fact(key)? {
            FactValue::Bool(b) => Some(b),
            _ => None,
        }
    }

    /// Get a FRE fact as i64. Returns None if not set or not int.
    ///
    /// 按 i64 读取 FRE fact。未设置或类型不匹配时返回 None。
    pub fn get_fact_int(&self, key: &str) -> Option<i64> {
        match self.get_fact(key)? {
            FactValue::Int(n) => Some(n),
            _ => None,
        }
    }

    /// Get a FRE fact as f64. Returns None if not set or not float.
    ///
    /// 按 f64 读取 FRE fact。未设置或类型不匹配时返回 None。
    pub fn get_fact_float(&self, key: &str) -> Option<f64> {
        match self.get_fact(key)? {
            FactValue::Float(f) => Some(f),
            _ => None,
        }
    }

    /// Get a FRE fact as string. Returns None if not set or not text.
    ///
    /// 按字符串读取 FRE fact。未设置或类型不匹配时返回 None。
    pub fn get_fact_string(&self, key: &str) -> Option<String> {
        match self.get_fact(key)? {
            FactValue::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Set a FRE fact to a bool value.
    ///
    /// 将 FRE fact 设置为 bool 值。
    pub fn set_fact_bool(&self, key: &str, value: bool) {
        host_api::set_fact(key, &host_api::FactValue::BoolVal(value));
    }

    /// Set a FRE fact to an int value.
    ///
    /// 将 FRE fact 设置为 int 值。
    pub fn set_fact_int(&self, key: &str, value: i64) {
        host_api::set_fact(key, &host_api::FactValue::IntVal(value));
    }

    /// Set a FRE fact to a float value.
    ///
    /// 将 FRE fact 设置为 float 值。
    pub fn set_fact_float(&self, key: &str, value: f64) {
        host_api::set_fact(key, &host_api::FactValue::FloatVal(value));
    }

    /// Set a FRE fact to a string value.
    ///
    /// 将 FRE fact 设置为字符串值。
    pub fn set_fact_string(&self, key: &str, value: &str) {
        host_api::set_fact(key, &host_api::FactValue::TextVal(value.to_string()));
    }

    /// Emit a FRE event by name. Applied after the current callback returns.
    ///
    /// 按名称发出 FRE event。当前回调返回后应用。
    pub fn emit_event(&self, event: &str) {
        host_api::emit_event(event);
    }

    /// Get the position of a named entity by tag. Returns None if not found.
    ///
    /// 按 tag 获取命名实体位置。找不到时返回 None。
    pub fn get_entity_position_by_tag(&self, tag: &str) -> Option<Vec2> {
        host_api::get_entity_position_by_tag(tag).map(|pos| Vec2::new(pos.x, pos.y))
    }

    /// Spawn a bullet emitter with the given pattern ID at a position.
    /// Returns an opaque handle that can be used to despawn it later.
    ///
    /// 在指定位置生成弹幕发射器，返回之后可用于销毁的不透明句柄。
    pub fn spawn_emitter(&self, pattern_id: &str, x: f32, y: f32) -> u64 {
        host_api::spawn_emitter(pattern_id, host_api::Vec2 { x, y })
    }

    /// Despawn a previously spawned emitter by handle.
    ///
    /// 按句柄销毁之前生成的发射器。
    pub fn despawn_emitter(&self, handle: u64) {
        host_api::despawn_emitter(handle);
    }

    /// Open a named view (must be defined in a `.view_layout.ron` file).
    ///
    /// 打开命名 View（必须定义在 `.view_layout.ron` 文件中）。
    pub fn open_view(&self, view_id: &str) {
        host_api::open_view(view_id);
    }

    /// Close the currently active view (if any).
    ///
    /// 关闭当前激活的 View（如果存在）。
    pub fn close_view(&self) {
        host_api::close_view();
    }

    /// Play a sound effect by asset key.
    ///
    /// 按资源 key 播放音效。
    pub fn play_sound(&self, sound_key: &str) {
        host_api::play_sound(sound_key);
    }

    /// Get the current mode name. Returns None if no mode is active.
    ///
    /// 获取当前 mode 名称。没有激活 mode 时返回 None。
    pub fn get_current_mode(&self) -> Option<String> {
        host_api::get_current_mode()
    }

    /// Get the current sub-state name within the active mode.
    ///
    /// 获取当前激活 mode 内的 sub-state 名称。
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
///
/// 输入查询辅助对象（零大小、无状态）。
pub struct InputHelper;

impl InputHelper {
    /// Check if a semantic action is currently pressed.
    ///
    /// 检查语义 action 当前是否被按住。
    pub fn pressed(&self, action: Action) -> bool {
        host_api::is_action_pressed(action.to_wit())
    }

    /// Check if a semantic action was just pressed this frame.
    ///
    /// 检查语义 action 是否在当前帧刚刚按下。
    pub fn just_pressed(&self, action: Action) -> bool {
        host_api::is_action_just_pressed(action.to_wit())
    }
}

/// Kinematics control helper (zero-sized, stateless).
///
/// 运动学控制辅助对象（零大小、无状态）。
pub struct KinematicsHelper;

impl KinematicsHelper {
    /// Set the entity's velocity.
    ///
    /// 设置实体速度。
    pub fn set_velocity(&self, x: f32, y: f32) {
        host_api::set_velocity(host_api::Vec2 { x, y });
    }
}

/// Collision primitive helper (zero-sized, stateless).
///
/// 碰撞 primitive 辅助对象（零大小、无状态）。
pub struct CollisionHelper;

impl CollisionHelper {
    /// Create a host-owned collision region.
    ///
    /// 创建宿主拥有的碰撞区域。
    pub fn create_region(&self, center: Vec2, half_size: Vec2) -> Option<RegionHandle> {
        let raw = host_api::create_collision_region(
            host_api::Vec2 {
                x: center.x,
                y: center.y,
            },
            host_api::Vec2 {
                x: half_size.x,
                y: half_size.y,
            },
        );
        (raw != 0).then_some(RegionHandle(raw))
    }

    /// Remove a host-owned collision region.
    ///
    /// 移除宿主拥有的碰撞区域。
    pub fn remove_region(&self, handle: RegionHandle) {
        host_api::remove_collision_region(handle.0);
    }

    /// Create a host-owned movement constraint for a region and collider.
    ///
    /// 为区域和碰撞体创建宿主拥有的移动约束。
    pub fn create_movement_constraint(
        &self,
        region: RegionHandle,
        collider: ColliderShape,
    ) -> Option<ConstraintHandle> {
        host_api::create_movement_constraint(region.0, collider.to_wit()).map(ConstraintHandle)
    }

    /// Remove a host-owned movement constraint.
    ///
    /// 移除宿主拥有的移动约束。
    pub fn remove_movement_constraint(&self, handle: ConstraintHandle) {
        host_api::remove_movement_constraint(handle.0);
    }

    /// Constrain a target position through a host-owned movement constraint.
    ///
    /// 通过宿主拥有的移动约束限制目标位置。
    pub fn constrain_movement(&self, handle: ConstraintHandle, position: Vec2) -> Option<Vec2> {
        host_api::constrain_movement(
            handle.0,
            host_api::Vec2 {
                x: position.x,
                y: position.y,
            },
        )
        .map(|pos| Vec2::new(pos.x, pos.y))
    }
}

/// Simple 2D vector for SDK use.
///
/// SDK 使用的简单二维向量。
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// Create a vector from x and y components.
    ///
    /// 从 x 和 y 分量创建向量。
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Return the vector length.
    ///
    /// 返回向量长度。
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Return a normalized vector, or zero for near-zero input.
    ///
    /// 返回归一化向量；输入接近零时返回零向量。
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
