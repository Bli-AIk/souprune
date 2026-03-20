//! # sequencer/fact_chapter.rs
//!
//! Processing systems for FRE-based conditional chapters.

mod branching;
mod fact_ops;
mod load_enemies;
mod load_fre;

pub use branching::{process_conditional_chapter_system, process_fact_switch_chapter_system};
pub use fact_ops::{process_emit_fact_event_chapter_system, process_modify_fact_chapter_system};
pub use load_enemies::{complete_load_enemies_chapter_system, process_load_enemies_chapter_system};
pub use load_fre::{complete_load_fre_chapter_system, process_load_fre_chapter_system};
