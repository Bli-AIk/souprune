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
//! - **Data-Driven**: Dialogue state managed via FRE facts, not hardcoded resources
//!
//! - **解耦组件**：Typewriter 和 MortarController 相互独立
//! - **可配置输入**：所有输入通过 FRE 规则处理，无硬编码按键
//! - **多焦点支持**：多个对话实体可同时激活
//! - **数据驱动**：对话状态通过 FRE facts 管理，而非硬编码资源

mod components;
mod config;
mod systems;
mod typewriter_bridge;

pub use components::MortarController;
#[allow(deprecated)]
pub use config::DialogueBlockingConfig;
pub use config::DialogueInputConfig;

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

        #[allow(deprecated)]
        app.init_resource::<DialogueInputConfig>()
            .init_resource::<DialogueBlockingConfig>()
            .register_type::<MortarController>()
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
                    // Mortar event handling
                    typewriter_bridge::handle_typewriter_mortar_events,
                    // Pending dialogue startup (reads FRE facts, spawns view, starts Mortar)
                    systems::handle_pending_dialogue_start_system,
                )
                    .chain(),
            );
    }
}

/// Initialize dialogue-related facts in LayeredFactDatabase.
///
/// 在 LayeredFactDatabase 中初始化对话相关的 facts。
fn init_dialogue_facts(mut facts: ResMut<LayeredFactDatabase>) {
    // Initialize default values for dialogue-related FRE facts
    // 初始化对话相关的 FRE facts 默认值

    // Focus control - replaces DialogueFocus component
    // 焦点控制 - 替代 DialogueFocus 组件
    facts.set("dialogue:has_focus", FactValue::Bool(false));

    // Typewriter state facts
    // 打字机状态 facts
    facts.set("dialogue:typewriter_playing", FactValue::Bool(false));
    facts.set("dialogue:all_typewriters_finished", FactValue::Bool(true));
    facts.set("dialogue:any_typewriter_finished", FactValue::Bool(true));

    // Dialogue configuration facts
    // 对话配置 facts
    facts.set("dialogue:simple_text_active", FactValue::Bool(false));
    facts.set(
        "dialogue:simple_text",
        FactValue::String(String::new()),
    );
    facts.set("dialogue:has_typewriter", FactValue::Bool(true));

    // Pending dialogue configuration (set via FRE rules to trigger dialogue startup)
    // 待处理的对话配置（通过 FRE 规则设置以触发对话启动）
    facts.set(
        "dialogue:pending_mortar_path",
        FactValue::String(String::new()),
    );
    facts.set(
        "dialogue:pending_mortar_node",
        FactValue::String(String::new()),
    );
    // Pending view path for dialogue UI
    // 待处理的对话 UI 视图路径
    facts.set(
        "dialogue:pending_view",
        FactValue::String(String::new()),
    );

    // Focus mode: "all_finished" or "first_finished"
    // 焦点模式："all_finished" 或 "first_finished"
    facts.set(
        "dialogue:focus_mode",
        FactValue::String("all_finished".to_string()),
    );

    info!("DialoguePlugin: Initialized dialogue facts");
}
