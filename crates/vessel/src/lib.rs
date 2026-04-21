//! # Vessel — WASM-driven RON build infrastructure
//!
//! # Vessel — WASM 驱动的 RON 构建基础设施
//!
//! Vessel is a build-time host for the `WASM guest -> host -> RON` pipeline.
//! It loads a WebAssembly component, calls its `build` export, and writes the
//! emitted RON files to disk.
//!
//! Vessel 是 `WASM guest -> host -> RON` 流水线的构建期宿主。
//! 它会加载一个 WebAssembly component，调用它的 `build` 导出函数，
//! 然后将发射出的 RON 文件写入磁盘。

pub mod cli;
mod component_host;
mod output;

pub mod prelude;

pub use component_host::{BuildSummary, GeneratedRonFile, build_component, load_component_files};
pub use output::write_generated_files;
