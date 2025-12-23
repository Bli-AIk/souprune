//! A standalone test host application that simulates the game engine.
//! It demonstrates how to load a mod DLL dynamically and invoke its functions,
//! useful for testing ABI compatibility without running the full game.
//!
//! 一个独立的测试宿主应用程序，模拟游戏引擎。
//! 演示如何动态加载模组 DLL 并调用其函数，
//! 用于在不运行完整游戏的情况下测试 ABI 兼容性。

use libloading::{Library, Symbol};
use souprune_api::{Action, ContextHandle, CreateSoulModeFn, HostApi};
use std::ffi::{c_float, CStr};

// === 1. 模拟宿主侧的实现 ===

// 模拟的日志函数
extern "C" fn host_log(_level: u32, msg: *const u8, len: usize) {
    unsafe {
        let slice = std::slice::from_raw_parts(msg, len);
        let s = String::from_utf8_lossy(slice);
        println!("[HOST] Log received: {}", s);
    }
}

// 模拟的输入函数 (假装玩家一直按着右键)
extern "C" fn host_input_is_action_pressed(_ctx: *const ContextHandle, action: Action) -> bool {
    println!("[HOST] Checking input for action: {:?}", action);
    // 模拟：只有 Right 键被按下
    action == Action::Right
}

// 模拟的运动学函数
extern "C" fn host_kinematics_set_vel(_ctx: *mut ContextHandle, x: c_float, y: c_float) {
    println!("[HOST] Kinematics command: Set Velocity to ({}, {})", x, y);
}

// === 2. 主流程 ===

fn main() {
    // 构建 API 表 (VTable)
    let api = HostApi {
        log: host_log,
        input_is_action_pressed: host_input_is_action_pressed,
        kinematics_set_velocity: host_kinematics_set_vel,
    };

    unsafe {
        // A. 加载 DLL (注意：路径取决于你的操作系统和编译模式)
        // Windows: target/debug/souprune_mod_test.dll
        // Linux:   target/debug/libsouprune_mod_test.so
        // macOS:   target/debug/libsouprune_mod_test.dylib
        let lib_path = if cfg!(target_os = "windows") {
            "target/debug/souprune_mod_test.dll"
        } else {
            "target/debug/libsouprune_mod_test.so"
        };

        println!("Loading mod from: {}", lib_path);
        let lib = Library::new(lib_path).expect("Failed to load DLL");

        // B. 验证 Mod ID
        let get_id_func: Symbol<extern "C" fn() -> *const i8> =
            lib.get(b"get_soul_mode_id").expect("Function get_soul_mode_id not found");

        let id_ptr = get_id_func();
        let id_cstr = CStr::from_ptr(id_ptr);
        let id_str = id_cstr.to_str().expect("Invalid UTF-8 ID");
        println!("[HOST] Found Mod ID: '{}'", id_str);

        if id_str != "test_soul" {
            println!("[HOST] ERROR: Expected mod ID 'test_soul', but got '{}'. Aborting.", id_str);
            return;
        }

        println!("[HOST] ID Verified. Initializing...");

        // C. 查找入口函数
        let func: Symbol<CreateSoulModeFn> =
            lib.get(b"create_soul_mode").expect("Symbol not found");

        // D. 握手：传入 HostApi，获取 ModVTable
        let vtable = func(&api);

        // E. 模拟游戏循环
        let mut dummy_context_handle = std::mem::zeroed(); // 模拟一个句柄

        println!("--- Start Simulation ---");

        // Call OnEnter
        if let Some(on_enter) = vtable.on_enter {
            on_enter(&mut dummy_context_handle);
        }

        // Call OnUpdate (模拟一帧)
        if let Some(on_update) = vtable.on_update {
            on_update(&mut dummy_context_handle, 0.016);
        }

        // Call OnExit
        if let Some(on_exit) = vtable.on_exit {
            on_exit(&mut dummy_context_handle);
        }

        println!("--- End Simulation ---");
    }
}
