//! Collects optional tooling and integration helpers that support Souprune without belonging to the core runtime.
//!
//! 汇总那些支撑 Souprune、但不属于核心运行时的可选工具与集成辅助模块。
//!
//! This module exists for support code such as debug tools, alternate asset
//! loaders, and external-content adapters. These pieces are useful to the
//! application, but they are not foundational enough to live in `core`, nor are
//! they part of the game-state layer in `app_state`.
//!
//! 这里放的是调试工具、额外资源加载器以及外部内容适配器之类的支撑代码。
//! 它们对应用很有用，但又没有基础设施到应当进入 `core`，也不属于
//! `app_state` 里的具体玩法状态。

pub mod debug;
pub mod markdown;
pub mod mortar;
pub mod multi_source;
