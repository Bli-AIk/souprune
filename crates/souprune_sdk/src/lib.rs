//! The main entry point for the Rust Modding SDK.
//! It re-exports necessary types and provides the `declare_soul_mode!` macro
//! to simplify mod registration.
//!
//! Rust 模组开发 SDK 的主要入口点。
//! 重新导出必要的类型，并提供 `declare_soul_mode!` 宏以简化模组注册流程。

pub mod context;
pub mod traits;

// Re-export for user convenience
// 重新导出，方便用户使用
pub use context::Context;
pub use traits::SoulMode;
// Also export HostApi from soup_api for macro use
// 也要把 soup_api 的 HostApi 导出来给宏用
pub use souprune_api::{Action, ContextHandle, HostApi, SoulModeVTable};

/// This macro is used to register user's Structs as Mods.
///
/// 这个宏用于把用户的 Struct 注册为 Mod
#[macro_export]
macro_rules! declare_souls {
    ( $( ($id:literal, $mod_type:ty, $constructor:expr) ),* $(,)? ) => {
        // Static variable to store the API table
        // 静态变量存储 API 表
        static mut HOST_API: Option<$crate::HostApi> = None;

        /// Export the number of Soul Modes in this DLL
        #[unsafe(no_mangle)]
        pub extern "C" fn get_soul_mode_count() -> u32 {
            let mut count = 0;
            $(
                count += 1;
                let _ = $id; // Suppress unused warning
            )*
            count
        }

        /// Export the ID of a Soul Mode by index
        #[unsafe(no_mangle)]
        pub extern "C" fn get_soul_mode_id(index: u32) -> *const u8 {
            let ids = [ $( concat!($id, "\0").as_ptr() ),* ];
            if (index as usize) < ids.len() {
                ids[index as usize]
            } else {
                std::ptr::null()
            }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn create_soul_mode(
            id: *const u8,
            api: *const $crate::HostApi,
        ) -> $crate::SoulModeVTable {
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
                    // Define adapter functions locally for this specific soul mode
                    extern "C" fn wrapper_on_enter(handle: *mut $crate::ContextHandle) {
                        unsafe {
                            if let Some(api) = &HOST_API {
                                let mut context = $crate::Context::new(handle, api);
                                let mut mode: $mod_type = $constructor();
                                mode.on_enter(&mut context);
                            }
                        }
                    }

                    extern "C" fn wrapper_on_update(handle: *mut $crate::ContextHandle, dt: f32) {
                        unsafe {
                            if let Some(api) = &HOST_API {
                                let mut context = $crate::Context::new(handle, api);
                                let mut mode: $mod_type = $constructor();
                                mode.on_update(&mut context, dt);
                            }
                        }
                    }

                    extern "C" fn wrapper_on_exit(handle: *mut $crate::ContextHandle) {
                        unsafe {
                            if let Some(api) = &HOST_API {
                                let mut context = $crate::Context::new(handle, api);
                                let mut mode: $mod_type = $constructor();
                                mode.on_exit(&mut context);
                            }
                        }
                    }

                    return $crate::SoulModeVTable {
                        on_enter: Some(wrapper_on_enter),
                        on_update: Some(wrapper_on_update),
                        on_exit: Some(wrapper_on_exit),
                    };
                }
            )*

            // Return empty VTable if ID not found
            $crate::SoulModeVTable {
                on_enter: None,
                on_update: None,
                on_exit: None,
            }
        }
    };
}
