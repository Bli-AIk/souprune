use souprune_api::{ContextHandle, HostApi};
use std::ffi::CString;

/// Context 是 Mod 与引擎对话的唯一窗口。
/// 它不存储状态，只是一个访问 Host 功能的“透镜”。
pub struct Context<'a> {
    pub(crate) handle: *mut ContextHandle, // 传回给 Host 用
    pub(crate) api: &'a HostApi,           // 调用 Host 函数用
}

impl<'a> Context<'a> {
    /// 仅限 SDK 内部使用：构建一个安全上下文
    /// Mod 作者不需要调用这个
    pub unsafe fn new(handle: *mut ContextHandle, api: &'a HostApi) -> Self {
        Self { handle, api }
    }

    // === 下面是暴露给 Mod 的功能模块 ===

    pub fn physics(&mut self) -> PhysicsHelper<'_, 'a> {
        PhysicsHelper { context: self }
    }

    pub fn input(&mut self) -> InputHelper<'_, 'a> {
        InputHelper { context: self }
    }

    pub fn log(&self, msg: &str) {
        let c_str = CString::new(msg).unwrap_or_default();
        // 自动转换 Rust String -> C Pointer
        (self.api.log)(0, c_str.as_ptr() as *const u8, c_str.as_bytes().len());
    }
}

// === 模块化封装 ===

// 物理模块封装
pub struct PhysicsHelper<'b, 'a> {
    context: &'b mut Context<'a>,
}

impl<'b, 'a> PhysicsHelper<'b, 'a> {
    pub fn set_velocity(&self, x: f32, y: f32) {
        // 在这里把 Safe Rust 转换成 FFI 调用
        (self.context.api.physics_set_velocity)(self.context.handle, x, y);
    }
}

// 输入模块封装
pub struct InputHelper<'b, 'a> {
    context: &'b mut Context<'a>,
}

impl<'b, 'a> InputHelper<'b, 'a> {
    pub fn axis(&self, name: &str) -> f32 {
        let c_name = CString::new(name).unwrap_or_default();
        (self.context.api.input_get_axis)(self.context.handle, c_name.as_ptr() as *const u8)
    }
}
