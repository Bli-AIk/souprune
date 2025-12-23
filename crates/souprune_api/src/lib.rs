//! Defines the fundamental C ABI for the modding system.
//! It contains raw C-compatible structs, enums, and function pointer definitions
//! that act as the contract between the Host (Game Engine) and the Guest (Mod).
//!
//! 定义了模组系统的基础 C ABI。
//! 包含原始的 C 兼容结构体、枚举和函数指针定义，
//! 作为宿主（游戏引擎）和客体（模组）之间的契约。

use core::ffi::c_float;

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
pub struct SoulModeVTable {
    pub on_enter: Option<extern "C" fn(context: *mut ContextHandle)>,

    pub on_update: Option<extern "C" fn(context: *mut ContextHandle, delta_time: c_float)>,

    pub on_exit: Option<extern "C" fn(context: *mut ContextHandle)>,
}

/// Function type to get the number of exported soul modes in the DLL.
pub type GetSoulModeCountFn = extern "C" fn() -> u32;

/// Function type to get the ID of a soul mode by index.
/// Returns null if index is out of bounds.
pub type GetSoulModeIdFn = extern "C" fn(index: u32) -> *const u8;

/// Handshake protocol: This is the type signature of the function exported by the DLL to create a soul mode instance.
/// The engine will look for a symbol named "create_soul_mode".
///
/// 握手协议：这是 DLL 导出的用于创建 soul mode 实例的函数类型签名
/// 引擎会查找名为 "create_soul_mode" 的符号
pub type CreateSoulModeFn =
    unsafe extern "C" fn(id: *const u8, api: *const HostApi) -> SoulModeVTable;
