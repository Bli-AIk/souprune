//! Input transaction bridge for FRE events.
//!
//! FRE 事件的输入事务桥接层。

use bevy::prelude::*;
use bevy_fact_rule_event::FactEvent;

use crate::core::fre_facts;
use crate::core::input::{Direction, InputCommand, InputEnvelope, InputEnvelopeEvent, InputTarget};

/// Bridge that maps input envelopes into semantic FRE events.
///
/// 将输入事务映射为语义 FRE 事件的桥接器。
#[derive(Debug, Default, Clone, Copy)]
pub struct FreInputBridge;

impl FreInputBridge {
    /// Convert an input envelope into a FRE input event id.
    ///
    /// 将输入事务转换为 FRE 输入事件 ID。
    pub fn event_id(&self, envelope: &InputEnvelope) -> Option<&'static str> {
        if !matches!(envelope.target, InputTarget::FreScope) {
            return None;
        }

        Some(match envelope.command {
            InputCommand::Navigate(Direction::Up) => fre_facts::INPUT_NAVIGATE_UP,
            InputCommand::Navigate(Direction::Down) => fre_facts::INPUT_NAVIGATE_DOWN,
            InputCommand::Navigate(Direction::Left) => fre_facts::INPUT_NAVIGATE_LEFT,
            InputCommand::Navigate(Direction::Right) => fre_facts::INPUT_NAVIGATE_RIGHT,
            InputCommand::Confirm => fre_facts::INPUT_CONFIRM,
            InputCommand::Cancel => fre_facts::INPUT_CANCEL,
            InputCommand::Menu => fre_facts::INPUT_MENU,
        })
    }
}

/// Dispatch FRE-targeted input envelopes as semantic fact events.
///
/// 将面向 FRE 的输入事务分发为语义事实事件。
pub fn dispatch_fre_input_system(
    mut input_events: MessageReader<InputEnvelopeEvent>,
    mut fact_events: MessageWriter<FactEvent>,
) {
    let bridge = FreInputBridge;

    for input_event in input_events.read() {
        if let Some(event_id) = bridge.event_id(&input_event.envelope) {
            fact_events.write(FactEvent::new(event_id));
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::message::Messages;
    use bevy::prelude::*;
    use bevy_fact_rule_event::FactEvent;

    use crate::core::input::{
        Direction, InputCommand, InputContextId, InputEnvelope, InputEnvelopeEvent, InputTarget,
    };

    #[test]
    fn confirm_command_emits_only_semantic_confirm_event() {
        let mut app = App::new();
        app.add_message::<InputEnvelopeEvent>()
            .add_message::<FactEvent>()
            .add_systems(Update, super::dispatch_fre_input_system);

        app.world_mut()
            .write_message(InputEnvelopeEvent::new(InputEnvelope::new(
                InputContextId::Battle,
                InputTarget::FreScope,
                InputCommand::Confirm,
                "KeyboardX",
            )));

        app.update();

        let events = read_fact_event_ids(&app);
        assert_eq!(events, vec!["input:confirm"]);
    }

    #[test]
    fn navigation_command_does_not_leak_source_action_name() {
        let mut app = App::new();
        app.add_message::<InputEnvelopeEvent>()
            .add_message::<FactEvent>()
            .add_systems(Update, super::dispatch_fre_input_system);

        app.world_mut()
            .write_message(InputEnvelopeEvent::new(InputEnvelope::new(
                InputContextId::Overworld,
                InputTarget::FreScope,
                InputCommand::Navigate(Direction::Down),
                "KeyboardArrowDown",
            )));

        app.update();

        let events = read_fact_event_ids(&app);
        assert_eq!(events, vec!["input:navigate_down"]);
        assert!(!events.iter().any(|event| event.contains("keyboard")));
        assert!(!events.iter().any(|event| event.starts_with("action:")));
    }

    fn read_fact_event_ids(app: &App) -> Vec<String> {
        let fact_events = app.world().resource::<Messages<FactEvent>>();
        let mut cursor = fact_events.get_cursor();
        cursor
            .read(fact_events)
            .map(|event| event.id.0.clone())
            .collect()
    }
}
