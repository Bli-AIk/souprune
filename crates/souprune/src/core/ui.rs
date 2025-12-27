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

use crate::app_state::overworld::OverworldState;
use crate::core::ron_loader::RonAssetLoader;

use bevy::prelude::*;

/// Universal UI update system set
///
/// 通用 UI 更新系统集
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UIUpdate;

mod camera;
pub(crate) mod components;
mod cursor;
mod custom_sprite_material;
pub(crate) mod layout;
mod lifecycle;
mod procedural_textures;
mod ron_ui;
mod shaders;
mod smud_shape;
mod state;
mod text;

use camera::{
    update_camera_anchored_ui_on_camera_move_system, update_camera_anchored_ui_on_change_system,
    update_dynamic_camera_anchors_system,
};
use components::{UILayerNavigationConfig, UILayerTransitionConfig};
use cursor::{spawn_box_cursor_visual_system, update_box_cursor_state_system};
pub(crate) use layout::SmudStructureAsset;
use layout::UILayoutAsset;
use lifecycle::{despawn_backpack_ui_system, spawn_backpack_ui_system};
pub use ron_ui::{UILayoutHandle, UILayoutWatcher};
use ron_ui::{
    load_navigation_and_transitions_system, spawn_ron_ui_system, ui_animation_init_system,
    update_dynamic_text_system, update_ui_from_map_system, watch_ui_layout_changes_system,
};
use smud_shape::{
    update_smud_shape_system, update_ui_box_visibility_system,
    update_ui_container_visibility_system,
};
use state::{menu_overworld_state_transitions_system, update_overworld_ui_navigation_system};
use text::{assign_text_material_system, refresh_text_glyphs_system, show_text_when_ready_system};

use crate::app_state::AppState;
#[cfg(feature = "debug")]
use components::{
    BoxCursor, BoxCursorPosition, BoxCursorVisibility, CameraAnchored, RonUI, UIBox,
    UIBoxVisibility, UILayer,
};

use bevy::sprite_render::Material2dPlugin;

/// RON-driven UI plugin for both Overworld and Battle scenes.
///
/// This plugin loads UI layouts from RON files and renders them using SDF shapes and 3D text.
/// Different UI styles can be achieved by modifying the RON files without code changes.
///
/// RON 驱动的 UI 插件，支持 Overworld 和 Battle 场景。
/// 该插件从 RON 文件加载 UI 布局，并使用 SDF 形状和 3D 文本进行渲染。
/// 通过修改 RON 文件可以实现不同的 UI 风格，而无需更改代码。
pub(crate) struct CoreUIPlugin;

impl Plugin for CoreUIPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<UILayoutAsset>()
            .register_asset_loader(RonAssetLoader::<UILayoutAsset>::new(&["ui_layout.ron"]))
            .init_asset::<SmudStructureAsset>()
            .register_asset_loader(RonAssetLoader::<SmudStructureAsset>::new(&["smud.ron"]))
            .add_plugins(Material2dPlugin::<
                custom_sprite_material::CustomSpriteMaterial,
            >::default())
            .init_resource::<UILayerNavigationConfig>()
            .init_resource::<UILayerTransitionConfig>()
            .init_resource::<ron_ui::UIGlobalTriggerConfig>()
            .add_systems(Startup, procedural_textures::init_procedural_textures)
            .add_systems(
                Update,
                spawn_backpack_ui_system
                    .run_if(in_state(OverworldState::Backpack).and(in_state(AppState::Overworld))),
            )
            .add_systems(OnExit(OverworldState::Backpack), despawn_backpack_ui_system)
            .add_systems(PreUpdate, refresh_text_glyphs_system)
            .add_systems(
                Update,
                ron_ui::update_hp_bar_shader_params
                    .run_if(resource_exists::<procedural_textures::ProceduralTextures>),
            )
            .add_systems(
                Update,
                ron_ui::update_dynamic_ui_elements
                    .run_if(resource_exists::<crate::core::data::PlayerData>),
            )
            .add_systems(
                Update,
                (
                    update_ui_from_map_system,
                    watch_ui_layout_changes_system,
                    crate::core::ui::ron_ui::reload::rebuild_reloaded_ui_system,
                    load_navigation_and_transitions_system,
                    menu_overworld_state_transitions_system,
                    update_overworld_ui_navigation_system,
                    spawn_ron_ui_system,
                    ui_animation_init_system,
                    ron_ui::setup_hp_bar_sprites
                        .run_if(resource_exists::<procedural_textures::ProceduralTextures>),
                    update_smud_shape_system,
                    update_ui_box_visibility_system,
                    update_ui_container_visibility_system,
                    assign_text_material_system,
                    spawn_box_cursor_visual_system,
                    update_box_cursor_state_system,
                    show_text_when_ready_system,
                    update_camera_anchored_ui_on_camera_move_system,
                    update_camera_anchored_ui_on_change_system,
                    update_dynamic_camera_anchors_system,
                    update_dynamic_text_system,
                )
                    .in_set(UIUpdate),
            );

        #[cfg(feature = "debug")]
        {
            app.register_type::<RonUI>()
                .register_type::<UIBox>()
                .register_type::<UILayer>()
                .register_type::<CameraAnchored>()
                .register_type::<UIBoxVisibility>()
                .register_type::<BoxCursor>()
                .register_type::<BoxCursorPosition>()
                .register_type::<BoxCursorVisibility>();
        }
    }
}
