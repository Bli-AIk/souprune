//! Provides a safe wrapper around the raw `ContextHandle` and `HostApi`.
//! It allows mod developers to interact with the game world (e.g., getting input, setting velocity)
//! using idiomatic Rust methods.
//!
//! 提供对原始 `ContextHandle` 和 `HostApi` 的安全封装。
//! 允许模组开发者使用惯用的 Rust 方法与游戏世界交互（例如获取输入、设置速度）。

use souprune_api::{BulletStateC, ContextHandle, HostApi, Vec2C};
use std::ffi::CString;

/// Context is the only window for dialogue between Mod and Engine.
/// It does not store state, it is just a "lens" to access Host functions.
///
/// Context 是 Mod 与引擎对话的唯一窗口。
/// 它不存储状态，只是一个访问 Host 功能的"透镜"。
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
    ///
    /// # Safety
    ///
    /// `handle` and `api` must be valid pointers provided by the host.
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

// ============================================================================
// Bullet State Safe Wrapper (弹幕状态安全封装)
// ============================================================================

/// Safe wrapper around BulletStateC for danmaku algorithms.
/// Provides safe access to bullet state without requiring unsafe code in user scripts.
///
/// BulletStateC 的安全封装，用于弹幕算法。
/// 提供对弹幕状态的安全访问，用户脚本无需使用 unsafe 代码。
#[derive(Debug, Clone, Copy)]
pub struct BulletState {
    /// Time since bullet spawn (seconds)
    pub elapsed: f32,
    /// Spawn center position
    pub spawn_pos: Vec2,
    /// Initial offset from spawn center
    pub initial_offset: Vec2,
    /// Initial angle (radians)
    pub initial_angle: f32,
    /// Initial radius (for circular patterns)
    pub initial_radius: f32,
    /// Current velocity direction (normalized)
    pub direction: Vec2,
    /// Custom parameters from RON config
    params: [f32; 16], // Fixed size array to avoid heap allocation
    params_len: usize,
}

impl BulletState {
    /// Create a BulletState from raw C pointer.
    /// This is called internally by the SDK macro.
    ///
    /// 从原始 C 指针创建 BulletState。
    /// 由 SDK 宏内部调用。
    ///
    /// # Safety
    ///
    /// `state` must be a valid pointer to BulletStateC provided by the host.
    pub unsafe fn from_raw(state: *const BulletStateC) -> Self {
        // SAFETY: Caller guarantees state is valid
        let s = unsafe { &*state };

        // Safely copy parameters into fixed-size array
        let mut params = [0.0f32; 16];
        let params_len = s.params_len.min(16);
        if !s.params.is_null() && params_len > 0 {
            for i in 0..params_len {
                // SAFETY: We've checked params is not null and i < params_len
                params[i] = unsafe { *s.params.add(i) };
            }
        }

        Self {
            elapsed: s.elapsed,
            spawn_pos: Vec2 {
                x: s.spawn_x,
                y: s.spawn_y,
            },
            initial_offset: Vec2 {
                x: s.offset_x,
                y: s.offset_y,
            },
            initial_angle: s.initial_angle,
            initial_radius: s.initial_radius,
            direction: Vec2 {
                x: s.dir_x,
                y: s.dir_y,
            },
            params,
            params_len,
        }
    }

    /// Get parameter at index, or default value if out of bounds.
    ///
    /// 获取指定索引的参数，若越界则返回默认值。
    pub fn param(&self, index: usize, default: f32) -> f32 {
        if index < self.params_len {
            self.params[index]
        } else {
            default
        }
    }

    /// Get current position (spawn + offset).
    ///
    /// 获取当前位置（生成点 + 偏移）。
    pub fn current_pos(&self) -> Vec2 {
        Vec2 {
            x: self.spawn_pos.x + self.initial_offset.x,
            y: self.spawn_pos.y + self.initial_offset.y,
        }
    }
}

/// Simple 2D vector for SDK use.
///
/// SDK 使用的简单二维向量。
#[derive(Debug, Clone, Copy, Default)]
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

    /// Convert to C-compatible Vec2C for return value.
    ///
    /// 转换为 C 兼容的 Vec2C 用于返回值。
    pub fn to_vec2c(self) -> Vec2C {
        Vec2C {
            x: self.x,
            y: self.y,
        }
    }
}
