//! # reload.rs
//!
//! # reload.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module provides hot-reload support for view_layout.ron files.
//! It uses a component-based approach where any entity with `HotReloadableViewRoot`
//! can have its view rebuilt when the corresponding asset is modified.
//!
//! 本模块提供 view_layout.ron 文件的热重载支持。
//! 采用基于组件的方式，任何带有 `HotReloadableViewRoot` 的实体
//! 在对应资源被修改时都可以重建其视图。

use bevy::asset::AssetEvent;
use bevy::ecs::prelude::MessageReader;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{TiledMap, TiledMapAsset};

use super::super::layout::ViewLayoutAsset;
use super::resources::{HotReloadableViewRoot, PendingViewReloads, RonDrivenView, ViewLayoutHandle};
use crate::core::map_property_schema::{get_string_property, keys, validate_map_properties};
use crate::core::sprite::params::SpriteParams;
use crate::extra::debug::DebugCamera;

/// Load view layout from Tiled map properties (fallback for states.ron view_layout).
/// This system only sets ViewLayoutHandle if states.ron doesn't already define view_layout.
///
/// NOTE: This system does NOT trigger hot reload. It only preloads
/// the ViewLayoutHandle so it's ready when the UI state is entered.
///
/// 从 Tiled 地图属性加载视图布局（作为 states.ron view_layout 的回退）。
/// 此系统仅在 states.ron 未定义 view_layout 时设置 ViewLayoutHandle。
///
/// 注意：此系统不触发热重载。它仅预加载
/// ViewLayoutHandle 以便在进入 UI 状态时准备就绪。
pub fn update_view_from_map_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tiled_maps_query: Query<&TiledMap>,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
    existing_layout_handle: Option<Res<ViewLayoutHandle>>,
    mut current_view_path: Local<Option<String>>,
    mut validated_maps: Local<std::collections::HashSet<bevy::asset::AssetId<TiledMapAsset>>>,
) {
    // Skip if ViewLayoutHandle already exists (set by backpack_state_transition_system)
    // 如果 ViewLayoutHandle 已存在（由 backpack_state_transition_system 设置），则跳过
    if existing_layout_handle.is_some() {
        return;
    }

    for tiled_map in tiled_maps_query.iter() {
        if let Some(map_asset) = tiled_map_assets.get(&tiled_map.0) {
            // Validate map properties once per map load
            let map_id = tiled_map.0.id();
            if !validated_maps.contains(&map_id) {
                validate_map_properties(&map_asset.map.properties);
                validated_maps.insert(map_id);
            }

            // Load view layout from map property (using schema key constant)
            // This is a PRELOAD, not a hot reload trigger
            // 这是预加载，不是热重载触发
            if let Some(path) = get_string_property(&map_asset.map.properties, keys::BACKPACK_UI)
                && current_view_path.as_deref() != Some(path)
            {
                let path_owned = path.to_string();
                info!(
                    "[update_view_from_map] Map property '{}': preloading view layout from '{}'",
                    keys::BACKPACK_UI,
                    path_owned
                );
                *current_view_path = Some(path_owned.clone());

                let handle = asset_server.load(&path_owned);

                commands.insert_resource(ViewLayoutHandle {
                    handle,
                    last_modified: None,
                    path: path_owned,
                });
            }
        }
    }
}

/// Watch for view layout asset changes and mark entities for reload.
/// This system monitors all `ViewLayoutAsset` modifications and updates
/// the `PendingViewReloads` resource accordingly.
///
/// 监视视图布局资源变化并标记实体进行重载。
/// 此系统监控所有 `ViewLayoutAsset` 的修改并相应更新
/// `PendingViewReloads` 资源。
pub fn watch_view_layout_changes_system(
    mut events: MessageReader<AssetEvent<ViewLayoutAsset>>,
    mut pending_reloads: ResMut<PendingViewReloads>,
    hot_reload_roots: Query<&HotReloadableViewRoot>,
) {
    for event in events.read() {
        if let AssetEvent::Modified { id } = event {
            // Check if any hot-reloadable root uses this asset
            let mut has_matching_root = false;
            for root in hot_reload_roots.iter() {
                if root.layout_handle.id() == *id {
                    has_matching_root = true;
                    break;
                }
            }

            if has_matching_root {
                info!(
                    "[Hot Reload] ViewLayoutAsset {:?} modified, marking for reload",
                    id
                );
                pending_reloads.mark_for_reload(*id);
            }
        }
    }
}

/// Rebuild views for entities whose layout assets have been modified.
/// This system processes all pending reloads and rebuilds the view hierarchy
/// for each affected entity.
///
/// 重建布局资源已被修改的实体的视图。
/// 此系统处理所有待处理的重载并为每个受影响的实体重建视图层级。
#[allow(clippy::too_many_arguments)]
pub fn rebuild_pending_views_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut pending_reloads: ResMut<PendingViewReloads>,
    view_layouts: Res<Assets<ViewLayoutAsset>>,
    animation_assets: Res<Assets<crate::core::character_asset::AnimationConfigAsset>>,
    hot_reload_roots: Query<(Entity, &HotReloadableViewRoot)>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<DebugCamera>)>,
    mut sprite_params: SpriteParams,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
    layered_db: Res<bevy_fact_rule_event::LayeredFactDatabase>,
    item_registry: Res<crate::core::item::ItemRegistry>,
    ron_view_query: Query<(Entity, &ChildOf), With<RonDrivenView>>,
) {
    if !pending_reloads.has_pending() {
        return;
    }

    use super::parsing::PlayerDataView;
    let player_data = PlayerDataView::new(&layered_db);

    let Ok(camera_transform) = camera_query.single() else {
        warn!("[Hot Reload] No Camera2d found for view rebuild!");
        return;
    };

    // Take all pending reloads
    let pending_ids = pending_reloads.take_all();
    let mut rebuilt_count = 0;

    for (root_entity, hot_reload_root) in hot_reload_roots.iter() {
        let asset_id = hot_reload_root.layout_handle.id();

        // Check if this root's asset was modified
        if !pending_ids.contains(&asset_id) {
            continue;
        }

        // Get the layout asset
        let Some(view_layout) = view_layouts.get(&hot_reload_root.layout_handle) else {
            warn!(
                "[Hot Reload] ViewLayout not loaded for entity {:?}, path: {}",
                root_entity, hot_reload_root.layout_path
            );
            continue;
        };

        info!(
            "[Hot Reload] Rebuilding view for entity {:?}, path: {}",
            root_entity, hot_reload_root.layout_path
        );

        // Despawn old view children (RonDrivenView entities that are children of this root)
        let mut despawn_count = 0;
        for (ron_entity, parent) in ron_view_query.iter() {
            if parent.parent() == root_entity {
                despawn_entity_tree(&mut commands, ron_entity);
                despawn_count += 1;
            }
        }

        if despawn_count > 0 {
            info!(
                "[Hot Reload] Despawned {} old view entities for root {:?}",
                despawn_count, root_entity
            );
        }

        // Spawn new view
        super::spawn::spawn_ron_view_for_entity(
            &mut commands,
            &asset_server,
            root_entity,
            view_layout,
            camera_transform,
            &mut sprite_params,
            &animation_assets,
            &mortar_strings,
            &player_data,
            &item_registry,
            &hot_reload_root.layout_path,
        );

        rebuilt_count += 1;
    }

    if rebuilt_count > 0 {
        info!(
            "[Hot Reload] View hot reload complete! Rebuilt {} views",
            rebuilt_count
        );
    }
}

/// Despawn an entity and all its children recursively.
///
/// 递归销毁实体及其所有子实体。
fn despawn_entity_tree(commands: &mut Commands, root: Entity) {
    commands.queue(move |world: &mut World| {
        let mut stack = vec![root];
        while let Some(entity) = stack.pop() {
            if let Ok(entity_ref) = world.get_entity(entity)
                && let Some(children) = entity_ref.get::<Children>()
            {
                for child in children.iter() {
                    stack.push(child);
                }
            }

            let _ = world.despawn(entity);
        }
    });
}
