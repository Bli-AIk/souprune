use libloading::{Library, Symbol};
use souprune_api::{ContextHandle, CreateSoulModeFn, HostApi};
use std::ffi::{CStr, c_char, c_float};

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
extern "C" fn host_input_get_axis(_ctx: *const ContextHandle, name: *const u8) -> c_float {
    unsafe {
        let c_str = CStr::from_ptr(name as *const c_char);
        let s = c_str.to_string_lossy();
        if s == "Horizontal" {
            return 1.0; // 模拟按键按下
        }
        0.0
    }
}

// 模拟的物理函数
extern "C" fn host_physics_set_vel(_ctx: *mut ContextHandle, x: c_float, y: c_float) {
    println!("[HOST] Physics command: Set Velocity to ({}, {})", x, y);
}

// === 2. 主流程 ===

fn main() {
    // 构建 API 表 (VTable)
    let api = HostApi {
        log: host_log,
        input_get_axis: host_input_get_axis,
        physics_set_velocity: host_physics_set_vel,
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

        // B. 查找入口函数
        let func: Symbol<CreateSoulModeFn> =
            lib.get(b"create_soul_mode").expect("Symbol not found");

        // C. 握手：传入 HostApi，获取 ModVTable
        let vtable = func(&api);

        // D. 模拟游戏循环
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
