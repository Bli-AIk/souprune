//! Collects the bootstrap surface used to start the Souprune runtime.
//!
//! 汇总启动 Souprune 运行时所需的 bootstrap 边界。
//!
//! Acts as the narrow entry for startup-only wiring. It re-exports the
//! logging setup, plugin selection, resource seeding, and final runner so the
//! crate root can start the game without knowing the details of each startup
//! concern. Gameplay systems do not belong here; this module only gathers the
//! pieces that assemble the app.
//!
//! 启动期装配的窄入口。它把日志初始化、插件选择、资源注入和
//! 最终运行入口重新汇总出来，让 crate 根入口可以启动游戏，而不需要知道各个
//! 启动细节。这里不承载玩法系统，只负责把应用装配起来。

pub(crate) mod logging;
pub(crate) mod plugins;
pub(crate) mod resources;
pub(crate) mod runner;

pub use plugins::{get_file_importer_plugins, get_game_plugins, get_third_plugins};
pub use resources::{insert_font_resources, insert_input_resources, reset_game_state};
pub use runner::run;
