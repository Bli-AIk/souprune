//! View lifecycle support.

mod rules;
mod transitions;

pub(crate) use rules::{cleanup_view_rules_system, process_pending_view_rules_system};
pub(crate) use transitions::{
    StateTransitionTracker, UIInteractiveStateTracker, backpack_state_transition_system,
    state_transition_sound_system,
};
