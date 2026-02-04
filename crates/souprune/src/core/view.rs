//! # ui.rs
//!
//! ## Module Overview
//! This module manages the RON-driven UI system for the Overworld game state.
//! The UI is implemented using SDF rendering provided by bevy_alight_motion SdfMaterial and mesh-based text rendering
//! provided by bevy_rich_text3d.
//!
//! ## Source File Overview
//! This file defines the `OverworldViewPlugin` which loads View layouts from RON files.
//! Different UI styles (Undertale, Deltarune, etc.) can be achieved by simply modifying the RON files
//! without changing the code.
//!
//! ## 模块概述
//! 该模块管理着 Overworld 游戏状态的 RON 驱动 UI 系统。此 UI 是基于 bevy_alight_motion SdfMaterial 提供的 SDF 渲染
//! 与 bevy_rich_text3d 提供的基于 Mesh 的文本渲染实现的。
//!
//! ## 源文件概述
//! 该文件定义了从 RON 文件加载 View 布局的 `OverworldViewPlugin`。
//! 不同的 UI 风格（Undertale、Deltarune 等）只需修改 RON 文件即可实现，无需更改代码。

use crate::core::ron_loader::RonAssetLoader;

use bevy::prelude::*;

/// Universal UI update system set
///
/// 通用 UI 更新系统集
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ViewUpdate;

mod camera;
pub(crate) mod components;
mod custom_sprite_material;
pub(crate) mod layout;
mod lifecycle;
mod procedural_textures;
pub(crate) mod ron_view;
pub mod sdf_shape;
mod sdf_view_shape;
mod shaders;
mod state;
mod text;
mod visible_when;

pub use custom_sprite_material::PixelOutlineMaterial;

use camera::{
    update_camera_anchored_ui_on_camera_move_system, update_camera_anchored_ui_on_change_system,
    update_dynamic_camera_anchors_system,
};
pub use components::{
    ElementState, ViewElementHistory, find_element_by_full_name, find_elements_by_tag,
};
pub(crate) use layout::SdfStructureAsset;
use layout::ViewLayoutAsset;
use lifecycle::{
    StateTransitionTracker, UIInteractiveStateTracker, backpack_state_transition_system,
    state_transition_sound_system,
};
pub use ron_view::{HotReloadableViewRoot, PendingViewReloads, RonDrivenView, ViewLayoutHandle};
#[cfg(feature = "debug")]
use ron_view::{incremental_reload_system, watch_view_layout_changes_system};
use ron_view::{
    load_global_triggers_system, spawn_ron_view_system, ui_animation_init_system,
    update_dynamic_text_system, update_view_from_map_system,
};
use sdf_view_shape::update_sdf_view_shape_system;
use state::global_trigger_system;
use text::{assign_text_material_system, refresh_text_glyphs_system, show_text_when_ready_system};
use visible_when::evaluate_visible_when_system;

use crate::app_state::AppState;
use components::state_sprite::{
    evaluate_new_state_sprites_system, evaluate_state_sprite_rules_system,
    update_state_sprite_textures_system,
};
#[cfg(feature = "debug")]
use components::{CameraAnchored, ViewBox, ViewElement, ViewRoot};

use bevy::sprite_render::Material2dPlugin;

/// RON-driven View plugin for both Overworld and Battle scenes.
///
/// This plugin loads View layouts from RON files and renders them using SDF shapes and 3D text.
/// Different View styles can be achieved by modifying the RON files without code changes.
///
/// RON 驱动的 View 插件，支持 Overworld 和 Battle 场景。
/// 该插件从 RON 文件加载 View 布局，并使用 SDF 形状和 3D 文本进行渲染。
/// 通过修改 RON 文件可以实现不同的 View 风格，而无需更改代码。
pub(crate) struct CoreViewPlugin;

impl Plugin for CoreViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ViewLayoutAsset>()
            .register_asset_loader(RonAssetLoader::<ViewLayoutAsset>::new(&["view_layout.ron"]))
            .init_asset::<SdfStructureAsset>()
            .register_asset_loader(RonAssetLoader::<SdfStructureAsset>::new(&["sdf.ron"]))
            .add_plugins(Material2dPlugin::<
                custom_sprite_material::CustomSpriteMaterial,
            >::default())
            .add_plugins(Material2dPlugin::<
                custom_sprite_material::PixelOutlineMaterial,
            >::default())
            .init_resource::<ron_view::ViewGlobalTriggerConfig>()
            .init_resource::<UIInteractiveStateTracker>()
            .init_resource::<StateTransitionTracker>()
            .add_systems(Startup, procedural_textures::init_procedural_textures)
            // Use dynamic state transition detection instead of OnEnter/OnExit
            // since OverworldSubState is now string-based and dynamic
            // 使用动态状态转换检测替代 OnEnter/OnExit，因为 OverworldSubState 现在是基于字符串的动态状态
            .add_systems(
                Update,
                (
                    backpack_state_transition_system,
                    state_transition_sound_system,
                )
                    .run_if(in_state(AppState::Overworld)),
            )
            .add_systems(PreUpdate, refresh_text_glyphs_system)
            .add_systems(
                Update,
                ron_view::update_hp_bar_shader_params
                    .run_if(resource_exists::<procedural_textures::ProceduralTextures>),
            )
            // Split dynamic UI element updates into time-dependent (every frame) and fact-dependent (on change)
            // 将动态 UI 元素更新分为时间依赖（每帧）和 fact 依赖（变化时）
            .add_systems(
                Update,
                (
                    ron_view::update_time_dependent_ui_elements,
                    ron_view::update_fact_dependent_ui_elements,
                )
                    .run_if(resource_exists::<bevy_fact_rule_event::LayeredFactDatabase>),
            )
            // First group of UI systems
            .add_systems(
                Update,
                (
                    update_view_from_map_system,
                    load_global_triggers_system,
                    // Global trigger system for state changes (e.g., opening backpack)
                    // 全局触发器系统用于状态变更（如打开背包）
                    global_trigger_system,
                    spawn_ron_view_system,
                    ui_animation_init_system,
                )
                    .in_set(ViewUpdate),
            )
            // Second group of UI systems (rendering/display)
            .add_systems(
                Update,
                (
                    ron_view::setup_hp_bar_sprites
                        .run_if(resource_exists::<procedural_textures::ProceduralTextures>),
                    update_sdf_view_shape_system,
                    assign_text_material_system,
                    show_text_when_ready_system,
                    update_camera_anchored_ui_on_camera_move_system,
                    update_camera_anchored_ui_on_change_system,
                    update_dynamic_camera_anchors_system,
                    update_dynamic_text_system,
                    // State sprite systems (data-driven state management)
                    // 状态精灵系统（数据驱动的状态管理）
                    evaluate_state_sprite_rules_system,
                    evaluate_new_state_sprites_system,
                    update_state_sprite_textures_system,
                    // visible_when expression evaluation system
                    // visible_when 表达式评估系统
                    evaluate_visible_when_system
                        .run_if(resource_exists::<bevy_fact_rule_event::LayeredFactDatabase>),
                )
                    .in_set(ViewUpdate),
            );

        #[cfg(feature = "debug")]
        {
            // Hot reload systems (debug only)
            // 热重载系统（仅 debug 模式）
            app.init_resource::<ron_view::PendingViewReloads>()
                .add_systems(
                    Update,
                    (watch_view_layout_changes_system, incremental_reload_system)
                        .chain()
                        .in_set(ViewUpdate),
                )
                .register_type::<ViewBox>()
                .register_type::<CameraAnchored>()
                .register_type::<ViewElement>()
                .register_type::<ViewRoot>()
                .register_type::<ViewElementHistory>()
                .register_type::<ElementState>();
        }
    }
}
