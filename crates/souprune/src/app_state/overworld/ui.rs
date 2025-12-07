//! # ui.rs
//!
//! ## Module Overview
//! This module manages the Undertale-style and Deltarune-style UI components in the Overworld game state.
//! The UI is implemented using SDF rendering provided by bevy_smud and mesh-based text rendering
//! provided by bevy_rich_text3d.
//!
//! ## Source File Overview
//! This file defines the `UndertaleOverworldUIPlugin` and `DeltaruneOverworldUIPlugin`.
//! Currently, you should choose to use one of them for the entire project, rather than using both.
//!
//! ## 模块概述
//! 该模块管理着 Undertale 与 Deltarune 样式的 Overworld 游戏状态中的 UI 组件。此 UI 是基于 bevy_smud 提供的 SDF 渲染
//! 与 bevy_rich_text3d 提供的 基于 Mesh 的文本渲染实现的。
//!
//! ## 源文件概述
//! 该文件定义了 `UndertaleOverworldUIPlugin` 与 `DeltaruneOverworldUIPlugin`。
//! 目前，对于整个项目，你应当任选其一使用，而不是同时使用两者。

use bevy::prelude::*;

mod asset_loader;
mod camera;
pub(crate) mod components;
mod cursor;
pub(crate) mod layout;
mod lifecycle;
mod ron_ui_system;
mod state;
mod text;
mod ui_box;

use asset_loader::UILayoutAssetLoader;
use camera::{
    update_camera_anchored_ui_on_camera_move_system, update_camera_anchored_ui_on_change_system,
};
use components::{UILayerNavigationConfig, UILayerTransitionConfig};
use cursor::{spawn_box_cursor_visual_system, update_box_cursor_state_system};
use layout::UILayoutAsset;
use lifecycle::spawn_backpack_ui_system;
use ron_ui_system::{
    load_navigation_and_transitions_system,
    load_ui_layout_system, spawn_ron_ui_system,
};
use state::{menu_overworld_state_transitions_system, update_overworld_ui_navigation_system};
use text::{refresh_text_glyphs_system, show_text_when_ready_system};
use ui_box::{
    draw_backpack_ui_system, update_overworld_ui_box_system,
    update_overworld_ui_box_visibility_system,
};

#[cfg(feature = "debug")]
use components::{
    BoxCursor, BoxCursorPosition, BoxCursorVisibility, CameraAnchored, OverworldUI, OverworldUIBox,
    OverworldUIBoxVisibility, UILayer,
};
use crate::app_state::overworld::ui::ron_ui_system::{hot_reload_ron_ui_system, rebuild_reloaded_ui_system};

pub(crate) struct UndertaleOverworldUIPlugin;

impl Plugin for UndertaleOverworldUIPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<UILayoutAsset>()
            .init_asset_loader::<UILayoutAssetLoader>()
            .init_resource::<UILayerNavigationConfig>()
            .init_resource::<UILayerTransitionConfig>()
            .add_systems(Startup, load_ui_layout_system)
            .add_systems(Update, spawn_backpack_ui_system)
            .add_systems(PreUpdate, refresh_text_glyphs_system)
            .add_systems(
                Update,
                (
                    load_navigation_and_transitions_system,
                    menu_overworld_state_transitions_system,
                    update_overworld_ui_navigation_system,
                    spawn_ron_ui_system,
                    draw_backpack_ui_system,
                    update_overworld_ui_box_system,
                    update_overworld_ui_box_visibility_system,
                    spawn_box_cursor_visual_system,
                    update_box_cursor_state_system,
                    show_text_when_ready_system,
                    update_camera_anchored_ui_on_camera_move_system,
                    update_camera_anchored_ui_on_change_system,
                ),
            );

        #[cfg(feature = "debug")]
        {
            app.add_systems(
                Update,
                (hot_reload_ron_ui_system, rebuild_reloaded_ui_system),
            );
        }

        #[cfg(feature = "debug")]
        {
            app.register_type::<OverworldUI>()
                .register_type::<OverworldUIBox>()
                .register_type::<UILayer>()
                .register_type::<CameraAnchored>()
                .register_type::<OverworldUIBoxVisibility>()
                .register_type::<BoxCursor>()
                .register_type::<BoxCursorPosition>()
                .register_type::<BoxCursorVisibility>();
        }
    }
}

pub(crate) struct DeltaruneOverworldUIPlugin;

impl Plugin for DeltaruneOverworldUIPlugin {
    fn build(&self, _app: &mut App) {}
}
