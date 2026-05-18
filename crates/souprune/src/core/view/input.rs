//! Input bridge for View-owned interactions.
//!
//! View 自有交互的输入桥接层。

use bevy::prelude::*;

use super::components::{ActiveView, ViewFocusStack, ViewRoot};
use crate::core::input::{InputEnvelope, InputEnvelopeEvent, InputTarget};

/// Bridge that applies unified input envelopes to active Views.
///
/// 把统一输入事务应用到活跃 View 的桥接器。
#[derive(Debug, Default, Clone, Copy)]
pub struct ViewInputBridge;

impl ViewInputBridge {
    /// Handle one input envelope for a View root.
    ///
    /// 为一个 View 根节点处理一笔输入事务。
    pub fn handle(&self, envelope: &InputEnvelope, view_root: &mut ViewRoot) {
        if !matches!(envelope.target, InputTarget::ActiveView) {
            return;
        }

        view_root.apply_input_command(&envelope.command);
    }
}

/// Dispatch View-targeted input envelopes to the active View root.
///
/// 将面向 View 的输入事务分发给当前活跃 View 根节点。
pub fn dispatch_view_input_system(
    mut events: MessageReader<InputEnvelopeEvent>,
    focus_stack: Option<Res<ViewFocusStack>>,
    active_view_query: Query<Entity, With<ActiveView>>,
    mut view_root_query: Query<&mut ViewRoot>,
) {
    let bridge = ViewInputBridge;

    for event in events.read() {
        if !matches!(event.envelope.target, InputTarget::ActiveView) {
            continue;
        }

        let target = match focus_stack.as_ref() {
            Some(stack) => stack
                .top()
                .filter(|entity| view_root_query.contains(*entity)),
            None => active_view_query
                .single()
                .ok()
                .filter(|entity| view_root_query.contains(*entity)),
        };

        let Some(entity) = target else {
            continue;
        };

        let Ok(mut view_root) = view_root_query.get_mut(entity) else {
            continue;
        };

        bridge.handle(&event.envelope, &mut view_root);
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::message::Messages;
    use bevy::prelude::*;
    use bevy_fact_rule_event::FactValue;

    use crate::core::fre_bridge::FreCustomActionEvent;
    use crate::core::input::{
        Direction, InputCommand, InputContextId, InputEnvelope, InputEnvelopeEvent, InputTarget,
    };
    use crate::core::view::components::{ActiveView, ViewFocusScope, ViewFocusStack, ViewRoot};

    #[test]
    fn navigate_down_changes_selection_through_view_control_method() {
        let mut view_root = ViewRoot::new("tests/menu.view.ron".to_string());
        view_root.set_local_value("selection", FactValue::Int(0));

        view_root.apply_input_command(&InputCommand::Navigate(Direction::Down));

        assert_eq!(view_root.local_state().get_int("selection"), Some(1));
        assert_eq!(
            view_root.local_state().get_string("view:input:navigation"),
            Some("down")
        );
    }

    #[test]
    fn bridge_handles_confirm_as_view_state_without_fre_custom_action() {
        let mut app = App::new();
        app.add_message::<InputEnvelopeEvent>()
            .add_message::<FreCustomActionEvent>()
            .add_systems(Update, super::dispatch_view_input_system);

        let mut view_root = ViewRoot::new("tests/menu.view.ron".to_string());
        view_root.set_local_value("confirm_pressed", false);
        app.world_mut().spawn((view_root, ActiveView));
        app.world_mut()
            .write_message(InputEnvelopeEvent::new(InputEnvelope::new(
                InputContextId::View,
                InputTarget::ActiveView,
                InputCommand::Confirm,
                "Confirm",
            )));

        app.update();

        let mut query = app.world_mut().query::<&ViewRoot>();
        let view_root = query.single(app.world()).unwrap();
        assert_eq!(
            view_root
                .local_state()
                .get_bool("view:input:confirm_requested"),
            Some(true)
        );
        assert_eq!(
            view_root.local_state().get_bool("confirm_pressed"),
            Some(true)
        );

        let fre_events = app.world().resource::<Messages<FreCustomActionEvent>>();
        let mut cursor = fre_events.get_cursor();
        assert_eq!(cursor.read(fre_events).count(), 0);
    }

    #[test]
    fn dispatch_uses_focus_stack_top_when_multiple_view_roots_exist() {
        let mut app = App::new();
        app.add_message::<InputEnvelopeEvent>()
            .init_resource::<ViewFocusStack>()
            .add_systems(Update, super::dispatch_view_input_system);

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
        app.world_mut()
            .resource_mut::<ViewFocusStack>()
            .push(bottom);
        app.world_mut().resource_mut::<ViewFocusStack>().push(top);

        app.world_mut()
            .write_message(InputEnvelopeEvent::new(InputEnvelope::new(
                InputContextId::View,
                InputTarget::ActiveView,
                InputCommand::Confirm,
                "Confirm",
            )));

        app.update();

        let bottom_root = app.world().entity(bottom).get::<ViewRoot>().unwrap();
        assert_eq!(
            bottom_root
                .local_state()
                .get_bool(ViewRoot::INPUT_CONFIRM_REQUESTED),
            None
        );

        let top_root = app.world().entity(top).get::<ViewRoot>().unwrap();
        assert_eq!(
            top_root
                .local_state()
                .get_bool(ViewRoot::INPUT_CONFIRM_REQUESTED),
            Some(true)
        );
    }
}
