//! # Dialogue System
//!
//! # 对话系统
//!
//! This module provides dialogue system integration with Mortar, Typewriter, and FRE.
//!
//! 本模块提供 Mortar、Typewriter 和 FRE 的对话系统整合。
//!
//! ## Design Principles
//!
//! ## 设计原则
//!
//! - **Decoupled Components**: Typewriter and MortarController are independent
//! - **Configurable Input**: All input handling through FRE rules, no hardcoded keys
//! - **Multi-focus Support**: Multiple dialogue entities can be active simultaneously
//!
//! - **解耦组件**：Typewriter 和 MortarController 相互独立
//! - **可配置输入**：所有输入通过 FRE 规则处理，无硬编码按键
//! - **多焦点支持**：多个对话实体可同时激活

mod components;
mod config;
mod systems;
mod typewriter_bridge;

pub use components::{DialogueFocus, MortarController};
pub use config::{DialogueBlockingConfig, DialogueInputConfig};

use bevy::prelude::*;

/// Plugin that integrates dialogue systems with Mortar, Typewriter, and FRE.
///
/// 整合 Mortar、Typewriter 和 FRE 的对话系统插件。
pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DialogueInputConfig>()
            .init_resource::<DialogueBlockingConfig>()
            .register_type::<MortarController>()
            .register_type::<DialogueFocus>()
            .add_systems(
                Update,
                (
                    systems::sync_typewriter_state_to_facts_system,
                    systems::sync_mortar_text_to_facts_system,
                    systems::dialogue_advance_system,
                    systems::dialogue_skip_typewriter_system,
                    typewriter_bridge::handle_typewriter_mortar_events,
                )
                    .chain(),
            );
    }
}
