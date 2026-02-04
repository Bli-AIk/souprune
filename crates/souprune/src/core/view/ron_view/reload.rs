//! # reload.rs
//!
//! # reload.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module provides hot-reload support for view_layout.ron files.
//! Uses an incremental update approach where only modified properties are updated
//! without destroying and rebuilding the entire view hierarchy.
//!
//! 本模块提供 view_layout.ron 文件的热重载支持。
//! 采用增量更新方式，仅更新修改的属性，
//! 不需要销毁并重建整个视图层级。

#[cfg(feature = "debug")]
use bevy::asset::AssetEvent;
#[cfg(feature = "debug")]
use bevy::ecs::prelude::MessageReader;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{TiledMap, TiledMapAsset};

use super::super::layout::ViewLayoutAsset;
use super::resources::ViewLayoutHandle;
#[cfg(feature = "debug")]
use super::resources::{HotReloadableViewRoot, PendingViewReloads};
use crate::core::map_property_schema::{get_string_property, keys, validate_map_properties};

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

/// Watch for view layout asset changes and mark entities for reload (debug only).
///
/// 监视视图布局资源变化并标记实体进行重载（仅 debug 模式）。
#[cfg(feature = "debug")]
pub fn watch_view_layout_changes_system(
    mut events: MessageReader<AssetEvent<ViewLayoutAsset>>,
    mut pending_reloads: ResMut<PendingViewReloads>,
    hot_reload_roots: Query<&HotReloadableViewRoot>,
    mut frame_counter: Local<u64>,
) {
    *frame_counter += 1;

    // Log every 300 frames (~5 seconds at 60fps) to confirm system is running
    if *frame_counter % 300 == 0 {
        let root_count = hot_reload_roots.iter().count();
        info!(
            "[Hot Reload] System running (frame {}), monitoring {} hot reload roots",
            *frame_counter, root_count
        );
    }

    for event in events.read() {
        info!("[Hot Reload] Received AssetEvent: {:?}", event);

        if let AssetEvent::Modified { id } = event {
            let root_count = hot_reload_roots.iter().count();
            info!(
                "[Hot Reload] ViewLayoutAsset {:?} modified, checking {} hot reload roots",
                id, root_count
            );

            // Check if any hot-reloadable root uses this asset
            let mut has_matching_root = false;
            for root in hot_reload_roots.iter() {
                info!(
                    "[Hot Reload] Checking root: handle_id={:?}, path={}",
                    root.layout_handle.id(),
                    root.layout_path
                );
                if root.layout_handle.id() == *id {
                    has_matching_root = true;
                    info!("[Hot Reload] Found matching root!");
                    break;
                }
            }

            if has_matching_root {
                info!(
                    "[Hot Reload] ViewLayoutAsset {:?} modified, marking for incremental update",
                    id
                );
                pending_reloads.mark_for_reload(*id);
            } else {
                info!(
                    "[Hot Reload] No matching hot reload root found for asset {:?}",
                    id
                );
            }
        }
    }
}

/// Incremental reload system - updates existing view elements without destroying them (debug only).
///
/// This system:
/// 1. Matches existing entities to new definitions by ViewElement.full_name
/// 2. Updates VisibleWhen expressions
/// 3. Updates Transform properties
/// 4. Updates Sprite color/flip properties
///
/// 增量重载系统 - 更新现有视图元素而不销毁它们（仅 debug 模式）。
///
/// 此系统：
/// 1. 通过 ViewElement.full_name 匹配现有实体和新定义
/// 2. 更新 VisibleWhen 表达式
/// 3. 更新 Transform 属性
/// 4. 更新 Sprite 颜色/翻转属性
#[cfg(feature = "debug")]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn incremental_reload_system(
    mut pending_reloads: ResMut<PendingViewReloads>,
    view_layouts: Res<Assets<ViewLayoutAsset>>,
    hot_reload_roots: Query<(Entity, &HotReloadableViewRoot)>,
    view_element_query: Query<(
        Entity,
        &crate::core::view::components::ViewElement,
        &ChildOf,
    )>,
    mut visible_when_query: Query<&mut crate::core::view::components::VisibleWhen>,
    mut transform_query: Query<&mut Transform>,
    mut sprite_query: Query<&mut Sprite>,
    children_query: Query<&Children>,
    layered_db: Res<bevy_fact_rule_event::LayeredFactDatabase>,
    view_root_query: Query<&crate::core::view::components::ViewRoot>,
) {
    if !pending_reloads.has_pending() {
        return;
    }

    let pending_ids = pending_reloads.take_all();
    let mut updated_count = 0;

    for (root_entity, hot_reload_root) in hot_reload_roots.iter() {
        let asset_id = hot_reload_root.layout_handle.id();

        if !pending_ids.contains(&asset_id) {
            continue;
        }

        let Some(view_layout) = view_layouts.get(&hot_reload_root.layout_handle) else {
            warn!(
                "[Hot Reload] ViewLayout not loaded for entity {:?}, path: {}",
                root_entity, hot_reload_root.layout_path
            );
            continue;
        };

        // Get namespace and local facts from ViewRoot
        let namespace = crate::core::view::components::ViewRoot::namespace_from_path(
            &hot_reload_root.layout_path,
        );

        // Get local facts from ViewRoot if available
        let local_facts = view_root_query
            .get(root_entity)
            .map(|vr| &vr.local_facts)
            .ok();

        info!(
            "[Hot Reload] Incremental update for view '{}' (namespace: {})",
            hot_reload_root.layout_path, namespace
        );

        // Build a map of node definitions by full_name (without repeat suffix)
        let mut node_defs_by_name: std::collections::HashMap<
            String,
            &super::super::layout::ViewNodeDef,
        > = std::collections::HashMap::new();
        collect_node_defs(&namespace, &view_layout.roots, &mut node_defs_by_name);

        // Create player data view for expression evaluation
        let player_data = if let Some(local_facts) = local_facts {
            super::parsing::PlayerDataView::with_local_facts(&layered_db, local_facts)
        } else {
            super::parsing::PlayerDataView::new(&layered_db)
        };

        // Find all ViewElement entities that are descendants of this root
        let descendants = collect_descendants(root_entity, &children_query);

        for descendant in descendants {
            let Ok((entity, view_element, _)) = view_element_query.get(descendant) else {
                continue;
            };

            // Look up the node definition and extract repeat index if applicable
            // For repeat elements (e.g., "namespace::EnemyHpBar_0"), try base name and extract index
            // 对于重复元素（如 "namespace::EnemyHpBar_0"），尝试基础名称并提取索引
            let (node_def, repeat_ctx) =
                if let Some(def) = node_defs_by_name.get(&view_element.full_name) {
                    (Some(*def), None)
                } else {
                    // Try to find base name by stripping _N suffix and create repeat context
                    // 尝试去掉 _N 后缀来查找基础名称并创建重复上下文
                    find_node_def_for_repeat_element_with_context(
                        &view_element.full_name,
                        &node_defs_by_name,
                    )
                };

            let Some(node_def) = node_def else {
                continue;
            };

            // Update VisibleWhen expression
            if let Some(new_visible_when) = &node_def.visible_when {
                if let Ok(mut visible_when) = visible_when_query.get_mut(entity) {
                    if visible_when.expression != *new_visible_when {
                        info!(
                            "[Hot Reload] Updating visible_when for '{}': '{}' -> '{}'",
                            view_element.full_name, visible_when.expression, new_visible_when
                        );
                        visible_when.expression = new_visible_when.clone();
                        updated_count += 1;
                    }
                }
            }

            // Update Transform from sprite def
            if let Some(sprite_def) = &node_def.sprite {
                if let Some(t_def) = &sprite_def.transform {
                    if let Ok(mut transform) = transform_query.get_mut(entity) {
                        if let Some(trans) = &t_def.translation {
                            let new_translation = Vec3::new(
                                super::parsing::evaluate_float_expr_with_repeat(
                                    &trans.0,
                                    &player_data,
                                    None,
                                    repeat_ctx.as_ref(),
                                ),
                                super::parsing::evaluate_float_expr_with_repeat(
                                    &trans.1,
                                    &player_data,
                                    None,
                                    repeat_ctx.as_ref(),
                                ),
                                super::parsing::evaluate_float_expr_with_repeat(
                                    &trans.2,
                                    &player_data,
                                    None,
                                    repeat_ctx.as_ref(),
                                ),
                            );
                            if transform.translation != new_translation {
                                info!(
                                    "[Hot Reload] Updating translation for '{}': {:?} -> {:?}",
                                    view_element.full_name, transform.translation, new_translation
                                );
                                transform.translation = new_translation;
                                updated_count += 1;
                            }
                        }
                        if let Some(scale) = &t_def.scale {
                            let new_scale = Vec3::new(
                                super::parsing::evaluate_float_expr_with_repeat(
                                    &scale.0,
                                    &player_data,
                                    None,
                                    repeat_ctx.as_ref(),
                                ),
                                super::parsing::evaluate_float_expr_with_repeat(
                                    &scale.1,
                                    &player_data,
                                    None,
                                    repeat_ctx.as_ref(),
                                ),
                                super::parsing::evaluate_float_expr_with_repeat(
                                    &scale.2,
                                    &player_data,
                                    None,
                                    repeat_ctx.as_ref(),
                                ),
                            );
                            if transform.scale != new_scale {
                                info!(
                                    "[Hot Reload] Updating scale for '{}': {:?} -> {:?}",
                                    view_element.full_name, transform.scale, new_scale
                                );
                                transform.scale = new_scale;
                                updated_count += 1;
                            }
                        }
                        if let Some(rot) = t_def.rotation {
                            let new_rotation = Quat::from_rotation_z(rot.to_radians());
                            if transform.rotation != new_rotation {
                                info!(
                                    "[Hot Reload] Updating rotation for '{}': {:?} -> {:?}",
                                    view_element.full_name, transform.rotation, new_rotation
                                );
                                transform.rotation = new_rotation;
                                updated_count += 1;
                            }
                        }
                    }
                }

                // Update Sprite properties (color, flip)
                if let Ok(mut sprite) = sprite_query.get_mut(entity) {
                    // Update color
                    if let Some(color_def) = &sprite_def.color {
                        let (r, g, b, a) =
                            super::super::layout::serde_types::color_tuple_to_static(color_def);
                        let new_color = Color::srgba(r, g, b, a);
                        if sprite.color != new_color {
                            info!(
                                "[Hot Reload] Updating color for '{}': {:?} -> {:?}",
                                view_element.full_name, sprite.color, new_color
                            );
                            sprite.color = new_color;
                            updated_count += 1;
                        }
                    }
                    // Update flip_x
                    if sprite.flip_x != sprite_def.flip_x {
                        info!(
                            "[Hot Reload] Updating flip_x for '{}': {} -> {}",
                            view_element.full_name, sprite.flip_x, sprite_def.flip_x
                        );
                        sprite.flip_x = sprite_def.flip_x;
                        updated_count += 1;
                    }
                    // Update flip_y
                    if sprite.flip_y != sprite_def.flip_y {
                        info!(
                            "[Hot Reload] Updating flip_y for '{}': {} -> {}",
                            view_element.full_name, sprite.flip_y, sprite_def.flip_y
                        );
                        sprite.flip_y = sprite_def.flip_y;
                        updated_count += 1;
                    }
                }
            }
        }
    }

    if updated_count > 0 {
        info!(
            "[Hot Reload] Incremental update complete! Updated {} properties",
            updated_count
        );
    }
}

/// Collect all node definitions into a map keyed by full_name.
///
/// 将所有节点定义收集到以 full_name 为键的映射中。
#[cfg(feature = "debug")]
fn collect_node_defs<'a>(
    namespace: &str,
    nodes: &'a [super::super::layout::ViewNodeDef],
    map: &mut std::collections::HashMap<String, &'a super::super::layout::ViewNodeDef>,
) {
    for node in nodes {
        if !node.name.is_empty() {
            let full_name = format!("{}::{}", namespace, node.name);
            map.insert(full_name, node);
        }
        // Recursively collect children
        collect_node_defs(namespace, &node.children, map);
    }
}

/// Find node definition for a repeat element by stripping _N suffix and return with repeat context.
/// For example, "namespace::EnemyHpBar_0" -> looks up "namespace::EnemyHpBar" and returns context with index 0
///
/// 通过去掉 _N 后缀来查找重复元素的节点定义，并返回重复上下文。
/// 例如，"namespace::EnemyHpBar_0" -> 查找 "namespace::EnemyHpBar" 并返回索引为 0 的上下文
#[cfg(feature = "debug")]
fn find_node_def_for_repeat_element_with_context<'a>(
    full_name: &str,
    map: &std::collections::HashMap<String, &'a super::super::layout::ViewNodeDef>,
) -> (
    Option<&'a super::super::layout::ViewNodeDef>,
    Option<super::parsing::RepeatContext>,
) {
    // Try to strip _N suffix (where N is a number)
    // 尝试去掉 _N 后缀（其中 N 是数字）
    if let Some(last_underscore) = full_name.rfind('_') {
        let suffix = &full_name[last_underscore + 1..];
        if let Ok(index) = suffix.parse::<usize>() {
            let base_name = &full_name[..last_underscore];
            if let Some(def) = map.get(base_name) {
                let ctx = super::parsing::RepeatContext::new(index);
                return (Some(*def), Some(ctx));
            }
        }
    }
    (None, None)
}

/// Collect all descendant entities of a root entity.
///
/// 收集根实体的所有后代实体。
#[cfg(feature = "debug")]
fn collect_descendants(root: Entity, children_query: &Query<&Children>) -> Vec<Entity> {
    let mut result = Vec::new();
    let mut stack = vec![root];

    while let Some(entity) = stack.pop() {
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                result.push(child);
                stack.push(child);
            }
        }
    }

    result
}
