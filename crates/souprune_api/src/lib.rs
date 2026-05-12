//! WIT interface definitions for the SoupRune mod system.
//!
//! This crate hosts the WIT (WebAssembly Interface Types) file that defines the
//! contract between Host (framework) and Guest (mod). The WIT source lives in
//! `wit/souprune-mod.wit` and is consumed by both `wasmtime::component::bindgen!`
//! (host side) and `wit_bindgen::generate!` (guest side).
//!
//! Additionally, shared plain-Rust types that mirror WIT records are provided here
//! so that framework code which does *not* link against wasmtime can still work with
//! the same data shapes.
//!
//! SoupRune 模组系统的 WIT 接口定义。
//! WIT 源文件位于 `wit/souprune-mod.wit`，由宿主侧和客体侧共同使用。

/// Semantic input action, mirrors the WIT `action` enum.
///
/// 语义输入动作，对应 WIT 中的 `action` 枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Action {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    Confirm = 4,
    Cancel = 5,
    Menu = 6,
}

/// High-level input context identifier, mirrors the WIT `input-context-id` record.
///
/// 高层输入上下文标识，对应 WIT 中的 `input-context-id` 记录。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputContextId {
    Overworld,
    Battle,
    Dialogue,
    View,
    Custom(String),
}

/// Navigation direction used by input commands, mirrors the WIT `direction` enum.
///
/// 输入命令使用的导航方向，对应 WIT 中的 `direction` 枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Semantic input command, mirrors the WIT `input-command` variant.
///
/// 语义输入命令，对应 WIT 中的 `input-command` 变体。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputCommand {
    Navigate(Direction),
    Confirm,
    Cancel,
    Menu,
}

/// 2D vector, mirrors the WIT `vec2` record.
///
/// 二维向量，对应 WIT 中的 `vec2` 记录。
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
}

/// Named property for danmaku behaviors, mirrors the WIT `prop` record.
///
/// 弹幕行为的命名属性，对应 WIT 中的 `prop` 记录。
#[derive(Debug, Clone)]
pub struct Prop {
    pub name: String,
    pub value: f32,
}

/// Bullet context passed to danmaku callbacks, mirrors the WIT `bullet-context` record.
///
/// 传递给弹幕回调的上下文，对应 WIT 中的 `bullet-context` 记录。
#[derive(Debug, Clone, Default)]
pub struct BulletContext {
    pub elapsed: f32,
    pub delta_time: f32,
    pub spawn_pos: Vec2,
    pub offset: Vec2,
    pub initial_angle: f32,
    pub initial_radius: f32,
    pub player_pos: Vec2,
    pub props: Vec<Prop>,
}

/// Output from a danmaku on-update call, mirrors the WIT `bullet-output` record.
///
/// 弹幕 on-update 调用的输出，对应 WIT 中的 `bullet-output` 记录。
#[derive(Debug, Clone, Copy, Default)]
pub struct BulletOutput {
    pub offset: Vec2,
    pub rotation: f32,
    /// Opacity override. Negative means no change.
    pub opacity: f32,
    /// Scale delta (0.0 = no change from base scale).
    pub scale_x: f32,
    pub scale_y: f32,
}

impl BulletOutput {
    pub const ZERO: Self = Self {
        offset: Vec2::ZERO,
        rotation: 0.0,
        opacity: -1.0,
        scale_x: 0.0,
        scale_y: 0.0,
    };

    pub fn new(x: f32, y: f32) -> Self {
        Self {
            offset: Vec2::new(x, y),
            ..Self::ZERO
        }
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }
}

// ============================================================================
// Spawn Pattern Types
// ============================================================================

/// A computed spawn point, mirrors the WIT `spawn-point` record.
#[derive(Debug, Clone, Copy, Default)]
pub struct SpawnPoint {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub radius: f32,
}

/// Context for spawn pattern generation, mirrors the WIT `spawn-context` record.
#[derive(Debug, Clone, Copy, Default)]
pub struct SpawnContext {
    pub center_x: f32,
    pub center_y: f32,
    pub player_x: f32,
    pub player_y: f32,
    pub time: f32,
}

/// Named parameter for spawn patterns, mirrors the WIT `pattern-param` record.
#[derive(Debug, Clone)]
pub struct PatternParam {
    pub name: String,
    pub value: f64,
}

/// Returns the path to the WIT directory shipped with this crate.
///
/// 返回此 crate 附带的 WIT 目录路径。
pub fn wit_dir() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/wit")
}
