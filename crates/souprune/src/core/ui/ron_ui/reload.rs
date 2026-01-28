use bevy::asset::AssetEvent;
use bevy::ecs::prelude::MessageReader;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{TiledMap, TiledMapAsset, tiled};

use super::super::components::RonUI;
use super::super::layout::ViewLayoutAsset;
use super::resources::{RonDrivenUI, UILayoutHandle, UILayoutWatcher};
use crate::core::sprite::params::SpriteParams;

pub fn update_ui_from_map_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tiled_maps_query: Query<&TiledMap>,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
    mut watcher: Option<ResMut<UILayoutWatcher>>,
    mut current_ui_path: Local<Option<String>>,
) {
    for tiled_map in tiled_maps_query.iter() {
        if let Some(map_asset) = tiled_map_assets.get(&tiled_map.0)
            && let Some(property_value) = map_asset.map.properties.get("backpack_ui")
            && let tiled::PropertyValue::StringValue(path) = property_value
            && current_ui_path.as_deref() != Some(path)
        {
            info!("Switching backpack UI to: {}", path);
            *current_ui_path = Some(path.clone());

            let handle = asset_server.load(path.clone());

            commands.insert_resource(UILayoutHandle {
                handle,
                last_modified: None,
            });

            if let Some(ref mut w) = watcher {
                w.pending_reload = true;
            } else {
                let mut w = UILayoutWatcher::new();
                w.pending_reload = true;
                commands.insert_resource(w);
            }
        }
    }
}

pub fn watch_ui_layout_changes_system(
    mut events: MessageReader<AssetEvent<ViewLayoutAsset>>,
    ui_layout_handle: Option<Res<UILayoutHandle>>,
    mut watcher: Option<ResMut<UILayoutWatcher>>,
    mut commands: Commands,
) {
    let Some(ui_layout_handle) = ui_layout_handle else {
        return;
    };

    for event in events.read() {
        if let AssetEvent::Modified { id } = event
            && *id == ui_layout_handle.handle.id()
        {
            info!("[Hot Reload] RON UI asset modified, triggering reload...");
            if let Some(ref mut w) = watcher {
                w.pending_reload = true;
            } else {
                let mut w = UILayoutWatcher::new();
                w.pending_reload = true;
                commands.insert_resource(w);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn rebuild_reloaded_ui_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_layout_handle: Option<Res<UILayoutHandle>>,
    mut watcher: Option<ResMut<UILayoutWatcher>>,
    ui_layouts: Res<Assets<ViewLayoutAsset>>,
    animation_assets: Res<Assets<crate::core::character_asset::AnimationConfigAsset>>,
    overworld_ui_query: Query<(Entity, &RonUI), Without<RonDrivenUI>>,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut sprite_params: SpriteParams,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
    player_data: Res<crate::core::data::PlayerData>,
    item_registry: Res<crate::core::item::ItemRegistry>,
    ron_ui_query: Query<Entity, With<RonDrivenUI>>,
) {
    let Some(ref mut watcher) = watcher else {
        return;
    };

    if !watcher.pending_reload {
        return;
    }

    let Some(handle) = ui_layout_handle else {
        return;
    };

    let Some(ui_layout) = ui_layouts.get(&handle.handle) else {
        return;
    };

    let has_target = overworld_ui_query.iter().any(|_| true);

    if !has_target {
        debug!("RON UI hot reload pending - no UI entity active, will retry rebuild");
        return;
    }

    info!("Asset loaded! Rebuilding UI...");

    let Ok(camera_transform) = camera_query.single() else {
        warn!("No Camera2d found for UI rebuild!");
        watcher.pending_reload = false;
        return;
    };

    // Despawn old UI first (only now that we know we're rebuilding)
    //
    // 首先 despawn 旧 UI（仅当我们知道正在重建时）
    let despawn_count = ron_ui_query.iter().count();
    if despawn_count > 0 {
        info!(
            "Despawning {} old UI entities before rebuild",
            despawn_count
        );
        for entity in ron_ui_query.iter() {
            despawn_entity_tree(&mut commands, entity);
        }
    }

    let mut rebuilt_count = 0;
    for (ui_entity, _ron_ui) in overworld_ui_query.iter() {
        super::spawn::spawn_ron_ui_for_entity(
            &mut commands,
            &asset_server,
            ui_entity,
            ui_layout,
            camera_transform,
            &mut sprite_params,
            &animation_assets,
            &mortar_strings,
            &player_data,
            &item_registry,
        );
        rebuilt_count += 1;
    }

    watcher.pending_reload = rebuilt_count == 0;
    info!(
        "RON UI hot reload complete! Rebuilt {} UI entities",
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
