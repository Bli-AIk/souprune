//! Defines the fundamental C ABI for the modding system.
//! It contains raw C-compatible structs, enums, and function pointer definitions
//! that act as the contract between the Host (Game Engine) and the Guest (Mod).
//!
//! 定义了模组系统的基础 C ABI。
//! 包含原始的 C 兼容结构体、枚举和函数指针定义，
//! 作为宿主（游戏引擎）和客体（模组）之间的契约。

pub use core::ffi::{c_float, c_void};

/// A non-opaque context handle.
/// On the Bevy side, it points to a struct containing the Entity World ID.
/// On the Mod side, it is just a pointer with no fields that must be passed back to the Host as is.
///
/// 一个不透明的上下文句柄。
/// 在 Bevy 侧，它指向一个包含 Entity World ID 的结构体。
/// 在 Mod 侧，它只是一个没有任何字段的指针，必须原样传回给 Host。
#[repr(C)]
pub struct ContextHandle {
    _private: [u8; 0],
}

/// Corresponds to the Action enum in souprune core, but fixed in the API layer to ensure ABI compatibility.
///
/// 对应 souprune 核心的 Action 枚举，但在 API 层固定下来以保证 ABI 兼容
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    Confirm = 4,
    Cancel = 5,
    Menu = 6,
}

#[repr(C)]
pub struct HostApi {
    pub log: extern "C" fn(level: u32, msg: *const u8, len: usize),

    // === Sensors (传感器) ===
    pub input_is_action_pressed:
        extern "C" fn(context: *const ContextHandle, action: Action) -> bool,

    // === Actuators (执行器) ===
    pub kinematics_set_velocity: extern "C" fn(context: *mut ContextHandle, x: c_float, y: c_float),
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BehaviorVTable {
    pub on_enter: Option<extern "C" fn(instance: *mut c_void, context: *mut ContextHandle)>,

    pub on_update: Option<
        extern "C" fn(instance: *mut c_void, context: *mut ContextHandle, delta_time: c_float),
    >,

    pub on_exit: Option<extern "C" fn(instance: *mut c_void, context: *mut ContextHandle)>,

    pub destroy: Option<extern "C" fn(instance: *mut c_void)>,
}

#[repr(C)]
pub struct BehaviorInstance {
    pub instance: *mut c_void,
    pub vtable: BehaviorVTable,
}

/// Function type to get the number of exported behaviors in the DLL.
pub type GetBehaviorCountFn = extern "C" fn() -> u32;

/// Function type to get the ID of a behavior by index.
/// Returns null if index is out of bounds.
pub type GetBehaviorIdFn = extern "C" fn(index: u32) -> *const u8;

/// Handshake protocol: This is the type signature of the function exported by the DLL to create a behavior instance.
/// The engine will look for a symbol named "create_behavior".
///
/// 握手协议：这是 DLL 导出的用于创建 behavior 实例的函数类型签名
/// 引擎会查找名为 "create_behavior" 的符号
pub type CreateBehaviorFn =
    unsafe extern "C" fn(id: *const u8, api: *const HostApi) -> BehaviorInstance;

// ============================================================================
// Danmaku Scripting API (弹幕脚本 API)
// ============================================================================

/// C-compatible 2D vector for FFI.
///
/// 用于 FFI 的 C 兼容二维向量。
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Vec2C {
    pub x: c_float,
    pub y: c_float,
}

impl Vec2C {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: c_float, y: c_float) -> Self {
        Self { x, y }
    }
}

/// Bullet runtime state for C-ABI.
/// This is passed to danmaku algorithm functions for stateless evaluation.
///
/// 弹幕运行时状态的 C 兼容表示。
/// 传递给弹幕算法函数用于无状态计算。
#[repr(C)]
pub struct BulletStateC {
    /// Time since bullet spawn (seconds)
    pub elapsed: c_float,
    /// Spawn center X coordinate
    pub spawn_x: c_float,
    /// Spawn center Y coordinate
    pub spawn_y: c_float,
    /// Initial offset X (relative to spawn center)
    pub offset_x: c_float,
    /// Initial offset Y (relative to spawn center)
    pub offset_y: c_float,
    /// Initial angle (radians)
    pub initial_angle: c_float,
    /// Initial radius (for circular patterns)
    pub initial_radius: c_float,
    /// Current velocity direction X
    pub dir_x: c_float,
    /// Current velocity direction Y
    pub dir_y: c_float,
    /// Pointer to custom parameters array (from RON config)
    pub params: *const c_float,
    /// Length of parameters array
    pub params_len: usize,
}

impl Default for BulletStateC {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            spawn_x: 0.0,
            spawn_y: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            initial_angle: 0.0,
            initial_radius: 0.0,
            dir_x: 0.0,
            dir_y: -1.0,
            params: std::ptr::null(),
            params_len: 0,
        }
    }
}

/// Danmaku algorithm function signature.
/// Input: readonly bullet state
/// Output: position offset for this frame
///
/// 弹幕算法函数签名。
/// 输入：只读弹幕状态
/// 输出：本帧的位置偏移
pub type DanmakuUpdateFn = extern "C" fn(state: *const BulletStateC) -> Vec2C;

/// Function type to get the number of exported danmaku algorithms in the DLL.
pub type GetAlgorithmCountFn = extern "C" fn() -> u32;

/// Function type to get the ID of an algorithm by index.
/// Returns null if index is out of bounds.
pub type GetAlgorithmIdFn = extern "C" fn(index: u32) -> *const u8;

/// Function type to retrieve a danmaku algorithm by ID.
/// Returns None if the ID is not found.
pub type GetAlgorithmFn = unsafe extern "C" fn(id: *const u8) -> Option<DanmakuUpdateFn>;
