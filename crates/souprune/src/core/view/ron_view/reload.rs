use bevy::asset::AssetEvent;
use bevy::ecs::prelude::MessageReader;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{TiledMap, TiledMapAsset, tiled};

use super::super::components::InteractiveLayer;
use super::super::layout::ViewLayoutAsset;
use super::super::lifecycle::BackpackViewRoot;
use super::resources::{RonDrivenView, ViewLayoutHandle, ViewLayoutWatcher};
use crate::core::sprite::params::SpriteParams;

/// Load view layout from Tiled map properties.
///
/// 从 Tiled 地图属性加载视图布局。
pub fn update_view_from_map_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tiled_maps_query: Query<&TiledMap>,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
    mut watcher: Option<ResMut<ViewLayoutWatcher>>,
    mut current_view_path: Local<Option<String>>,
) {
    for tiled_map in tiled_maps_query.iter() {
        if let Some(map_asset) = tiled_map_assets.get(&tiled_map.0)
            && let Some(property_value) = map_asset.map.properties.get("backpack_ui")
            && let tiled::PropertyValue::StringValue(path) = property_value
            && current_view_path.as_deref() != Some(path)
        {
            info!("Switching backpack view to: {}", path);
            *current_view_path = Some(path.clone());

            let handle = asset_server.load(path.clone());

            commands.insert_resource(ViewLayoutHandle {
                handle,
                last_modified: None,
                path: path.clone(),
            });

            if let Some(ref mut w) = watcher {
                w.pending_reload = true;
            } else {
                let mut w = ViewLayoutWatcher::new();
                w.pending_reload = true;
                commands.insert_resource(w);
            }
        }
    }
}

/// Watch for view layout asset changes for hot reload.
///
/// 监视视图布局资源变化以支持热重载。
pub fn watch_view_layout_changes_system(
    mut events: MessageReader<AssetEvent<ViewLayoutAsset>>,
    view_layout_handle: Option<Res<ViewLayoutHandle>>,
    mut watcher: Option<ResMut<ViewLayoutWatcher>>,
    mut commands: Commands,
) {
    let Some(view_layout_handle) = view_layout_handle else {
        return;
    };

    for event in events.read() {
        if let AssetEvent::Modified { id } = event
            && *id == view_layout_handle.handle.id()
        {
            info!("[Hot Reload] RON view asset modified, triggering reload...");
            if let Some(ref mut w) = watcher {
                w.pending_reload = true;
            } else {
                let mut w = ViewLayoutWatcher::new();
                w.pending_reload = true;
                commands.insert_resource(w);
            }
        }
    }
}

/// Rebuild view layout when hot reload is triggered.
///
/// 热重载触发时重建视图布局。
#[allow(clippy::too_many_arguments)]
pub fn rebuild_reloaded_view_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    view_layout_handle: Option<Res<ViewLayoutHandle>>,
    mut watcher: Option<ResMut<ViewLayoutWatcher>>,
    view_layouts: Res<Assets<ViewLayoutAsset>>,
    animation_assets: Res<Assets<crate::core::character_asset::AnimationConfigAsset>>,
    backpack_root_query: Query<Entity, With<BackpackViewRoot>>,
    interactive_layer_query: Query<Entity, With<InteractiveLayer>>,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut sprite_params: SpriteParams,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
    player_data: Res<crate::core::data::PlayerData>,
    item_registry: Res<crate::core::item::ItemRegistry>,
    ron_view_query: Query<Entity, With<RonDrivenView>>,
) {
    let Some(ref mut watcher) = watcher else {
        return;
    };

    if !watcher.pending_reload {
        return;
    }

    let Some(handle) = view_layout_handle else {
        return;
    };

    let Some(view_layout) = view_layouts.get(&handle.handle) else {
        return;
    };

    // Check if we have a backpack root or interactive layers as our target
    let has_target = !backpack_root_query.is_empty() || !interactive_layer_query.is_empty();

    if !has_target {
        debug!("RON view hot reload pending - no view entity active, will retry rebuild");
        return;
    }

    info!("Asset loaded! Rebuilding view...");

    let Ok(camera_transform) = camera_query.single() else {
        warn!("No Camera2d found for view rebuild!");
        watcher.pending_reload = false;
        return;
    };

    // Despawn old view first (only now that we know we're rebuilding)
    //
    // 首先 despawn 旧视图（仅当我们知道正在重建时）
    let despawn_count = ron_view_query.iter().count();
    if despawn_count > 0 {
        info!(
            "Despawning {} old view entities before rebuild",
            despawn_count
        );
        for entity in ron_view_query.iter() {
            despawn_entity_tree(&mut commands, entity);
        }
    }

    let mut rebuilt_count = 0;

    // Find backpack root to use as parent for the rebuilt view
    if let Ok(view_entity) = backpack_root_query.single() {
        super::spawn::spawn_ron_view_for_entity(
            &mut commands,
            &asset_server,
            view_entity,
            view_layout,
            camera_transform,
            &mut sprite_params,
            &animation_assets,
            &mortar_strings,
            &player_data,
            &item_registry,
            &handle.path,
        );
        rebuilt_count += 1;
    }

    watcher.pending_reload = rebuilt_count == 0;
    info!(
        "RON view hot reload complete! Rebuilt {} view entities",
        rebuilt_count
    );
}

fn despawn_entity_tree(commands: &mut Commands, root: Entity) {
    // Schedule recursive despawn to avoid borrowing the world inside the system.
    //
    // 调度递归 despawn 以避免在系统内借用 world。
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
