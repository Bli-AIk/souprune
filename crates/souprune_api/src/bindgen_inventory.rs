//! Inventory of all FFI types and functions for code generation.
//! Uses interoptopus to generate C and C# bindings.
//!
//! FFI 类型和函数的清单，用于代码生成。
//! 使用 interoptopus 生成 C 和 C# 绑定。

use interoptopus::inventory::Inventory;
use interoptopus::{ffi_function, ffi_type, function};

// ============================================================================
// Mirror Types for Interoptopus (interoptopus 镜像类型)
// ============================================================================
// These types mirror the ones in lib.rs but with interoptopus attributes.
// We use these only for code generation, not at runtime.
// 这些类型是 lib.rs 中类型的镜像，但带有 interoptopus 属性。
// 我们只在代码生成时使用这些类型，不在运行时使用。

/// C-compatible 2D vector for FFI.
#[ffi_type]
#[derive(Debug, Clone, Copy, Default)]
pub struct Vec2C {
    pub x: f32,
    pub y: f32,
}

/// Bullet output for C-ABI.
#[ffi_type]
#[derive(Debug, Clone, Copy, Default)]
pub struct BulletOutputC {
    pub offset_x: f32,
    pub offset_y: f32,
    pub rotation: f32,
}

/// Bullet context for C-ABI (simplified for bindings).
#[ffi_type]
#[derive(Debug, Clone, Copy, Default)]
pub struct BulletContextC {
    pub elapsed: f32,
    pub delta_time: f32,
    pub spawn_x: f32,
    pub spawn_y: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub initial_angle: f32,
    pub initial_radius: f32,
    pub player_x: f32,
    pub player_y: f32,
}

/// Action enum for input handling.
#[ffi_type]
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

// ============================================================================
// Helper Functions for C# / Other Languages
// ============================================================================

/// Create a new BulletOutputC with position offset.
#[ffi_function]
#[unsafe(no_mangle)]
pub extern "C" fn danmaku_output_new(offset_x: f32, offset_y: f32) -> BulletOutputC {
    BulletOutputC {
        offset_x,
        offset_y,
        rotation: 0.0,
    }
}

/// Create a BulletOutputC with rotation.
#[ffi_function]
#[unsafe(no_mangle)]
pub extern "C" fn danmaku_output_with_rotation(
    offset_x: f32,
    offset_y: f32,
    rotation: f32,
) -> BulletOutputC {
    BulletOutputC {
        offset_x,
        offset_y,
        rotation,
    }
}

/// Create a new Vec2C.
#[ffi_function]
#[unsafe(no_mangle)]
pub extern "C" fn vec2c_new(x: f32, y: f32) -> Vec2C {
    Vec2C { x, y }
}

/// Get length of Vec2C.
#[ffi_function]
#[unsafe(no_mangle)]
pub extern "C" fn vec2c_length(v: Vec2C) -> f32 {
    (v.x * v.x + v.y * v.y).sqrt()
}

/// Normalize a Vec2C.
#[ffi_function]
#[unsafe(no_mangle)]
pub extern "C" fn vec2c_normalize(v: Vec2C) -> Vec2C {
    let len = (v.x * v.x + v.y * v.y).sqrt();
    if len > 0.0001 {
        Vec2C {
            x: v.x / len,
            y: v.y / len,
        }
    } else {
        Vec2C { x: 0.0, y: 0.0 }
    }
}

/// Get elapsed time from bullet context.
#[ffi_function]
#[unsafe(no_mangle)]
pub extern "C" fn bullet_context_get_elapsed(ctx: &BulletContextC) -> f32 {
    ctx.elapsed
}

/// Get delta time from bullet context.
#[ffi_function]
#[unsafe(no_mangle)]
pub extern "C" fn bullet_context_get_delta_time(ctx: &BulletContextC) -> f32 {
    ctx.delta_time
}

/// Get spawn position from bullet context.
#[ffi_function]
#[unsafe(no_mangle)]
pub extern "C" fn bullet_context_get_spawn_pos(ctx: &BulletContextC) -> Vec2C {
    Vec2C {
        x: ctx.spawn_x,
        y: ctx.spawn_y,
    }
}

/// Get initial offset from bullet context.
#[ffi_function]
#[unsafe(no_mangle)]
pub extern "C" fn bullet_context_get_offset(ctx: &BulletContextC) -> Vec2C {
    Vec2C {
        x: ctx.offset_x,
        y: ctx.offset_y,
    }
}

/// Get initial angle from bullet context.
#[ffi_function]
#[unsafe(no_mangle)]
pub extern "C" fn bullet_context_get_initial_angle(ctx: &BulletContextC) -> f32 {
    ctx.initial_angle
}

/// Get initial radius from bullet context.
#[ffi_function]
#[unsafe(no_mangle)]
pub extern "C" fn bullet_context_get_initial_radius(ctx: &BulletContextC) -> f32 {
    ctx.initial_radius
}

/// Get player position from bullet context.
#[ffi_function]
#[unsafe(no_mangle)]
pub extern "C" fn bullet_context_get_player_pos(ctx: &BulletContextC) -> Vec2C {
    Vec2C {
        x: ctx.player_x,
        y: ctx.player_y,
    }
}

// ============================================================================
// Inventory Builder
// ============================================================================

/// Builds the inventory of all FFI types for code generation.
pub fn build_inventory() -> Inventory {
    Inventory::builder()
        // Register helper functions
        .register(function!(danmaku_output_new))
        .register(function!(danmaku_output_with_rotation))
        .register(function!(vec2c_new))
        .register(function!(vec2c_length))
        .register(function!(vec2c_normalize))
        .register(function!(bullet_context_get_elapsed))
        .register(function!(bullet_context_get_delta_time))
        .register(function!(bullet_context_get_spawn_pos))
        .register(function!(bullet_context_get_offset))
        .register(function!(bullet_context_get_initial_angle))
        .register(function!(bullet_context_get_initial_radius))
        .register(function!(bullet_context_get_player_pos))
        .validate()
        .build()
}
