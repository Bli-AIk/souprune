//! # sequencer/tween.rs
//!
//! TweenViewElement systems and utilities for the battle sequencer using bevy_tween.

mod interpolators;
mod systems;

pub use interpolators::SpriteAlphaInterpolator;
pub(crate) use interpolators::ViewBoxSizeInterpolator;
pub use systems::{process_tween_view_element_system, process_tween_wait_chapter_system};
