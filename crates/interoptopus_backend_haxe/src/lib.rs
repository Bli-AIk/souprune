//! Generates Haxe (hxcpp) bindings for [Interoptopus](https://github.com/ralfbiedert/interoptopus).
//!
//! This backend generates Haxe extern classes that can be used with hxcpp to call
//! native Rust code through C FFI.
//!
//! 为 Interoptopus 生成 Haxe (hxcpp) 绑定。
//! 此后端生成可用于 hxcpp 调用原生 Rust 代码的 Haxe extern 类。
//!
//! # Usage
//!
//! ```rust,ignore
//! use interoptopus_backend_haxe::{Config, Interop};
//!
//! let inventory = my_library::my_inventory();
//!
//! Interop::builder()
//!     .inventory(inventory)
//!     .config(Config {
//!         lib_name: "souprune".to_string(),
//!         package_name: "souprune.ffi".to_string(),
//!         ..Default::default()
//!     })
//!     .build()?
//!     .write_file("bindings/haxe/SoupruneApi.hx")?;
//! ```

mod converters;
mod interop;

pub use interop::{Config, Interop, InteropBuilder};
