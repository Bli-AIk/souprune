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
pub use config::{ActiveDialogueState, DialogueBlockingConfig, DialogueInputConfig};

use bevy::prelude::*;
use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};

/// Plugin that integrates dialogue systems with Mortar, Typewriter, and FRE.
///
/// 整合 Mortar、Typewriter 和 FRE 的对话系统插件。
pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        // Add TypewriterPlugin as a dependency
        // 添加 TypewriterPlugin 作为依赖
        app.add_plugins(bevy_ecs_typewriter::TypewriterPlugin);

        app.init_resource::<DialogueInputConfig>()
            .init_resource::<DialogueBlockingConfig>()
            .init_resource::<ActiveDialogueState>()
            .register_type::<MortarController>()
            .register_type::<DialogueFocus>()
            .add_systems(Startup, init_dialogue_facts)
            .add_systems(
                Update,
                (
                    // Lifecycle systems
                    systems::spawn_dialogue_controller_system,
                    systems::despawn_dialogue_controller_system,
                    // Sync systems
                    systems::sync_mortar_text_to_typewriter_system,
                    systems::sync_typewriter_text_to_facts_system,
                    systems::sync_typewriter_state_to_facts_system,
                    // Input handling systems
                    systems::dialogue_advance_system,
                    systems::dialogue_skip_typewriter_system,
                    // Mortar event bridge
                    typewriter_bridge::handle_typewriter_mortar_events,
                )
                    .chain(),
            );
    }
}

/// Initialize dialogue-related facts in LayeredFactDatabase.
///
/// 在 LayeredFactDatabase 中初始化对话相关的 facts。
fn init_dialogue_facts(mut facts: ResMut<LayeredFactDatabase>) {
    // Set default value for typewriter state fact
    // This ensures the FRE condition can evaluate even before any typewriter exists
    // 设置打字机状态 fact 的默认值
    // 这确保即使没有打字机存在，FRE 条件也能正确评估
    facts.set("dialogue_typewriter_playing", FactValue::Bool(false));
    info!("DialoguePlugin: Initialized dialogue_typewriter_playing = false");
}
