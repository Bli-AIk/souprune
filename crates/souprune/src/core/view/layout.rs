//! # layout.rs
//!
//! # layout.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! The View Layout module entry point.
//! This module organizes the View Schema definition and Serde helper types.
//! It corresponds to the `layout/` directory.
//!
//! 视图布局模块入口点。
//! 本模块组织了视图 Schema 定义和 Serde 辅助类型。
//! 它对应 `layout/` 目录。

pub mod serde_types;
pub mod view_schema;

pub use serde_types::*;
pub use view_schema::*;
