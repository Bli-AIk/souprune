//! Collects the editor bootstrap modules that turn the crate into a working editing app.
//!
//! 汇总把编辑器 crate 组装成可运行编辑应用所需的 bootstrap 模块。
//!
//! This file is the editor-side startup boundary. It keeps configuration,
//! resources, preview behavior, panel wiring, and application setup grouped
//! under one bootstrap surface so the crate root can stay focused on exposing
//! the editor plugin instead of spelling out startup details.
//!
//! 这个文件是编辑器侧的启动边界。它把配置、资源、预览行为、面板装配和应用
//! 初始化收拢到同一个 bootstrap 面上，让 crate 根入口只关注暴露编辑器插件，
//! 而不用展开一串启动细节。

mod config;
mod mode;
mod panels;
mod preview;
mod resources;
mod setup;

pub(crate) use setup::build_editor_app;
