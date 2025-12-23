use souprune_api::{ContextHandle, HostApi};
use std::ffi::CString;

/// Context is the only window for dialogue between Mod and Engine.
/// It does not store state, it is just a "lens" to access Host functions.
///
/// Context 是 Mod 与引擎对话的唯一窗口。
/// 它不存储状态，只是一个访问 Host 功能的“透镜”。
pub struct Context<'a> {
    pub(crate) handle: *mut ContextHandle, // 传回给 Host 用
    pub(crate) api: &'a HostApi,           // 调用 Host 函数用
}

impl<'a> Context<'a> {
    /// For SDK internal use only: construct a safe context.
    /// Mod authors do not need to call this.
    ///
    /// 仅限 SDK 内部使用：构建一个安全上下文
    /// Mod 作者不需要调用这个
    pub unsafe fn new(handle: *mut ContextHandle, api: &'a HostApi) -> Self {
        Self { handle, api }
    }

    // === Below are functional modules exposed to Mod (下面是暴露给 Mod 的功能模块) ===

    /// Access kinematics control (e.g., movement).
    ///
    /// 访问运动学控制（如：移动）。
    pub fn kinematics(&mut self) -> KinematicsHelper<'_, 'a> {
        KinematicsHelper { context: self }
    }

    /// Access input state.
    ///
    /// 访问输入状态。
    pub fn input(&mut self) -> InputHelper<'_, 'a> {
        InputHelper { context: self }
    }

    /// Print log message to the engine console/log file.
    ///
    /// 将日志消息打印到引擎控制台/日志文件。
    pub fn log(&self, msg: &str) {
        let c_str = CString::new(msg).unwrap_or_default();
        // 自动转换 Rust String -> C Pointer
        (self.api.log)(0, c_str.as_ptr() as *const u8, c_str.as_bytes().len());
    }
}

// === Modular Encapsulation (模块化封装) ===

/// Kinematics module wrapper
///
/// 运动学模块封装
pub struct KinematicsHelper<'b, 'a> {
    context: &'b mut Context<'a>,
}

impl<'b, 'a> KinematicsHelper<'b, 'a> {
    /// Set the entity's velocity directly.
    ///
    /// 直接设置实体的速度。
    pub fn set_velocity(&self, x: f32, y: f32) {
        // 在这里把 Safe Rust 转换成 FFI 调用
        (self.context.api.kinematics_set_velocity)(self.context.handle, x, y);
    }
}

/// Input module wrapper
///
/// 输入模块封装
pub struct InputHelper<'b, 'a> {
    context: &'b mut Context<'a>,
}

impl<'b, 'a> InputHelper<'b, 'a> {
    /// Check if a semantic action is currently pressed.
    ///
    /// 检查当前是否按下了某个语义动作。
    pub fn pressed(&self, action: souprune_api::Action) -> bool {
        (self.context.api.input_is_action_pressed)(self.context.handle, action)
    }
}
