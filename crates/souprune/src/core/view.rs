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
pub mod dynamic_material;
pub(crate) mod expr_eval;
pub(crate) mod layout;
mod lifecycle;
mod procedural_textures;
pub mod reconcile;
pub(crate) mod ron_view;
pub mod sdf_shape;
mod sdf_view_shape;
mod state;
mod text;
mod visible_when;

pub use custom_sprite_material::PixelOutlineMaterial;

use camera::{
    update_camera_anchored_ui_on_camera_move_system, update_camera_anchored_ui_on_change_system,
    update_dynamic_camera_anchors_system,
};
pub use components::{
    ElementState, ViewElementHistory, ViewRoot, find_element_by_full_name, find_elements_by_tag,
};
pub(crate) use layout::SdfStructureAsset;
use layout::ViewLayoutAsset;
use lifecycle::{
    StateTransitionTracker, UIInteractiveStateTracker, backpack_state_transition_system,
    state_transition_sound_system,
};
pub use ron_view::{RonDrivenView, ViewLayoutHandle};
use ron_view::{
    load_global_triggers_system, spawn_ron_view_system, ui_animation_init_system,
    update_dynamic_text_system, update_view_from_map_system,
};
use sdf_view_shape::update_sdf_view_shape_system;
use state::global_trigger_system;
use text::{assign_text_material_system, refresh_text_glyphs_system, show_text_when_ready_system};
use visible_when::evaluate_visible_when_system;

/// Message to request spawning a new View.
///
/// 请求生成新 View 的消息。
#[derive(Message, Debug, Clone)]
pub struct SpawnViewRequest {
    /// Path to the view layout asset (e.g., "overworld/view/dialogue.view.ron")
    ///
    /// 视图布局资源路径
    pub path: String,
}

/// Message to request despawning Views.
///
/// 请求销毁 View 的消息。
#[derive(Message, Debug, Clone)]
pub struct DespawnViewRequest {
    /// Optional path to despawn a specific View. If None, despawns all dynamically spawned Views.
    ///
    /// 可选的路径，用于销毁特定 View。如果为 None，则销毁所有动态生成的 View。
    pub path: Option<String>,
}

use crate::app_state::AppState;
use components::state_sprite::{
    evaluate_new_state_sprites_system, evaluate_state_sprite_rules_system,
    update_state_sprite_textures_system,
};
#[cfg(feature = "debug")]
use components::{CameraAnchored, ViewBox, ViewElement};
use lifecycle::cleanup_view_rules_system;
use lifecycle::process_pending_view_rules_system;
use reconcile::ViewReconciliationPlugin;

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
            // Only support .view.ron format (legacy .view_layout.ron is fully deprecated)
            // 仅支持 .view.ron 格式（旧的 .view_layout.ron 已完全弃用）
            .register_asset_loader(RonAssetLoader::<ViewLayoutAsset>::new(&["view.ron"]))
            .init_asset::<SdfStructureAsset>()
            .register_asset_loader(RonAssetLoader::<SdfStructureAsset>::new(&["sdf.ron"]))
            // Register ShaderMaterial for inspector
            // 为检查器注册 ShaderMaterial
            .register_type::<components::ShaderMaterial>()
            // Keep PixelOutlineMaterial for chase state highlight (still used)
            // 保留 PixelOutlineMaterial 用于追逐状态高亮（仍在使用）
            .add_plugins(Material2dPlugin::<
                custom_sprite_material::PixelOutlineMaterial,
            >::default())
            // Add DynamicMaterial2d plugin for runtime shader selection
            // 添加 DynamicMaterial2d 插件用于运行时着色器选择
            .add_plugins(dynamic_material::DynamicMaterial2dPlugin)
            // Add reconciliation plugin for declarative view updates
            // 添加协调插件用于声明式视图更新
            .add_plugins(ViewReconciliationPlugin)
            .init_resource::<ron_view::ViewGlobalTriggerConfig>()
            .init_resource::<UIInteractiveStateTracker>()
            .init_resource::<StateTransitionTracker>()
            // Register SpawnViewRequest message
            // 注册 SpawnViewRequest 消息
            .add_message::<SpawnViewRequest>()
            // Register DespawnViewRequest message
            // 注册 DespawnViewRequest 消息
            .add_message::<DespawnViewRequest>()
            .add_systems(Startup, procedural_textures::init_procedural_textures)
            // Handle SpawnViewRequest messages
            // 处理 SpawnViewRequest 消息
            .add_systems(Update, handle_spawn_view_request_system)
            // Handle DespawnViewRequest messages
            // 处理 DespawnViewRequest 消息
            .add_systems(Update, handle_despawn_view_request_system)
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
                // New shader material update system (replaces old HP bar systems)
                // Must run AFTER setup_shader_materials_system to detect newly added materials
                // 新的着色器材质更新系统（取代旧的 HP 条系统）
                // 必须在 setup_shader_materials_system 之后运行以检测新添加的材质
                ron_view::update_shader_materials_system
                    .run_if(resource_exists::<procedural_textures::ProceduralTextures>)
                    .after(ron_view::setup_shader_materials_system),
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
                )
                    .in_set(ViewUpdate),
            )
            // spawn_ron_view_system has many parameters, add separately
            // spawn_ron_view_system 有很多参数，单独添加
            .add_systems(Update, spawn_ron_view_system.in_set(ViewUpdate))
            // Handle dynamically spawned views (via SpawnViewRequest)
            // 处理动态 spawn 的 View（通过 SpawnViewRequest）
            .add_systems(
                Update,
                ron_view::spawn_dynamic_view_system.in_set(ViewUpdate),
            )
            .add_systems(
                Update,
                (
                    ui_animation_init_system,
                    // Cleanup View-layer rules when ViewRoot entities are despawned
                    // 当 ViewRoot 实体销毁时清理 View 层规则
                    cleanup_view_rules_system,
                    // Process pending View rules when FRE assets finish loading
                    // 当 FRE 资产加载完成后处理待处理的 View 规则
                    // This must run after spawn_ron_view_system to ensure PendingViewRules is ready
                    // 必须在 spawn_ron_view_system 之后运行以确保 PendingViewRules 已准备好
                    process_pending_view_rules_system.after(spawn_ron_view_system),
                )
                    .in_set(ViewUpdate),
            )
            // Second group of UI systems (rendering/display)
            .add_systems(
                Update,
                (
                    ron_view::setup_shader_materials_system
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
            // Debug-only type registration for inspector
            // 仅 debug 模式下的类型注册（用于检查器）
            // Hot reload is handled by ViewReconciliationPlugin
            // 热重载由 ViewReconciliationPlugin 处理
            app.register_type::<ViewBox>()
                .register_type::<CameraAnchored>()
                .register_type::<ViewElement>()
                .register_type::<ViewRoot>()
                .register_type::<ViewElementHistory>()
                .register_type::<ElementState>();
        }
    }
}

/// System to handle SpawnViewRequest messages and spawn views.
///
/// 处理 SpawnViewRequest 消息并生成视图的系统。
fn handle_spawn_view_request_system(
    mut events: MessageReader<SpawnViewRequest>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for request in events.read() {
        info!("Spawning view from request: {}", request.path);

        // Load the view layout asset
        let handle: Handle<ViewLayoutAsset> = asset_server.load(&request.path);

        // Spawn a view root entity with the layout handle
        // The actual view spawning is handled by spawn_dynamic_view_system
        // Include Transform/Visibility components to support hierarchy
        commands.spawn((
            ron_view::HotReloadableViewRoot {
                layout_path: request.path.clone(),
                layout_handle: handle.clone(),
            },
            components::ViewRoot::new(request.path.clone()),
            components::ActiveView, // Mark as active for FRE input handling and dialogue text sync
            RonDrivenView,
            Name::new(format!("SpawnedView:{}", request.path)),
            // Required for hierarchy visibility propagation
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
        ));
    }
}

/// System to handle DespawnViewRequest messages and despawn views.
///
/// 处理 DespawnViewRequest 消息并销毁视图的系统。
fn handle_despawn_view_request_system(
    mut events: MessageReader<DespawnViewRequest>,
    mut commands: Commands,
    query: Query<(Entity, &components::ViewRoot), With<RonDrivenView>>,
) {
    for request in events.read() {
        let despawned_count = if let Some(ref path) = request.path {
            // Despawn specific view by path
            let mut count = 0;
            for (entity, view_root) in query.iter() {
                if view_root.layout_path == *path {
                    info!("Despawning view: {} (entity {:?})", path, entity);
                    commands.entity(entity).despawn();
                    count += 1;
                }
            }
            count
        } else {
            // Despawn all dynamically spawned views
            let mut count = 0;
            for (entity, view_root) in query.iter() {
                info!(
                    "Despawning view: {} (entity {:?})",
                    view_root.layout_path, entity
                );
                commands.entity(entity).despawn();
                count += 1;
            }
            count
        };

        if despawned_count == 0 {
            info!(
                "DespawnViewRequest: no views to despawn (path: {:?})",
                request.path
            );
        } else {
            info!(
                "DespawnViewRequest: despawned {} views (path: {:?})",
                despawned_count, request.path
            );
        }
    }
}
