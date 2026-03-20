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
