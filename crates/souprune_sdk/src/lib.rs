pub mod context;
pub mod traits;

// Re-export for user convenience
// 重新导出，方便用户使用
pub use context::Context;
pub use traits::SoulMode;
// Also export HostApi from soup_api for macro use
// 也要把 soup_api 的 HostApi 导出来给宏用
pub use souprune_api::{Action, ContextHandle, HostApi, SoulModeVTable};

/// This macro is used to register a user's Struct as a Mod.
///
/// 这个宏用于把用户的 Struct 注册为 Mod
#[macro_export]
macro_rules! declare_soul_mode {
    ($mod_type:ty, $constructor:expr) => {
        // Static variable to store the API table
        // 静态变量存储 API 表
        static mut HOST_API: Option<$crate::HostApi> = None;

        // Wrapped state, used to hold the user's Mod instance in the C closure.
        // Note: This is simplified; in reality, we might need Box::into_raw to pass it as a pointer to C.
        // But for demonstration, we assume the Mod is stateless or created anew each time (this is incorrect, to be fixed).
        // Correction: The correct approach is that the context in vtable should be managed by Host.
        // But to keep Host unaware of the concrete Mod type, we usually use a global Map or Box::raw on the Mod side to store the instance.
        //
        // Given the current architecture design: ContextHandle is given by Host, representing the Entity.
        // Where does Mod's own self state live?
        //
        // Plan A: Mod is pure logic (stateless), data is all in Host's Component.
        // Plan B: Mod has state.
        //
        // Let's temporarily assume Mod is a stateless logic controller, state is accessed via context.
        // If Mod needs to store its own state, we need a more complex FFI handshake.
        //
        // Let's implement the Stateless Logic version first.
        //
        // 包装后的状态，用于在 C 闭包中持有用户的 Mod 实例
        // 注意：这里简化了，实际可能需要 Box::into_raw 把它变成指针传给 C
        // 但为了演示简单，我们假设 Mod 是无状态的，或者每次都创建新的（这不对，待会修正）
        // 修正：正确的做法是，vtable 里的 context 应该由 Host 管理，
        // 但为了让 Host 不知道 Mod 的具体类型，通常我们在 Mod 侧用一个全局 Map 或者 Box::raw 来存实例。
        //
        // 鉴于目前架构的设计：ContextHandle 是 Host 给的，代表 Entity。
        // Mod 自身的 self 状态存在哪？
        //
        // 方案 A: Mod 是纯逻辑（无状态），数据都在 Host 的 Component 里。
        // 方案 B: Mod 有状态。
        //
        // 让我们暂时假设 Mod 是无状态的逻辑控制器 (Controller)，状态都通过 context 存取。
        // 如果需要存 Mod 自己的状态，我们需要更复杂的 FFI 握手。
        //
        // 让我们先实现 无状态逻辑 版本。

        #[unsafe(no_mangle)]
        pub extern "C" fn create_soul_mode(api: *const $crate::HostApi) -> $crate::SoulModeVTable {
            unsafe {
                HOST_API = Some(api.read());
            }

            $crate::SoulModeVTable {
                on_enter: Some(wrapper_on_enter),
                on_update: Some(wrapper_on_update),
                on_exit: Some(wrapper_on_exit),
            }
        }

        // === Adapter Functions (C ABI -> Rust Trait) (适配器函数) ===

        extern "C" fn wrapper_on_update(handle: *mut $crate::ContextHandle, dt: f32) {
            unsafe {
                if let Some(api) = &HOST_API {
                    // 1. Construct Safe Context
                    // 1. 构建 Safe Context
                    let mut context = $crate::Context::new(handle, api);
                    // 2. Create user Mod instance (Assumed temporary new or singleton provided by user)
                    // *Note*: This limits Mod to not having cross-frame private fields.
                    // If Mod needs to store state, Host needs to provide a void* memory for us.
                    // Current architecture doc vtable has no void* user_data.
                    // We implement "Pure Functional/Stateless" first.
                    // 2. 创建用户的 Mod 实例 (这里假设每次都 new 一个临时的，或者由用户提供单例)
                    // *注意*：这限制了 Mod 自身不能有跨帧的 private field。
                    // 如果 Mod 需要存状态，需要 Host 提供一块 void* 内存给我们存。
                    // 现在的架构文档里，vtable 没有 void* user_data。
                    // 我们先按“纯函数式/无状态”来实现。
                    let mut mode = $constructor();
                    // 3. Call user logic
                    // 3. 调用用户逻辑
                    mode.on_update(&mut context, dt);
                }
            }
        }

        extern "C" fn wrapper_on_enter(handle: *mut $crate::ContextHandle) {
            unsafe {
                if let Some(api) = &HOST_API {
                    let mut context = $crate::Context::new(handle, api);
                    let mut mode = $constructor();
                    mode.on_enter(&mut context);
                }
            }
        }

        extern "C" fn wrapper_on_exit(handle: *mut $crate::ContextHandle) {
            unsafe {
                if let Some(api) = &HOST_API {
                    let mut context = $crate::Context::new(handle, api);
                    let mut mode = $constructor();
                    mode.on_exit(&mut context);
                }
            }
        }
    };
}
