//! Reads editor startup values that must be derived from the active Souprune project configuration.
//!
//! 读取那些必须从当前 Souprune 项目配置推导出来的编辑器启动参数。
//!
//! Bridges runtime project config into editor-only startup data. It
//! extracts preview resolution information and builds the key map used by the
//! embedded View preview, so the editor can mirror the active game's input and
//! rendering assumptions instead of hard-coding its own copies.
//!
//! 把运行时项目配置桥接成编辑器启动期需要的数据。它负责提取预览分辨率，
//! 也负责构建嵌入式 View 预览使用的按键映射，让编辑器能够复用当前游戏项目的
//! 输入与渲染设定，而不是偷偷维护一套自己的副本。

use bevy::prelude::*;
use souprune::editor_api as api;

use crate::panels;

pub(super) fn load_resolution_config(app: &mut App) -> (u32, u32, u32) {
    let config = app
        .world()
        .get_resource::<souprune::config::SoupruneConfig>();

    (
        config.map_or(2, |c| c.window.resolution_scale),
        config.map_or(320u32, |c| c.render.base_resolution_width),
        config.map_or(240u32, |c| c.render.base_resolution_height),
    )
}

pub(super) fn insert_preview_key_map(app: &mut App) {
    let config = app
        .world()
        .get_resource::<souprune::config::SoupruneConfig>()
        .expect("SoupruneConfig required");
    let projects_base = souprune::config::get_projects_base_path();
    let input_config_path = projects_base
        .join(&config.project.mod_name)
        .join(&config.game.input_config_path);
    let input_config = api::input::InputConfig::load_from_file(&input_config_path);
    let key_map =
        panels::view_preview::ViewPreviewKeyMap(input_config.build_keycode_to_action_map());
    app.insert_resource(key_map);
}
