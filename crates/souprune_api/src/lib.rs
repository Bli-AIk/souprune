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

#[repr(C)]
pub struct HostApi {
    pub log: extern "C" fn(level: u32, msg: *const u8, len: usize),

    // === 传感器 (Sensors) ===
    pub input_get_axis:
        extern "C" fn(context: *const ContextHandle, name_ptr: *const u8) -> c_float,

    // === 执行器 (Actuators) ===
    pub physics_set_velocity: extern "C" fn(context: *mut ContextHandle, x: c_float, y: c_float),
}

#[repr(C)]
pub struct SoulModeVTable {
    pub on_enter: Option<extern "C" fn(context: *mut ContextHandle)>,

    pub on_update: Option<extern "C" fn(context: *mut ContextHandle, delta_time: c_float)>,

    pub on_exit: Option<extern "C" fn(context: *mut ContextHandle)>,
}

/// 握手协议：这是 DLL 导出的唯一函数的类型签名
/// 引擎会查找名为 "create_soul_mode" 的符号，并强制转为此类型调用
pub type CreateSoulModeFn = extern "C" fn(api: *const HostApi) -> SoulModeVTable;
