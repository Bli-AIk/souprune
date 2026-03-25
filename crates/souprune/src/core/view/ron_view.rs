//! Collects the runtime pieces for Souprune's RON-driven View implementation.
//!
//! 汇总 Souprune 的 RON 驱动 View 实现所需的运行时部件。
//!
//! This module is the root of the dynamic view pipeline: parsing layouts,
//! evaluating expressions, spawning entities, reacting to hot reload, and
//! updating generated UI at runtime. The submodules hold the concrete logic;
//! this file defines the module boundary and re-exports the small surface that
//! other parts of the view system actually consume.
//!
//! 这个模块是动态 View 管线的根入口：负责布局解析、表达式求值、实体生成、
//! 热重载响应以及运行时更新已生成的 UI。具体逻辑分散在各个子模块里，而这个
//! 文件负责定义模块边界，并重导出 View 系统真正需要消费的那小部分表面。

pub mod evaluation;
pub mod parsing;
pub mod player_data;
pub mod reload;
pub mod resources;
pub mod setup;
pub mod spawn;
pub mod spawn_helpers;
mod spawn_nodes;
pub mod update;

// Re-export common types and systems
pub use resources::{HotReloadableViewRoot, RonDrivenView};

// Re-export RepeatContext for use in expression evaluation

pub use setup::*;
pub use spawn::*;
pub use update::*;
