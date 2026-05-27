//! Dynamic View spawning system.
//!
//! 动态 View 生成系统。

use super::{
    FreSystemParams, camera_relative_parent_for_view, camera_relative_view_offset,
    layout_requests_focus_scope, layout_viewport_size, required_pre_spawn_fre_files_loaded,
    spawn_ron_view_for_entity,
};
use crate::core::game_action::GameFreAsset;
use crate::core::sprite::params::SpriteParams;
use crate::core::view::camera::select_view_camera_target;
use crate::core::view::components::*;
use crate::core::view::layout::*;
use crate::core::view::ron_view::parsing::{
    DataPathResolvers, ExprFunctionResolvers, PlayerDataView,
};
use crate::core::view::ron_view::resources::{HotReloadableViewRoot, RonDrivenView, ViewGenerated};
use crate::core::view::spatial::{ViewSpatialRoot, spatial_root_transform};
use crate::extra::debug::DebugCamera;
use bevy::prelude::*;
use bevy_fact_rule_event::LayeredFactDatabase;

/// Unified system to spawn all dynamic Views.
/// All View spawning goes through SpawnViewRequest → this system.
///
/// 统一的动态 View 生成系统。
/// 所有 View 生成都通过 SpawnViewRequest → 此系统。
pub fn spawn_dynamic_view_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    view_layouts: Res<Assets<ViewLayoutAsset>>,
    animation_assets: Res<Assets<crate::core::character_asset::AnimationConfigAsset>>,
    fre_assets: Res<Assets<GameFreAsset>>,
    // Query for views with HotReloadableViewRoot + RonDrivenView but not yet generated
    // 查询有 HotReloadableViewRoot + RonDrivenView 但尚未生成的 View
    mut dynamic_view_query: Query<
        (
            Entity,
            &mut HotReloadableViewRoot,
            &ViewRoot,
            Option<&PendingViewData>,
        ),
        (
            With<RonDrivenView>,
            Without<ViewGenerated>,
            Without<ViewBox>,
        ),
    >,
    main_2d_camera_query: Query<
        (Entity, &Camera, &Projection),
        (
            With<Camera2d>,
            With<crate::core::camera::MainGameCamera>,
            Without<DebugCamera>,
        ),
    >,
    main_3d_camera_query: Query<
        (Entity, &Camera),
        (
            With<Camera3d>,
            With<crate::core::camera::MainGameCamera>,
            Without<DebugCamera>,
        ),
    >,
    mut sprite_params: SpriteParams,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
    layered_db: Res<LayeredFactDatabase>,
    mut fre_params: FreSystemParams,
    data_resolvers: Option<Res<DataPathResolvers>>,
    expr_func_resolvers: Option<Res<ExprFunctionResolvers>>,
) {
    let player_data = PlayerDataView::new(&layered_db)
        .with_resolvers(data_resolvers.as_deref(), None)
        .with_expr_functions(expr_func_resolvers.as_deref());

    for (view_entity, mut hot_reload_root, _view_root, pending_view_data) in
        dynamic_view_query.iter_mut()
    {
        // Check if asset is loaded
        let Some(view_layout) = view_layouts.get(&hot_reload_root.layout_handle) else {
            trace!(
                "[spawn_dynamic_view] Waiting for asset to load: {}",
                hot_reload_root.layout_path
            );
            continue;
        };

        // If there are pending bindings, wait for all FRE assets to load
        if let Some(pvd) = pending_view_data {
            let all_loaded = pvd.fre_handles.iter().all(|h| fre_assets.get(h).is_some());
            if !all_loaded {
                trace!(
                    "[spawn_dynamic_view] Waiting for FRE binding assets: {}",
                    hot_reload_root.layout_path
                );
                continue;
            }
        }

        if !required_pre_spawn_fre_files_loaded(
            view_layout,
            &asset_server,
            &fre_assets,
            &mut hot_reload_root,
        ) {
            trace!(
                "[spawn_dynamic_view] Waiting for required FRE assets: {}",
                hot_reload_root.layout_path
            );
            continue;
        }

        let Some(camera_target) = select_view_camera_target(
            view_layout,
            main_2d_camera_query.iter(),
            main_3d_camera_query.iter(),
        ) else {
            warn!("[spawn_dynamic_view] No active MainGameCamera found for view spawning!");
            continue;
        };

        info!(
            "[spawn_dynamic_view] Spawning view: {}, entity={:?}",
            hot_reload_root.layout_path, view_entity
        );

        // Get bindings from PendingViewData if present
        let bindings = pending_view_data.map(|pvd| &pvd.bindings);
        let visible_size = camera_target.visible_size;

        spawn_ron_view_for_entity(
            &mut commands,
            &asset_server,
            view_entity,
            view_layout,
            &mut sprite_params,
            &animation_assets,
            &fre_assets,
            &mortar_strings,
            &player_data,
            &hot_reload_root.layout_path,
            &hot_reload_root.pre_spawn_events,
            bindings,
            &layered_db,
            &mut fre_params.rule_registry,
            &mut fre_params.enum_registry,
            layout_viewport_size(view_layout, visible_size),
        );

        // Camera-relative Views parent the root entity to the selected camera.
        // Explicit world-space Views stay as standalone world entities.
        if let Some(ViewSpaceDef::World3dPlane(plane)) = &view_layout.space {
            commands.entity(view_entity).insert((
                spatial_root_transform(plane),
                ViewSpatialRoot {
                    plane: plane.as_ref().clone(),
                },
            ));
        } else if let Some(camera_parent) =
            camera_relative_parent_for_view(view_layout, camera_target.camera_relative_parent)
        {
            commands.entity(view_entity).insert((
                Transform::from_translation(
                    camera_relative_view_offset(view_layout, visible_size).extend(0.0),
                ),
                ChildOf(camera_parent),
            ));
        }

        // 自动推断 ViewFocusScope：有 requires（FRE 规则声明）或 bindings（外部数据绑定）
        // → 进入焦点栈，由生命周期系统派生唯一 ActiveView
        if !view_layout.requires.is_empty()
            || pending_view_data.is_some()
            || layout_requests_focus_scope(view_layout)
        {
            commands.entity(view_entity).insert(ViewFocusScope);
        }

        // Add ViewGenerated and ReconciliationEnabled; remove PendingViewData
        commands.entity(view_entity).insert((
            ViewGenerated,
            crate::core::view::reconcile::ReconciliationEnabled,
        ));
        commands.entity(view_entity).remove::<PendingViewData>();

        info!(
            "[spawn_dynamic_view] Added ViewGenerated to entity {:?}",
            view_entity
        );
    }
}
