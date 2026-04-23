//! # souprune_vessel — SoupRune extension for Vessel
//!
//! # souprune_vessel — Vessel 的 SoupRune 扩展
//!
//! Provides export support and authoring helpers for generating SoupRune
//! RON files from Rust code.
//!
//! 提供导出支持与编写辅助，
//! 用于从 Rust 代码生成 SoupRune RON 文件。

pub mod build_support;
pub mod guest;
pub mod prelude;

pub use souprune_vessel_macros::performance;

/// Export a Rust guest as a Vessel build-time WASM component.
///
/// 将一个 Rust guest 导出为 Vessel 构建期 WASM component。
#[macro_export]
macro_rules! vessel_guest {
    (fn build($reg:ident : &mut $reg_ty:ty) -> $ret:ty $body:block) => {
        struct VesselContentModule;

        impl $crate::guest::wit::Guest for VesselContentModule {
            fn build() -> ::std::vec::Vec<$crate::guest::wit::GeneratedFile> {
                let mut $reg: $reg_ty = <$reg_ty>::new();
                let build_result: $ret = (|| -> $ret { $body })();
                if let Err(err) = build_result {
                    panic!("[vessel_guest] build failed: {err}");
                }
                $reg.into_generated_files()
            }
        }

        $crate::guest::wit::export!(VesselContentModule with_types_in $crate::guest);
    };
}
