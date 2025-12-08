//! # ui.rs
//!
//! ## Module Overview
//! This module manages the RON-driven UI system for the Overworld game state.
//! The UI is implemented using SDF rendering provided by bevy_smud and mesh-based text rendering
//! provided by bevy_rich_text3d.
//!
//! ## Source File Overview
//! This file defines the `OverworldUIPlugin` which loads UI layouts from RON files.
//! Different UI styles (Undertale, Deltarune, etc.) can be achieved by simply modifying the RON files
//! without changing the code.
//!
//! ## 模块概述
//! 该模块管理着 Overworld 游戏状态的 RON 驱动 UI 系统。此 UI 是基于 bevy_smud 提供的 SDF 渲染
//! 与 bevy_rich_text3d 提供的基于 Mesh 的文本渲染实现的。
//!
//! ## 源文件概述
//! 该文件定义了从 RON 文件加载 UI 布局的 `OverworldUIPlugin`。
//! 不同的 UI 风格（Undertale、Deltarune 等）只需修改 RON 文件即可实现，无需更改代码。

use crate::overworld::ui::ron_ui_system::rebuild_reloaded_ui_system;
use bevy::prelude::*;

mod asset_loader;
mod camera;
pub(crate) mod components;
mod cursor;
pub(crate) mod layout;
mod lifecycle;
mod ron_ui_system;
mod shaders;
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
    load_navigation_and_transitions_system, spawn_ron_ui_system, update_ui_from_map_system,
};
use state::{menu_overworld_state_transitions_system, update_overworld_ui_navigation_system};
use text::{refresh_text_glyphs_system, show_text_when_ready_system};
use ui_box::{update_overworld_ui_box_system, update_overworld_ui_box_visibility_system};

use crate::app_state::overworld::ui::ron_ui_system::hot_reload_ron_ui_system;
#[cfg(feature = "debug")]
use components::{
    BoxCursor, BoxCursorPosition, BoxCursorVisibility, CameraAnchored, OverworldUI, OverworldUIBox,
    OverworldUIBoxVisibility, UILayer,
};

/// RON-driven UI plugin for the Overworld.
///
/// This plugin loads UI layouts from RON files and renders them using SDF shapes and 3D text.
/// Different UI styles can be achieved by modifying the RON files without code changes.
///
/// RON 驱动的 Overworld UI 插件。
/// 该插件从 RON 文件加载 UI 布局，并使用 SDF 形状和 3D 文本进行渲染。
/// 通过修改 RON 文件可以实现不同的 UI 风格，而无需更改代码。
pub(crate) struct OverworldUIPlugin;

impl Plugin for OverworldUIPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<UILayoutAsset>()
            .init_asset_loader::<UILayoutAssetLoader>()
            .init_resource::<UILayerNavigationConfig>()
            .init_resource::<UILayerTransitionConfig>()
            .init_resource::<ron_ui_system::UIGlobalTriggerConfig>()
            .add_systems(Update, spawn_backpack_ui_system)
            .add_systems(PreUpdate, refresh_text_glyphs_system)
            .add_systems(
                Update,
                (
                    update_ui_from_map_system,
                    rebuild_reloaded_ui_system,
                    load_navigation_and_transitions_system,
                    menu_overworld_state_transitions_system,
                    update_overworld_ui_navigation_system,
                    spawn_ron_ui_system,
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
            app.add_systems(Update, (hot_reload_ron_ui_system));
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
