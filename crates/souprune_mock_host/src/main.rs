//! A standalone test host application that simulates the game engine.
//! It demonstrates how to load a mod DLL dynamically and invoke its functions,
//! useful for testing ABI compatibility without running the full game.
//!
//! 一个独立的测试宿主应用程序，模拟游戏引擎。
//! 演示如何动态加载模组 DLL 并调用其函数，
//! 用于在不运行完整游戏的情况下测试 ABI 兼容性。

use libloading::{Library, Symbol};
use souprune_api::{
    Action, ContextHandle, CreateSoulModeFn, GetSoulModeCountFn, GetSoulModeIdFn, HostApi,
};
use std::ffi::{CStr, CString, c_float};

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

        // B. 验证 Mod 列表
        let get_count: Symbol<GetSoulModeCountFn> = lib
            .get(b"get_soul_mode_count")
            .expect("Function get_soul_mode_count not found");
        let count = get_count();
        println!("[HOST] Found {} Soul Modes.", count);

        let get_id: Symbol<GetSoulModeIdFn> = lib
            .get(b"get_soul_mode_id")
            .expect("Function get_soul_mode_id not found");

        for i in 0..count {
            let id_ptr = get_id(i);
            let id_str = CStr::from_ptr(id_ptr as *const i8)
                .to_str()
                .expect("Invalid UTF-8 ID");
            println!("[HOST] Soul Mode #{}: '{}'", i, id_str);
        }

        // C. 查找入口函数
        let create_func: Symbol<CreateSoulModeFn> =
            lib.get(b"create_soul_mode").expect("Symbol not found");

        // D. 测试 "test_soul"
        println!("\n--- Testing 'test_soul' ---");
        let id_test = CString::new("test_soul").unwrap();
        let vtable_test = create_func(id_test.as_ptr() as *const u8, &api);

        if let Some(on_enter) = vtable_test.on_enter {
            let mut dummy = std::mem::zeroed();
            on_enter(&mut dummy);
        } else {
            println!("[HOST] 'test_soul' on_enter is missing!");
        }

        if let Some(on_update) = vtable_test.on_update {
            let mut dummy = std::mem::zeroed();
            on_update(&mut dummy, 0.016);
        }

        // E. 测试 "second_soul"
        println!("\n--- Testing 'second_soul' ---");
        let id_second = CString::new("second_soul").unwrap();
        let vtable_second = create_func(id_second.as_ptr() as *const u8, &api);

        if let Some(on_enter) = vtable_second.on_enter {
            let mut dummy = std::mem::zeroed();
            on_enter(&mut dummy);
        } else {
            println!("[HOST] 'second_soul' on_enter is missing!");
        }

        println!("--- End Simulation ---");
    }
}
