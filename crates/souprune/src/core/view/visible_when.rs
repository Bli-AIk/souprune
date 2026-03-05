//! Runtime visibility evaluation system using `visible_when` expressions.
//!
//! 使用 `visible_when` 表达式的运行时可见性评估系统。
//!
//! This module provides the system that evaluates `visible_when` expressions
//! and updates entity visibility accordingly.
//!
//! ## Performance Optimization
//!
//! This system only runs when relevant data changes:
//! - Global LayeredFactDatabase changes
//! - Any ViewRoot's local_facts changes
//!
//! ## 性能优化
//!
//! 此系统仅在相关数据变更时运行：
//! - 全局 LayeredFactDatabase 变更
//! - 任何 ViewRoot 的 local_facts 变更
//!
//! 本模块提供评估 `visible_when` 表达式并相应更新实体可见性的系统。

use super::components::{ViewRoot, VisibleWhen};
use super::ron_view::parsing::PlayerDataView;
use bevy::prelude::*;
use bevy_fact_rule_event::LayeredFactDatabase;

/// System that evaluates `visible_when` expressions and updates visibility.
///
/// This system only runs when the fact database or any ViewRoot's local_facts changes,
/// avoiding unnecessary per-frame computation.
///
/// 评估 `visible_when` 表达式并更新可见性的系统。
///
/// 此系统仅在 fact 数据库或任何 ViewRoot 的 local_facts 变更时运行，
/// 避免不必要的每帧计算。
pub fn evaluate_visible_when_system(
    layered_db: Res<LayeredFactDatabase>,
    view_root_query: Query<&ViewRoot>,
    changed_view_roots: Query<Entity, Changed<ViewRoot>>,
    child_of_query: Query<&ChildOf>,
    mut query: Query<(Entity, &VisibleWhen, &mut Visibility)>,
) {
    // Performance optimization: Only evaluate when data actually changes
    // 性能优化：仅在数据实际变更时评估
    let global_changed = layered_db.is_changed();
    let any_view_root_changed = !changed_view_roots.is_empty();

    if !global_changed && !any_view_root_changed {
        return;
    }

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
            trace!(
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
