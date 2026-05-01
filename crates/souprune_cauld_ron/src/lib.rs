//! # souprune_cauld_ron — SoupRune extension for Cauld-ron
//!
//! # souprune_cauld_ron — Cauld-ron 的 SoupRune 扩展
//!
//! Provides export support and authoring helpers for generating SoupRune
//! RON files from Rust code.
//!
//! 提供导出支持与编写辅助，
//! 用于从 Rust 代码生成 SoupRune RON 文件。

pub mod build_support;
pub mod deps;
pub mod expr;
pub mod float_output;
pub mod guest;
#[cfg(not(target_arch = "wasm32"))]
pub mod host;
pub mod prelude;

#[cfg(not(target_arch = "wasm32"))]
pub use host::{build_component, write_generated_files};

/// Export a Rust guest as a Cauld-ron build-time WASM component.
///
/// 将一个 Rust guest 导出为 Cauld-ron 构建期 WASM component。
///
/// # Arguments
/// - `$reg`: Identifier for the registry instance.
/// - `$reg_ty`: Type of the registry (usually `Registry`).
/// - `$body`: The build logic that populates the registry.
///
/// # 参数
/// - `$reg`: 注册表实例的标识符。
/// - `$reg_ty`: 注册表类型（通常为 `Registry`）。
/// - `$body`: 填充注册表的构建逻辑。
#[macro_export]
macro_rules! cauld_ron_guest {
    (fn build($reg:ident : &mut $reg_ty:ty) -> $ret:ty $body:block) => {
        struct CauldRonContentModule;

        impl $crate::guest::wit::Guest for CauldRonContentModule {
            fn build() -> ::std::vec::Vec<$crate::guest::wit::GeneratedFile> {
                let mut $reg: $reg_ty = <$reg_ty>::new();
                let build_result: $ret = (|| -> $ret { $body })();
                if let Err(err) = build_result {
                    panic!("[cauld_ron_guest] build failed: {err}");
                }
                $reg.into_generated_files()
            }
        }

        $crate::guest::wit::export!(CauldRonContentModule with_types_in $crate::guest);
    };
}
