//! Defines the fundamental C ABI for the modding system.
//! It contains raw C-compatible structs, enums, and function pointer definitions
//! that act as the contract between the Host (Game Engine) and the Guest (Mod).
//!
//! 定义了模组系统的基础 C ABI。
//! 包含原始的 C 兼容结构体、枚举和函数指针定义，
//! 作为宿主（游戏引擎）和客体（模组）之间的契约。

#[cfg(feature = "bindgen")]
pub mod bindgen_inventory;

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

/// Host API function table for behavior mods.
/// Contains function pointers for logging, input, and kinematics.
///
/// 行为模组的宿主 API 函数表。
/// 包含日志、输入和运动学的函数指针。
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

/// Named property for danmaku behaviors.
///
/// 弹幕行为的命名属性。
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PropC {
    pub name: *const u8,
    pub name_len: usize,
    pub value: c_float,
}

/// Bullet context for C-ABI.
/// This is passed to danmaku behavior functions.
/// Contains both immutable spawn data and current frame data.
///
/// 弹幕上下文的 C 兼容表示。
/// 传递给弹幕行为函数。
/// 包含不可变的生成数据和当前帧数据。
#[repr(C)]
pub struct BulletContextC {
    /// Time since bullet spawn (seconds)
    pub elapsed: c_float,
    /// Delta time for this frame
    pub delta_time: c_float,
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
    /// Player position X (for aimed behaviors)
    pub player_x: c_float,
    /// Player position Y (for aimed behaviors)
    pub player_y: c_float,
    /// Pointer to custom properties array (from RON config)
    pub props: *const PropC,
    /// Length of properties array
    pub props_len: usize,
    /// Legacy: Pointer to custom parameters array
    pub params: *const c_float,
    /// Legacy: Length of parameters array
    pub params_len: usize,
}

impl Default for BulletContextC {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            delta_time: 0.0,
            spawn_x: 0.0,
            spawn_y: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            initial_angle: 0.0,
            initial_radius: 0.0,
            player_x: 0.0,
            player_y: 0.0,
            props: std::ptr::null(),
            props_len: 0,
            params: std::ptr::null(),
            params_len: 0,
        }
    }
}

/// Bullet output for C-ABI.
/// Contains the computed position offset and other visual properties.
///
/// 弹幕输出的 C 兼容表示。
/// 包含计算出的位置偏移和其他视觉属性。
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BulletOutputC {
    /// Position offset X
    pub offset_x: c_float,
    /// Position offset Y
    pub offset_y: c_float,
    /// Rotation delta (radians)
    pub rotation: c_float,
}

impl BulletOutputC {
    pub const ZERO: Self = Self {
        offset_x: 0.0,
        offset_y: 0.0,
        rotation: 0.0,
    };

    pub fn new(x: c_float, y: c_float) -> Self {
        Self {
            offset_x: x,
            offset_y: y,
            rotation: 0.0,
        }
    }

    pub fn with_rotation(mut self, rotation: c_float) -> Self {
        self.rotation = rotation;
        self
    }
}

/// VTable for danmaku behaviors (similar to BehaviorVTable).
/// Allows stateful danmaku algorithms with on_enter/on_update/on_exit lifecycle.
///
/// 弹幕行为的 VTable（类似于 BehaviorVTable）。
/// 允许有状态的弹幕算法，具有 on_enter/on_update/on_exit 生命周期。
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DanmakuVTable {
    /// Called once when the bullet is spawned.
    /// Use this to capture initial state (e.g., player position for aimed behaviors).
    pub on_enter: Option<extern "C" fn(instance: *mut c_void, context: *const BulletContextC)>,

    /// Called every frame to compute bullet movement.
    /// Returns the position offset for this frame.
    pub on_update: Option<
        extern "C" fn(instance: *mut c_void, context: *const BulletContextC) -> BulletOutputC,
    >,

    /// Called when the bullet is despawned.
    pub on_exit: Option<extern "C" fn(instance: *mut c_void)>,

    /// Called to destroy the instance and free memory.
    pub destroy: Option<extern "C" fn(instance: *mut c_void)>,
}

/// Instance of a danmaku behavior.
#[repr(C)]
pub struct DanmakuInstance {
    pub instance: *mut c_void,
    pub vtable: DanmakuVTable,
}

/// Function type to get the number of exported danmaku algorithms in the DLL.
pub type GetAlgorithmCountFn = extern "C" fn() -> u32;

/// Function type to get the ID of an algorithm by index.
/// Returns null if index is out of bounds.
pub type GetAlgorithmIdFn = extern "C" fn(index: u32) -> *const u8;

/// Function type to create a danmaku behavior instance by ID.
pub type CreateDanmakuFn = unsafe extern "C" fn(id: *const u8) -> DanmakuInstance;

// Legacy API - kept for compatibility but deprecated
// 旧版 API - 保留兼容性但已弃用

/// Bullet runtime state for C-ABI (Legacy).
#[repr(C)]
#[deprecated(note = "Use BulletContextC and DanmakuVTable instead")]
pub struct BulletStateC {
    pub elapsed: c_float,
    pub spawn_x: c_float,
    pub spawn_y: c_float,
    pub offset_x: c_float,
    pub offset_y: c_float,
    pub initial_angle: c_float,
    pub initial_radius: c_float,
    pub dir_x: c_float,
    pub dir_y: c_float,
    pub player_x: c_float,
    pub player_y: c_float,
    pub params: *const c_float,
    pub params_len: usize,
}

#[allow(deprecated)]
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
            player_x: 0.0,
            player_y: 0.0,
            params: std::ptr::null(),
            params_len: 0,
        }
    }
}

/// Legacy danmaku function signature (deprecated).
#[deprecated(note = "Use DanmakuVTable instead")]
#[allow(deprecated)]
pub type DanmakuUpdateFn = extern "C" fn(state: *const BulletStateC) -> Vec2C;

/// Legacy function type (deprecated).
#[deprecated(note = "Use CreateDanmakuFn instead")]
#[allow(deprecated)]
pub type GetAlgorithmFn = unsafe extern "C" fn(id: *const u8) -> Option<DanmakuUpdateFn>;
