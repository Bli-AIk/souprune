//! View focus lifecycle and derived active-view state.
//!
//! View 焦点生命周期和派生的活跃 View 状态。

use bevy::prelude::*;

use super::super::components::{ActiveView, ViewFocusScope, ViewFocusStack, ViewRoot};

/// Push newly focusable View roots onto the focus stack.
///
/// 将新增的可聚焦 View 根节点推入焦点栈。
pub(crate) fn push_added_focus_scopes_system(
    mut focus_stack: ResMut<ViewFocusStack>,
    added_scopes: Query<Entity, Added<ViewFocusScope>>,
) {
    for entity in added_scopes.iter() {
        focus_stack.push(entity);
    }
}

/// Remove despawned View roots from the focus stack.
///
/// 从焦点栈移除已销毁的 View 根节点。
pub(crate) fn cleanup_removed_view_focus_system(
    mut removed_roots: RemovedComponents<ViewRoot>,
    mut focus_stack: ResMut<ViewFocusStack>,
) {
    for entity in removed_roots.read() {
        focus_stack.remove(entity);
    }
}

/// Derive the unique `ActiveView` marker from the focus stack top.
///
/// 从焦点栈栈顶派生唯一的 `ActiveView` 标记。
pub(crate) fn sync_active_view_system(
    mut commands: Commands,
    mut focus_stack: ResMut<ViewFocusStack>,
    focus_roots: Query<Entity, (With<ViewRoot>, With<ViewFocusScope>)>,
    active_views: Query<Entity, With<ActiveView>>,
) {
    focus_stack.retain(|entity| focus_roots.contains(entity));
    let top = focus_stack.top();

    for entity in active_views.iter() {
        if Some(entity) != top {
            commands.entity(entity).remove::<ActiveView>();
        }
    }

    let Some(entity) = top else {
        return;
    };

    if !active_views.contains(entity)
        && let Ok(mut entity_commands) = commands.get_entity(entity)
    {
        entity_commands.insert(ActiveView);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::core::view::components::{ActiveView, ViewFocusScope, ViewFocusStack, ViewRoot};

    #[test]
    fn stack_push_deduplicates_and_remove_restores_previous_top() {
        let first = Entity::from_raw_u32(1).unwrap();
        let second = Entity::from_raw_u32(2).unwrap();
        let mut stack = ViewFocusStack::default();

        stack.push(first);
        stack.push(second);
        stack.push(first);

        assert_eq!(stack.top(), Some(first));
        assert_eq!(stack.remove(first), Some(first));
        assert_eq!(stack.top(), Some(second));
        assert_eq!(stack.remove(first), None);

        stack.clear();
        assert_eq!(stack.top(), None);
    }

    #[test]
    fn two_focus_scopes_mark_only_top_active_view() {
        let mut app = App::new();
        app.init_resource::<ViewFocusStack>().add_systems(
            Update,
            (
                super::push_added_focus_scopes_system,
                super::sync_active_view_system,
            ),
        );

        let bottom = app
            .world_mut()
            .spawn((
                ViewRoot::new("tests/bottom.view.ron".to_string()),
                ViewFocusScope,
            ))
            .id();
        let top = app
            .world_mut()
            .spawn((
                ViewRoot::new("tests/top.view.ron".to_string()),
                ViewFocusScope,
            ))
            .id();

        app.update();
        app.update();

        assert!(!app.world().entity(bottom).contains::<ActiveView>());
        assert!(app.world().entity(top).contains::<ActiveView>());
    }

    #[test]
    fn removing_top_view_restores_previous_active_view() {
        let mut app = App::new();
        app.init_resource::<ViewFocusStack>().add_systems(
            Update,
            (
                super::push_added_focus_scopes_system,
                super::cleanup_removed_view_focus_system,
                super::sync_active_view_system,
            ),
        );

        let bottom = app
            .world_mut()
            .spawn((
                ViewRoot::new("tests/bottom.view.ron".to_string()),
                ViewFocusScope,
            ))
            .id();
        let top = app
            .world_mut()
            .spawn((
                ViewRoot::new("tests/top.view.ron".to_string()),
                ViewFocusScope,
            ))
            .id();

        app.update();
        app.update();
        app.world_mut().entity_mut(top).despawn();
        app.update();
        app.update();

        assert!(app.world().entity(bottom).contains::<ActiveView>());
        assert!(app.world().get_entity(top).is_err());
        assert_eq!(app.world().resource::<ViewFocusStack>().top(), Some(bottom));
    }
}
