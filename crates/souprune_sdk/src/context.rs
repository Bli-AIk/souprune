//! Provides a safe wrapper around the raw `ContextHandle` and `HostApi`.
//! It allows mod developers to interact with the game world (e.g., getting input, setting velocity)
//! using idiomatic Rust methods.
//!
//! 提供对原始 `ContextHandle` 和 `HostApi` 的安全封装。
//! 允许模组开发者使用惯用的 Rust 方法与游戏世界交互（例如获取输入、设置速度）。

use souprune_api::{BulletContextC, ContextHandle, HostApi};
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
// Bullet Context Safe Wrapper (弹幕上下文安全封装)
// ============================================================================

/// Safe wrapper around BulletContextC for danmaku algorithms.
/// Provides safe access to bullet context without requiring unsafe code in user scripts.
///
/// BulletContextC 的安全封装，用于弹幕算法。
/// 提供对弹幕上下文的安全访问，用户脚本无需使用 unsafe 代码。
#[derive(Debug, Clone)]
pub struct BulletContext {
    /// Time since bullet spawn (seconds)
    pub elapsed: f32,
    /// Delta time for this frame
    pub delta_time: f32,
    /// Spawn center position
    pub spawn_pos: Vec2,
    /// Initial offset from spawn center
    pub initial_offset: Vec2,
    /// Initial angle (radians)
    pub initial_angle: f32,
    /// Initial radius (for circular patterns)
    pub initial_radius: f32,
    /// Current player position (for aimed behaviors)
    pub player_pos: Vec2,
    /// Custom properties from RON config (named)
    props: Vec<(String, f32)>,
    /// Legacy custom parameters from RON config (indexed)
    params: [f32; 16],
    params_len: usize,
}

/// Parse a C string pointer and length into an owned `String`.
///
/// # Safety
///
/// `name` must be a valid pointer to `name_len` bytes, or null.
unsafe fn parse_c_str(name: *const u8, name_len: usize) -> String {
    if !name.is_null() && name_len > 0 {
        // SAFETY: caller guarantees valid pointer and length
        let slice = unsafe { std::slice::from_raw_parts(name, name_len) };
        String::from_utf8_lossy(slice).into_owned()
    } else {
        String::new()
    }
}

impl BulletContext {
    /// Create a BulletContext from raw C pointer.
    /// This is called internally by the SDK macro.
    ///
    /// 从原始 C 指针创建 BulletContext。
    /// 由 SDK 宏内部调用。
    ///
    /// # Safety
    ///
    /// `ctx` must be a valid pointer to BulletContextC provided by the host.
    pub unsafe fn from_raw(ctx: *const BulletContextC) -> Self {
        // SAFETY: Caller guarantees ctx is valid
        let c = unsafe { &*ctx };

        // Parse named properties
        let mut props = Vec::with_capacity(c.props_len);
        if !c.props.is_null() && c.props_len > 0 {
            for i in 0..c.props_len {
                // SAFETY: We've checked props is not null and i < props_len
                let prop_c = unsafe { &*c.props.add(i) };
                // SAFETY: parse_c_str handles null/zero-length checks
                let name = unsafe { parse_c_str(prop_c.name, prop_c.name_len) };
                props.push((name, prop_c.value));
            }
        }

        // Safely copy legacy parameters into fixed-size array
        let mut params = [0.0f32; 16];
        let params_len = c.params_len.min(16);
        if !c.params.is_null() && params_len > 0 {
            #[expect(clippy::needless_range_loop)] // reason: index needed for unsafe pointer arithmetic
            for i in 0..params_len {
                // SAFETY: We've checked params is not null and i < params_len
                params[i] = unsafe { *c.params.add(i) };
            }
        }

        Self {
            elapsed: c.elapsed,
            delta_time: c.delta_time,
            spawn_pos: Vec2 {
                x: c.spawn_x,
                y: c.spawn_y,
            },
            initial_offset: Vec2 {
                x: c.offset_x,
                y: c.offset_y,
            },
            initial_angle: c.initial_angle,
            initial_radius: c.initial_radius,
            player_pos: Vec2 {
                x: c.player_x,
                y: c.player_y,
            },
            props,
            params,
            params_len,
        }
    }

    /// Get a named property value, or None if not found.
    ///
    /// 获取指定名称的属性值，若未找到则返回 None。
    pub fn get_float(&self, name: &str) -> Option<f32> {
        self.props.iter().find(|(n, _)| n == name).map(|(_, v)| *v)
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
    pub fn spawn_position(&self) -> Vec2 {
        Vec2 {
            x: self.spawn_pos.x + self.initial_offset.x,
            y: self.spawn_pos.y + self.initial_offset.y,
        }
    }
}

// ============================================================================
// Bullet Output (弹幕输出)
// ============================================================================

/// Output from a danmaku behavior's on_update method.
///
/// 弹幕行为 on_update 方法的输出。
#[derive(Debug, Clone, Copy, Default)]
pub struct BulletOutput {
    /// Position offset
    pub offset: Vec2,
    /// Rotation delta (radians)
    pub rotation: f32,
}

impl BulletOutput {
    pub const ZERO: Self = Self {
        offset: Vec2::ZERO,
        rotation: 0.0,
    };

    pub fn new(x: f32, y: f32) -> Self {
        Self {
            offset: Vec2::new(x, y),
            rotation: 0.0,
        }
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Convert to C-compatible BulletOutputC for return value.
    pub fn to_output_c(self) -> souprune_api::BulletOutputC {
        souprune_api::BulletOutputC {
            offset_x: self.offset.x,
            offset_y: self.offset.y,
            rotation: self.rotation,
        }
    }
}

// ============================================================================
// Vec2 Helper (二维向量辅助)
// ============================================================================

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

    /// Subtract another vector
    pub fn sub(&self, other: &Vec2) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    /// Convert to C-compatible Vec2C for return value.
    ///
    /// 转换为 C 兼容的 Vec2C 用于返回值。
    pub fn to_vec2c(self) -> souprune_api::Vec2C {
        souprune_api::Vec2C {
            x: self.x,
            y: self.y,
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

impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}
