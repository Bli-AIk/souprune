//! Runtime visibility evaluation system using `visible_when` expressions.
//!
//! 使用 `visible_when` 表达式的运行时可见性评估系统。
//!
//! This module provides the system that evaluates `visible_when` expressions
//! every frame and updates entity visibility accordingly.
//!
//! 本模块提供每帧评估 `visible_when` 表达式并相应更新实体可见性的系统。

use super::components::{InteractiveLayer, ViewRoot, VisibleWhen};
use super::ron_view::parsing::PlayerDataView;
use bevy::prelude::*;
use bevy_fact_rule_event::LayeredFactDatabase;

/// System that synchronizes InteractiveLayer state to ViewRoot local_facts.
///
/// This system runs every frame and updates the following local facts in ViewRoot:
/// - `selection`: Current selection index from the active layer
/// - `depth`: Could be derived from layer hierarchy (future enhancement)
/// - `active`: Whether any layer is active
///
/// 将 InteractiveLayer 状态同步到 ViewRoot local_facts 的系统。
///
/// 此系统每帧运行，并在 ViewRoot 中更新以下局部事实：
/// - `selection`：来自活跃层的当前选择索引
/// - `depth`：可以从层级层次结构派生（未来增强）
/// - `active`：是否有任何层处于活跃状态
pub(crate) fn sync_interactive_layer_to_view_root_system(
    layer_query: Query<&InteractiveLayer>,
    mut view_root_query: Query<&mut ViewRoot>,
) {
    // Find the active layer
    // 找到活跃的层
    let active_layer = layer_query.iter().find(|layer| layer.is_active);

    for mut view_root in view_root_query.iter_mut() {
        if let Some(layer) = active_layer {
            // Update selection from the active layer
            // 从活跃层更新选择
            let current_selection = view_root
                .local_facts
                .get_int("selection")
                .unwrap_or_default();
            if current_selection != layer.current_selection as i64 {
                view_root
                    .local_facts
                    .set("selection".to_string(), layer.current_selection as i64);
                debug!(
                    "[ViewRoot] Updated 'selection' fact: {} (from layer '{}')",
                    layer.current_selection, layer.layer_id
                );
            }

            // Update active state
            // 更新活跃状态
            let was_active = view_root.local_facts.get_bool("active").unwrap_or(false);
            if !was_active {
                view_root.local_facts.set("active".to_string(), true);
                debug!("[ViewRoot] Updated 'active' fact: true");
            }

            // Derive depth from layer_id naming convention
            // 从 layer_id 命名约定派生深度
            // E.g., "BackpackMenu" -> 0, "BackpackItem" -> 1, "BackpackItemOptions" -> 2
            let depth = derive_depth_from_layer_id(&layer.layer_id);
            let current_depth = view_root.local_facts.get_int("depth").unwrap_or_default();
            if current_depth != depth {
                view_root.local_facts.set("depth".to_string(), depth);
                debug!(
                    "[ViewRoot] Updated 'depth' fact: {} (derived from layer '{}')",
                    depth, layer.layer_id
                );
            }
        } else {
            // No active layer, could optionally set active to false
            // 没有活跃层，可选地将 active 设置为 false
            // Note: We might want to keep this behavior configurable
            // 注意：我们可能希望这个行为是可配置的
        }
    }
}

/// Derive depth value from layer ID based on naming conventions.
///
/// This uses a simple heuristic:
/// - Main menu layers (ending with "Menu") -> depth 0
/// - Item/content layers (ending with "Item" or similar) -> depth 1
/// - Options/submenu layers (ending with "Options") -> depth 2
/// - Status layers (ending with "Status") -> depth 3
///
/// For Battle:
/// - "BattleMainMenu" -> depth 0
///
/// 根据命名约定从层 ID 派生深度值。
fn derive_depth_from_layer_id(layer_id: &str) -> i64 {
    // Battle layers
    if layer_id == "BattleMainMenu" {
        return 0;
    }

    // Backpack layers - use explicit mapping
    match layer_id {
        "BackpackMenu" => 0,
        "BackpackItem" => 1,
        "BackpackItemOptions" => 2,
        "BackpackStatus" => 3,
        _ => {
            // Fallback: Try to derive from naming
            if layer_id.ends_with("Menu") {
                0
            } else if layer_id.ends_with("Options") {
                2
            } else if layer_id.ends_with("Status") {
                3
            } else if layer_id.ends_with("Item") {
                1
            } else {
                0
            }
        }
    }
}

/// System that evaluates `visible_when` expressions and updates visibility.
///
/// This system runs every frame for entities that have the `VisibleWhen` component.
/// It creates a `PlayerDataView` that can access View-local facts through `ViewRoot`.
///
/// 评估 `visible_when` 表达式并更新可见性的系统。
///
/// 对于具有 `VisibleWhen` 组件的实体，此系统每帧运行。
/// 它创建一个可以通过 `ViewRoot` 访问 View 局部事实的 `PlayerDataView`。
pub(crate) fn evaluate_visible_when_system(
    layered_db: Res<LayeredFactDatabase>,
    view_root_query: Query<&ViewRoot>,
    child_of_query: Query<&ChildOf>,
    mut query: Query<(Entity, &VisibleWhen, &mut Visibility)>,
) {
    for (entity, visible_when, mut visibility) in query.iter_mut() {
        // Try to find the ViewRoot ancestor by traversing up the hierarchy
        // 通过向上遍历层级尝试找到 ViewRoot 祖先
        let view_root = find_view_root_ancestor(entity, &view_root_query, &child_of_query);

        let is_visible = if let Some(view_root) = view_root {
            let player_data = PlayerDataView::with_local_facts(&layered_db, &view_root.local_facts);
            evaluate_visible_when_expr(&visible_when.expression, &player_data)
        } else {
            // No ViewRoot found, use base player data
            // 未找到 ViewRoot，使用基础 player data
            let player_data = PlayerDataView::new(&layered_db);
            evaluate_visible_when_expr(&visible_when.expression, &player_data)
        };

        // Update visibility
        let desired_visibility = if is_visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        if *visibility != desired_visibility {
            *visibility = desired_visibility;
            debug!(
                "Entity {:?} visibility changed to {:?} based on '{}'",
                entity, desired_visibility, visible_when.expression
            );
        }
    }
}

/// Find the ViewRoot ancestor by traversing up the entity hierarchy.
/// Returns None if no ViewRoot is found within the hierarchy.
///
/// 通过向上遍历实体层级查找 ViewRoot 祖先。
/// 如果在层级中未找到 ViewRoot 则返回 None。
fn find_view_root_ancestor<'a>(
    entity: Entity,
    view_root_query: &'a Query<&ViewRoot>,
    child_of_query: &Query<&ChildOf>,
) -> Option<&'a ViewRoot> {
    // First check if the entity itself has ViewRoot
    // 首先检查实体本身是否有 ViewRoot
    if let Ok(view_root) = view_root_query.get(entity) {
        return Some(view_root);
    }

    // Traverse up the hierarchy
    // 向上遍历层级
    let mut current = entity;
    const MAX_DEPTH: usize = 32; // Prevent infinite loops
    for _ in 0..MAX_DEPTH {
        if let Ok(child_of) = child_of_query.get(current) {
            let parent = child_of.parent();
            if let Ok(view_root) = view_root_query.get(parent) {
                return Some(view_root);
            }
            current = parent;
        } else {
            // No more parents
            break;
        }
    }

    None
}

/// Evaluate a visible_when expression string to a boolean.
///
/// 将 visible_when 表达式字符串评估为布尔值。
fn evaluate_visible_when_expr(expr: &str, player_data: &PlayerDataView) -> bool {
    use super::ron_view::parsing::evaluate_visible_when;
    evaluate_visible_when(expr, player_data)
}
