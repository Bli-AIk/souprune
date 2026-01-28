//! # layout.rs
//!
//! # layout.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! The UI Layout module entry point.
//! This module organizes the UI Schema definition and Serde helper types.
//! It corresponds to the `layout/` directory.
//!
//! UI 布局模块入口点。
//! 本模块组织了 UI Schema 定义和 Serde 辅助类型。
//! 它对应 `layout/` 目录。

pub mod serde_types;
pub mod ui_schema;

pub use serde_types::*;
pub use ui_schema::*;
