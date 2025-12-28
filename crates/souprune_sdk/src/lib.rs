//! The main entry point for the Rust Modding SDK.
//! It re-exports necessary types and provides the `declare_behaviors!` macro
//! to simplify mod registration.
//!
//! Rust 模组开发 SDK 的主要入口点。
//! 重新导出必要的类型，并提供 `declare_behaviors!` 宏以简化模组注册流程。

pub mod context;
pub mod traits;

// Re-export for user convenience
// 重新导出，方便用户使用
pub use context::{BulletState, Context, Vec2};
pub use souprune_api::{
    Action, BehaviorInstance, BehaviorVTable, BulletStateC, ContextHandle, DanmakuUpdateFn,
    HostApi, Vec2C, c_float, c_void,
};
pub use traits::Behavior;

/// This macro is used to register user's Structs as Mods.
///
/// 这个宏用于把用户的 Struct 注册为 Mod
#[macro_export]
macro_rules! declare_behaviors {
    ( $( ($id:literal, $mod_type:ty, $constructor:expr) ),* $(,)? ) => {
        // Static variable to store the API table
        // 静态变量存储 API 表
        static mut HOST_API: Option<$crate::HostApi> = None;

        /// Export the number of Behaviors in this DLL
        #[unsafe(no_mangle)]
        pub extern "C" fn get_behavior_count() -> u32 {
            let mut count = 0;
            $(
                count += 1;
                let _ = $id; // Suppress unused warning
            )*
            count
        }

        /// Export the ID of a Behavior by index
        #[unsafe(no_mangle)]
        pub extern "C" fn get_behavior_id(index: u32) -> *const u8 {
            let ids = [ $( concat!($id, "\0").as_ptr() ),* ];
            if (index as usize) < ids.len() {
                ids[index as usize]
            } else {
                std::ptr::null()
            }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn create_behavior(
            id: *const u8,
            api: *const $crate::HostApi,
        ) -> $crate::BehaviorInstance {
            unsafe {
                HOST_API = Some(api.read());
            }

            let id_str = unsafe {
                std::ffi::CStr::from_ptr(id as *const i8)
                    .to_str()
                    .unwrap_or("")
            };

            $(
                if id_str == $id {
                    // Create instance on heap and leak it to Host
                    // 在堆上创建实例并泄漏给 Host
                    let instance = Box::new($constructor());
                    let instance_ptr = Box::into_raw(instance) as *mut core::ffi::c_void;

                    // Define adapter functions locally for this specific behavior
                    extern "C" fn wrapper_on_enter(instance: *mut core::ffi::c_void, handle: *mut $crate::ContextHandle) {
                        unsafe {
                            if let Some(api) = &HOST_API {
                                let mut context = $crate::Context::new(handle, api);
                                // Cast back to specific type
                                let mode = &mut *(instance as *mut $mod_type);
                                mode.on_enter(&mut context);
                            }
                        }
                    }

                    extern "C" fn wrapper_on_update(instance: *mut core::ffi::c_void, handle: *mut $crate::ContextHandle, dt: $crate::c_float) {
                        unsafe {
                            if let Some(api) = &HOST_API {
                                let mut context = $crate::Context::new(handle, api);
                                let mode = &mut *(instance as *mut $mod_type);
                                mode.on_update(&mut context, dt);
                            }
                        }
                    }

                    extern "C" fn wrapper_on_exit(instance: *mut core::ffi::c_void, handle: *mut $crate::ContextHandle) {
                        unsafe {
                            if let Some(api) = &HOST_API {
                                let mut context = $crate::Context::new(handle, api);
                                let mode = &mut *(instance as *mut $mod_type);
                                mode.on_exit(&mut context);
                            }
                        }
                    }

                    extern "C" fn wrapper_destroy(instance: *mut core::ffi::c_void) {
                         unsafe {
                            // Re-take ownership and drop
                            // 重新获取所有权并释放
                            let _ = Box::from_raw(instance as *mut $mod_type);
                        }
                    }

                    return $crate::BehaviorInstance {
                        instance: instance_ptr,
                        vtable: $crate::BehaviorVTable {
                            on_enter: Some(wrapper_on_enter),
                            on_update: Some(wrapper_on_update),
                            on_exit: Some(wrapper_on_exit),
                            destroy: Some(wrapper_destroy),
                        }
                    };
                }
            )*

            // Return empty Instance if ID not found
            $crate::BehaviorInstance {
                instance: std::ptr::null_mut(),
                vtable: $crate::BehaviorVTable {
                    on_enter: None,
                    on_update: None,
                    on_exit: None,
                    destroy: None,
                }
            }
        }
    };
}

/// This macro is used to register danmaku algorithms for the bullet pattern system.
/// Each algorithm should be an `extern "C"` function that wraps your safe algorithm.
/// Use `wrap_algorithm!` to create the wrapper from a safe function.
///
/// 这个宏用于注册弹幕算法到弹幕模式系统。
/// 每个算法应该是一个 `extern "C"` 函数，包装你的安全算法。
/// 使用 `wrap_algorithm!` 从安全函数创建包装器。
///
/// # Example
/// ```ignore
/// use souprune_sdk::{BulletState, Vec2, declare_algorithms, wrap_algorithm};
///
/// // Define your safe algorithm function
/// fn my_homing_algorithm(state: &BulletState) -> Vec2 {
///     let speed = state.param(0, 200.0);
///     Vec2::new(0.0, -speed * state.elapsed)
/// }
///
/// // Create the FFI wrapper
/// wrap_algorithm!(homing_spear_wrapper, my_homing_algorithm);
///
/// // Register the wrapper
/// declare_algorithms!(
///     ("homing_spear", homing_spear_wrapper),
/// );
/// ```
#[macro_export]
macro_rules! declare_algorithms {
    ( $( ($id:literal, $algo_fn:expr) ),* $(,)? ) => {
        /// Export the number of Danmaku Algorithms in this DLL
        #[unsafe(no_mangle)]
        pub extern "C" fn get_algorithm_count() -> u32 {
            let mut count = 0;
            $(
                count += 1;
                let _ = $id; // Suppress unused warning
            )*
            count
        }

        /// Export the ID of an Algorithm by index
        #[unsafe(no_mangle)]
        pub extern "C" fn get_algorithm_id(index: u32) -> *const u8 {
            let ids = [ $( concat!($id, "\0").as_ptr() ),* ];
            if (index as usize) < ids.len() {
                ids[index as usize]
            } else {
                std::ptr::null()
            }
        }

        /// Get an algorithm function by ID
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn get_algorithm(id: *const u8) -> Option<$crate::DanmakuUpdateFn> {
            let id_str = unsafe {
                std::ffi::CStr::from_ptr(id as *const i8)
                    .to_str()
                    .unwrap_or("")
            };

            $(
                if id_str == $id {
                    return Some($algo_fn);
                }
            )*

            None
        }
    };
}

/// Macro to wrap a safe algorithm function into an extern "C" FFI function.
/// The safe function should take `&BulletState` and return `Vec2`.
///
/// 将安全的算法函数包装成 `extern "C"` FFI 函数的宏。
/// 安全函数应接受 `&BulletState` 并返回 `Vec2`。
///
/// # Example
/// ```ignore
/// fn my_algorithm(state: &BulletState) -> Vec2 {
///     Vec2::new(state.elapsed * 100.0, 0.0)
/// }
///
/// wrap_algorithm!(my_algorithm_ffi, my_algorithm);
/// ```
#[macro_export]
macro_rules! wrap_algorithm {
    ($wrapper_name:ident, $safe_fn:expr) => {
        extern "C" fn $wrapper_name(state: *const $crate::BulletStateC) -> $crate::Vec2C {
            // SAFETY: The host guarantees that `state` is a valid pointer
            let safe_state = unsafe { $crate::BulletState::from_raw(state) };
            let result = $safe_fn(&safe_state);
            result.to_vec2c()
        }
    };
}
