//! Runtime visibility evaluation system using `visible_when` expressions.
//!
//! 使用 `visible_when` 表达式的运行时可见性评估系统。
//!
//! This module provides the system that evaluates `visible_when` expressions
//! every frame and updates entity visibility accordingly.
//!
//! 本模块提供每帧评估 `visible_when` 表达式并相应更新实体可见性的系统。

use super::components::{ViewRoot, VisibleWhen};
use super::ron_view::parsing::PlayerDataView;
use bevy::prelude::*;
use bevy_fact_rule_event::LayeredFactDatabase;

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
