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

use crate::app_state::overworld::OverworldState;
use bevy::prelude::*;

pub(crate) mod components;
mod systems;

use components::UILayerNavigationConfig;

#[cfg(feature = "debug")]
use components::{
    BoxCursor, BoxCursorPosition, BoxCursorVisibility, CameraAnchored, OverworldUI, OverworldUIBox,
    UILayer,
};

pub(crate) struct UndertaleOverworldUIPlugin;

impl Plugin for UndertaleOverworldUIPlugin {
    fn build(&self, app: &mut App) {
        use systems::*;

        app.init_resource::<UILayerNavigationConfig>()
            .add_systems(OnEnter(OverworldState::Backpack), spawn_backpack_ui_system)
            .add_systems(OnExit(OverworldState::Backpack), destroy_backpack_ui_system)
            .add_systems(PreUpdate, refresh_text_glyphs_system)
            .add_systems(
                Update,
                (
                    menu_overworld_state_transitions_system,
                    update_overworld_ui_navigation_system,
                    draw_backpack_ui_system,
                    update_overworld_ui_box_system,
                    spawn_box_cursor_visual_system,
                    update_box_cursor_state_system,
                    show_text_when_ready_system,
                    update_camera_anchored_ui_system,
                ),
            );

        #[cfg(feature = "debug")]
        {
            app.register_type::<OverworldUI>()
                .register_type::<OverworldUIBox>()
                .register_type::<UILayer>()
                .register_type::<CameraAnchored>()
                .register_type::<BoxCursor>()
                .register_type::<BoxCursorPosition>()
                .register_type::<BoxCursorVisibility>();
        }
    }
}

pub(crate) struct DeltaruneOverworldUIPlugin;

impl Plugin for DeltaruneOverworldUIPlugin {
    fn build(&self, app: &mut App) {}
}
