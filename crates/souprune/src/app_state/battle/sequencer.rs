//! # sequencer.rs
//!
//! # sequencer.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Sequencer is the linear sequence manager for the battle system.
//! It is responsible for managing and executing Chapters in the battle,
//! ensuring they proceed in order.
//!
//! Sequencer 是战斗系统的线性序列管理器。
//! 它负责管理和执行战斗中的章节（Chapter），确保它们按顺序进行。

mod camera;
mod context;
mod flow;
mod interaction;
mod performance;
mod player;
mod tween;
mod ui;
mod view_element;

// Re-export public types
pub use context::{BattleContext, BattleExecutionState};

use crate::app_state::AppState;
use crate::app_state::battle::BattleUpdate;
use bevy::prelude::*;
use bevy_tween::BevyTweenRegisterSystems;
use bevy_tween::tween::component_tween_system;

/// Module for the battle sequencer.
///
/// 战斗系统的线性序列管理器。
pub(crate) struct SequencerPlugin;

impl Plugin for SequencerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<context::BattleContext>()
            // Register custom interpolator systems for bevy_tween
            .add_tween_systems(component_tween_system::<tween::UIBoxSizeInterpolator>())
            .add_tween_systems(component_tween_system::<tween::SpriteAlphaInterpolator>())
            .add_systems(OnEnter(AppState::Battle), flow::load_default_chapter_system)
            .add_systems(
                Update,
                (
                    flow::advance_battle_flow_system,
                    player::process_player_action_system,
                    camera::process_camera_action_system,
                    ui::process_ui_action_system,
                    interaction::process_await_selection_system,
                    view_element::process_modify_view_element_system,
                    tween::process_tween_view_element_system,
                    performance::process_danmaku_performance_system,
                    performance::process_am_performance_system,
                    player::process_player_spawn_requests,
                    flow::process_wait_chapter_system,
                    tween::process_tween_wait_chapter_system,
                    performance::process_am_wait_chapter_system,
                    flow::process_parallel_chapter_system,
                    interaction::check_await_selection_completion_system,
                    interaction::update_interactive_layer_sprites_system,
                    flow::cleanup_finished_chapters_system,
                    flow::sync_battle_flow_system,
                )
                    .chain()
                    .in_set(BattleUpdate),
            );
    }
}
