//! # reload.rs
//!
//! # reload.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module provides view layout loading from Tiled map properties.
//! Hot reload is now handled by the unified reconciliation system in reconcile/system.rs.
//!
//! 本模块提供从 Tiled 地图属性加载视图布局的功能。
//! 热重载现在由 reconcile/system.rs 中的统一协调系统处理。

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{TiledMap, TiledMapAsset};

use crate::core::map_property_schema::validate_map_properties;

// Debug-only imports for hot reload system
#[cfg(feature = "debug")]
use super::resources::{HotReloadableViewRoot, PendingViewReloads};
#[cfg(feature = "debug")]
use crate::core::view::layout::ViewLayoutAsset;
#[cfg(feature = "debug")]
use bevy::asset::AssetEvent;
#[cfg(feature = "debug")]
use bevy::ecs::prelude::MessageReader;

/// Validate Tiled map properties once per map load.
/// Previously also preloaded ViewLayoutHandle, now only performs validation.
///
/// 每次地图加载时验证 Tiled 地图属性。
/// 之前还会预加载 ViewLayoutHandle，现在仅执行验证。
pub fn validate_map_properties_system(
    tiled_maps_query: Query<&TiledMap>,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
    mut validated_maps: Local<std::collections::HashSet<bevy::asset::AssetId<TiledMapAsset>>>,
) {
    for tiled_map in tiled_maps_query.iter() {
        if let Some(map_asset) = tiled_map_assets.get(&tiled_map.0) {
            let map_id = tiled_map.0.id();
            if !validated_maps.contains(&map_id) {
                validate_map_properties(&map_asset.map.properties);
                validated_maps.insert(map_id);
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
) {
    for event in events.read() {
        if let AssetEvent::Modified { id } = event {
            // Check if any hot-reloadable root uses this asset
            for root in hot_reload_roots.iter() {
                if root.layout_handle.id() == *id {
                    debug!(
                        "[Hot Reload] ViewLayoutAsset {:?} modified, marking for incremental update",
                        id
                    );
                    pending_reloads.mark_for_reload(*id);
                    break;
                }
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
/// 5. Updates DynamicViewElement stored definitions (to persist through fact changes)
///
/// 增量重载系统 - 更新现有视图元素而不销毁它们（仅 debug 模式）。
///
/// 此系统：
/// 1. 通过 ViewElement.full_name 匹配现有实体和新定义
/// 2. 更新 VisibleWhen 表达式
/// 3. 更新 Transform 属性
/// 4. 更新 Sprite 颜色/翻转属性
/// 5. 更新 DynamicViewElement 存储的定义（以在 fact 变化时保持）
#[cfg(feature = "debug")]
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
    mut dynamic_element_query: Query<&mut crate::core::view::components::DynamicViewElement>,
    mut camera_anchored_query: Query<&mut super::super::components::CameraAnchored>,
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
            "[Hot Reload] Processing view '{}' (namespace: {})",
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
            if let Some(new_visible_when) = &node_def.visible_when
                && let Ok(mut visible_when) = visible_when_query.get_mut(entity)
                && visible_when.expression != *new_visible_when
            {
                debug!(
                    "[Hot Reload] visible_when '{}': '{}' -> '{}'",
                    view_element.full_name, visible_when.expression, new_visible_when
                );
                visible_when.expression = new_visible_when.clone();
                updated_count += 1;
            }

            // Update CameraAnchored offset for ViewBox nodes
            // ViewBox 节点的 CameraAnchored 偏移更新
            if let Some(ui_logic) = &node_def.view_box
                && let Ok(mut camera_anchored) = camera_anchored_query.get_mut(entity)
            {
                let new_offset = super::super::layout::serde_types::serializable_vec3_to_static(
                    &ui_logic.offset,
                );
                if camera_anchored.offset != new_offset {
                    debug!(
                        "[Hot Reload] CameraAnchored offset '{}': {:?} -> {:?}",
                        view_element.full_name, camera_anchored.offset, new_offset
                    );
                    camera_anchored.offset = new_offset;
                    updated_count += 1;
                }
            }

            // Update Transform from sprite def (only for standalone sprites, not ViewBox)
            // ViewBox entities use CameraAnchored + view_box.offset, not sprite.transform
            // 从 sprite def 更新 Transform（仅适用于独立 sprite，不适用于 ViewBox）
            // ViewBox 实体使用 CameraAnchored + view_box.offset，而不是 sprite.transform
            if let Some(sprite_def) = &node_def.sprite {
                // Skip Transform update if this node has view_box (it's a ViewBox)
                // ViewBox position is managed separately via CameraAnchored component
                // 如果此节点有 view_box（它是 ViewBox），则跳过 Transform 更新
                // ViewBox 位置通过 CameraAnchored 组件单独管理
                let is_viewbox = node_def.view_box.is_some();
                let mut sprite_updated = false;

                if !is_viewbox
                    && let Some(t_def) = &sprite_def.transform
                    && let Ok(mut transform) = transform_query.get_mut(entity)
                {
                    // First update scale if present (needed for pivot calculation)
                    // 首先更新缩放（pivot计算需要）
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
                            debug!(
                                "[Hot Reload] scale '{}': {:?} -> {:?}",
                                view_element.full_name, transform.scale, new_scale
                            );
                            transform.scale = new_scale;
                            updated_count += 1;
                            sprite_updated = true;
                        }
                    }
                    // Update rotation if present (needed for pivot calculation)
                    // 更新旋转（pivot计算需要）
                    if let Some(rot) = t_def.rotation {
                        let new_rotation = Quat::from_rotation_z(rot.to_radians());
                        if transform.rotation != new_rotation {
                            debug!(
                                "[Hot Reload] rotation '{}': {:?} -> {:?}",
                                view_element.full_name, transform.rotation, new_rotation
                            );
                            transform.rotation = new_rotation;
                            updated_count += 1;
                            sprite_updated = true;
                        }
                    }
                    // Update translation with pivot offset applied
                    // 更新translation并应用pivot偏移
                    if let Some(trans) = &t_def.translation {
                        let base_translation = Vec3::new(
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
                        // Apply pivot offset if sprite has pivot defined
                        // (same logic as spawn.rs for HP bar and other sprites)
                        // 如果sprite定义了pivot则应用偏移
                        // （与spawn.rs中HP bar和其他sprite的逻辑相同）
                        let new_translation = if let Some(pivot) = &sprite_def.pivot {
                            let (pivot_x, pivot_y) =
                                super::super::layout::serde_types::vec2_tuple_to_static(pivot);
                            let shift_x = (0.5 - pivot_x) * transform.scale.x;
                            let shift_y = (0.5 - pivot_y) * transform.scale.y;
                            let shift = transform.rotation * Vec3::new(shift_x, shift_y, 0.0);
                            base_translation + shift
                        } else {
                            base_translation
                        };
                        if transform.translation != new_translation {
                            debug!(
                                "[Hot Reload] translation '{}': {:?} -> {:?}",
                                view_element.full_name, transform.translation, new_translation
                            );
                            transform.translation = new_translation;
                            updated_count += 1;
                            sprite_updated = true;
                        }
                    }
                } // end if !is_viewbox

                // Update Sprite properties (color, flip)
                if let Ok(mut sprite) = sprite_query.get_mut(entity) {
                    // Update color
                    if let Some(color_def) = &sprite_def.color {
                        let (r, g, b, a) =
                            super::super::layout::serde_types::color_tuple_to_static(color_def);
                        let new_color = Color::srgba(r, g, b, a);
                        if sprite.color != new_color {
                            debug!(
                                "[Hot Reload] color '{}': {:?} -> {:?}",
                                view_element.full_name, sprite.color, new_color
                            );
                            sprite.color = new_color;
                            updated_count += 1;
                            sprite_updated = true;
                        }
                    }
                    // Update flip_x
                    if sprite.flip_x != sprite_def.flip_x {
                        debug!(
                            "[Hot Reload] flip_x '{}': {} -> {}",
                            view_element.full_name, sprite.flip_x, sprite_def.flip_x
                        );
                        sprite.flip_x = sprite_def.flip_x;
                        updated_count += 1;
                        sprite_updated = true;
                    }
                    // Update flip_y
                    if sprite.flip_y != sprite_def.flip_y {
                        debug!(
                            "[Hot Reload] flip_y '{}': {} -> {}",
                            view_element.full_name, sprite.flip_y, sprite_def.flip_y
                        );
                        sprite.flip_y = sprite_def.flip_y;
                        updated_count += 1;
                        sprite_updated = true;
                    }
                }

                // Update DynamicViewElement stored definition only if any property changed
                // 只有当有属性变化时才更新 DynamicViewElement 存储的定义
                if sprite_updated
                    && let Ok(mut dynamic_elem) = dynamic_element_query.get_mut(entity)
                {
                    // Preprocess sprite_def if this is a repeat element
                    let processed_sprite_def = if let Some(ctx) = &repeat_ctx {
                        super::parsing::preprocess_sprite_def_for_repeat(sprite_def, ctx)
                    } else {
                        sprite_def.clone()
                    };
                    dynamic_elem.sprite_def = Some(processed_sprite_def);
                    debug!(
                        "[Hot Reload] Updated DynamicViewElement.sprite_def for '{}'",
                        view_element.full_name
                    );
                }
            }
        }
    }

    if updated_count > 0 {
        info!("[Hot Reload] Updated {} properties", updated_count);
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
