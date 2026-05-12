//! Input bridge for View-owned interactions.
//!
//! View 自有交互的输入桥接层。

use bevy::prelude::*;

use super::{ActiveView, ViewRoot};
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
    mut active_view_query: Query<&mut ViewRoot, With<ActiveView>>,
) {
    let bridge = ViewInputBridge;

    for event in events.read() {
        if !matches!(event.envelope.target, InputTarget::ActiveView) {
            continue;
        }

        let Ok(mut view_root) = active_view_query.single_mut() else {
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
    use crate::core::view::{ActiveView, ViewRoot};

    #[test]
    fn navigate_down_changes_selection_through_view_control_method() {
        let mut view_root = ViewRoot::new("tests/menu.view.ron".to_string());
        view_root.local_facts.set("selection", FactValue::Int(0));

        view_root.apply_input_command(&InputCommand::Navigate(Direction::Down));

        assert_eq!(view_root.local_facts.get_int("selection"), Some(1));
        assert_eq!(
            view_root.local_facts.get_string("view:input:navigation"),
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
        view_root.local_facts.set("confirm_pressed", false);
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
                .local_facts
                .get_bool("view:input:confirm_requested"),
            Some(true)
        );
        assert_eq!(
            view_root.local_facts.get_bool("confirm_pressed"),
            Some(true)
        );

        let fre_events = app.world().resource::<Messages<FreCustomActionEvent>>();
        let mut cursor = fre_events.get_cursor();
        assert_eq!(cursor.read(fre_events).count(), 0);
    }
}
