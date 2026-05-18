//! View lifecycle support.

mod focus;
mod rules;
mod transitions;

pub(crate) use focus::{
    cleanup_removed_view_focus_system, push_added_focus_scopes_system, sync_active_view_system,
};
pub(crate) use rules::{cleanup_view_rules_system, process_pending_view_rules_system};
pub(crate) use transitions::{
    StateTransitionTracker, StateViewTransitionSet, UIInteractiveStateTracker,
    backpack_state_transition_system, state_transition_sound_system,
};
