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
pub use context::Context;
pub use souprune_api::{
    Action, BehaviorInstance, BehaviorVTable, ContextHandle, HostApi, c_float, c_void,
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
