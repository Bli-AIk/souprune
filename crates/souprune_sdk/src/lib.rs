//! The main entry point for the Rust Modding SDK.
//! It re-exports necessary types and provides the `declare_behaviors!` and `declare_danmaku!` macros
//! to simplify mod registration.
//!
//! Rust 模组开发 SDK 的主要入口点。
//! 重新导出必要的类型，并提供 `declare_behaviors!` 和 `declare_danmaku!` 宏以简化模组注册流程。

pub mod context;
pub mod traits;

// Re-export for user convenience
// 重新导出，方便用户使用
pub use context::{BulletContext, BulletOutput, Context, Vec2};
pub use souprune_api::{
    Action, BehaviorInstance, BehaviorVTable, BulletContextC, BulletOutputC, ContextHandle,
    DanmakuInstance, DanmakuVTable, HostApi, Vec2C, c_float, c_void,
};
pub use traits::{Behavior, DanmakuBehavior};

/// This macro is used to register user's Structs as Behaviors (for player/entity control).
///
/// 这个宏用于把用户的 Struct 注册为 Behavior（用于玩家/实体控制）
#[macro_export]
macro_rules! declare_behaviors {
    ( $( ($id:literal, $mod_type:ty, $constructor:expr) ),* $(,)? ) => {
        static mut HOST_API: Option<$crate::HostApi> = None;

        #[unsafe(no_mangle)]
        pub extern "C" fn get_behavior_count() -> u32 {
            let mut count = 0;
            $( count += 1; let _ = $id; )*
            count
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn get_behavior_id(index: u32) -> *const u8 {
            let ids = [ $( concat!($id, "\0").as_ptr() ),* ];
            if (index as usize) < ids.len() { ids[index as usize] } else { std::ptr::null() }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn create_behavior(
            id: *const u8,
            api: *const $crate::HostApi,
        ) -> $crate::BehaviorInstance {
            unsafe { HOST_API = Some(api.read()); }

            let id_str = unsafe {
                std::ffi::CStr::from_ptr(id as *const core::ffi::c_char).to_str().unwrap_or("")
            };

            $(
                if id_str == $id {
                    let instance = Box::new($constructor());
                    let instance_ptr = Box::into_raw(instance) as *mut core::ffi::c_void;

                    extern "C" fn wrapper_on_enter(instance: *mut core::ffi::c_void, handle: *mut $crate::ContextHandle) {
                        unsafe {
                            if let Some(api) = &HOST_API {
                                let mut context = $crate::Context::new(handle, api);
                                let mode = &mut *(instance as *mut $mod_type);
                                $crate::Behavior::on_enter(mode, &mut context);
                            }
                        }
                    }

                    extern "C" fn wrapper_on_update(instance: *mut core::ffi::c_void, handle: *mut $crate::ContextHandle, dt: $crate::c_float) {
                        unsafe {
                            if let Some(api) = &HOST_API {
                                let mut context = $crate::Context::new(handle, api);
                                let mode = &mut *(instance as *mut $mod_type);
                                $crate::Behavior::on_update(mode, &mut context, dt);
                            }
                        }
                    }

                    extern "C" fn wrapper_on_exit(instance: *mut core::ffi::c_void, handle: *mut $crate::ContextHandle) {
                        unsafe {
                            if let Some(api) = &HOST_API {
                                let mut context = $crate::Context::new(handle, api);
                                let mode = &mut *(instance as *mut $mod_type);
                                $crate::Behavior::on_exit(mode, &mut context);
                            }
                        }
                    }

                    extern "C" fn wrapper_destroy(instance: *mut core::ffi::c_void) {
                         unsafe { let _ = Box::from_raw(instance as *mut $mod_type); }
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

            $crate::BehaviorInstance {
                instance: std::ptr::null_mut(),
                vtable: $crate::BehaviorVTable { on_enter: None, on_update: None, on_exit: None, destroy: None }
            }
        }
    };
}

/// This macro is used to register danmaku (bullet pattern) behaviors.
#[macro_export]
macro_rules! declare_danmaku {
    ( $( ($id:literal, $danmaku_type:ty, $constructor:expr) ),* $(,)? ) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn get_algorithm_count() -> u32 {
            let mut count = 0;
            $( count += 1; let _ = $id; )*
            count
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn get_algorithm_id(index: u32) -> *const u8 {
            let ids = [ $( concat!($id, "\0").as_ptr() ),* ];
            if (index as usize) < ids.len() { ids[index as usize] } else { std::ptr::null() }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn create_danmaku(id: *const u8) -> $crate::DanmakuInstance {
            let id_str = unsafe {
                std::ffi::CStr::from_ptr(id as *const core::ffi::c_char).to_str().unwrap_or("")
            };

            $(
                if id_str == $id {
                    let instance = Box::new($constructor());
                    let instance_ptr = Box::into_raw(instance) as *mut core::ffi::c_void;

                    extern "C" fn wrapper_on_enter(instance: *mut core::ffi::c_void, ctx: *const $crate::BulletContextC) {
                        unsafe {
                            let context = $crate::BulletContext::from_raw(ctx);
                            let danmaku = &mut *(instance as *mut $danmaku_type);
                            $crate::DanmakuBehavior::on_enter(danmaku, &context);
                        }
                    }

                    extern "C" fn wrapper_on_update(instance: *mut core::ffi::c_void, ctx: *const $crate::BulletContextC) -> $crate::BulletOutputC {
                        unsafe {
                            let context = $crate::BulletContext::from_raw(ctx);
                            let danmaku = &mut *(instance as *mut $danmaku_type);
                            $crate::DanmakuBehavior::on_update(danmaku, &context).to_output_c()
                        }
                    }

                    extern "C" fn wrapper_on_exit(instance: *mut core::ffi::c_void) {
                        unsafe {
                            let danmaku = &mut *(instance as *mut $danmaku_type);
                            $crate::DanmakuBehavior::on_exit(danmaku);
                        }
                    }

                    extern "C" fn wrapper_destroy(instance: *mut core::ffi::c_void) {
                        unsafe { let _ = Box::from_raw(instance as *mut $danmaku_type); }
                    }

                    return $crate::DanmakuInstance {
                        instance: instance_ptr,
                        vtable: $crate::DanmakuVTable {
                            on_enter: Some(wrapper_on_enter),
                            on_update: Some(wrapper_on_update),
                            on_exit: Some(wrapper_on_exit),
                            destroy: Some(wrapper_destroy),
                        }
                    };
                }
            )*

            $crate::DanmakuInstance {
                instance: std::ptr::null_mut(),
                vtable: $crate::DanmakuVTable { on_enter: None, on_update: None, on_exit: None, destroy: None }
            }
        }
    };
}
