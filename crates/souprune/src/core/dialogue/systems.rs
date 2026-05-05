//! Dialogue system core systems.
//!
//! 对话系统核心系统。

pub mod ghost_text;
mod input;
mod lifecycle;
mod mortar_sync;
mod state_sync;
pub mod text_animation;
mod voice;

pub use input::{
    dialogue_advance_system, dialogue_skip_typewriter_system, handle_dialogue_stop_event_system,
    has_fact_events,
};
pub use lifecycle::{
    DialogueControllerEntity, DialogueStartRequest, despawn_dialogue_controller_system,
    emit_pending_dialogue_ended_system, handle_mortar_dialogue_finished_system,
    handle_pending_dialogue_start_system, has_pending_dialogue_ended, has_pending_dialogue_start,
    should_check_dialogue_despawn, spawn_dialogue_controller_system,
};
pub use mortar_sync::{
    MortarFactBindings, prepare_item_dialogue_mortar_system, sync_mortar_text_to_typewriter_system,
};
pub use state_sync::{
    replay_typewriter_on_depth_resume_system, sync_typewriter_state_to_facts_system,
    sync_typewriter_text_to_facts_system,
};
pub use voice::typewriter_voice_system;
