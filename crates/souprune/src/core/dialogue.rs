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
                    // Pending dialogue startup (reads FRE facts, spawns view, starts Mortar)
                    // Must run first to set dialogue:active before spawn_dialogue_controller_system
                    // 必须先运行以在 spawn_dialogue_controller_system 之前设置 dialogue:active
                    systems::handle_pending_dialogue_start_system,
                    // Lifecycle systems
                    systems::spawn_dialogue_controller_system,
                    systems::despawn_dialogue_controller_system,
                    // Sync systems
                    systems::sync_mortar_text_to_typewriter_system,
                    systems::sync_typewriter_text_to_facts_system,
                    systems::sync_typewriter_state_to_facts_system,
                    // Input handling systems
                    systems::dialogue_advance_system,
                    systems::emit_pending_dialogue_ended_system,
                    systems::handle_mortar_dialogue_finished_system,
                    systems::dialogue_skip_typewriter_system,
                    // Mortar event handling
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
    // Initialize default values for dialogue-related FRE facts
    // 初始化对话相关的 FRE facts 默认值
    //
    // NOTE: These are written to GLOBAL layer because they are core dialogue system
    // state that must persist across scene transitions (e.g., Overworld -> Battle).
    // Scene transitions call clear_local() which would wipe local-layer facts.
    //
    // 注意：这些写入 GLOBAL 层，因为它们是核心对话系统状态，
    // 必须跨场景转换持久化（如 Overworld -> Battle）。
    // 场景转换会调用 clear_local() 清除局部层 facts。

    // Focus control - replaces DialogueFocus component
    // 焦点控制 - 替代 DialogueFocus 组件
    facts.set_global("dialogue:has_focus", FactValue::Bool(false));

    // Typewriter state facts
    // 打字机状态 facts
    facts.set_global("dialogue:typewriter_playing", FactValue::Bool(false));
    facts.set_global("dialogue:all_typewriters_finished", FactValue::Bool(true));
    facts.set_global("dialogue:any_typewriter_finished", FactValue::Bool(true));

    // Dialogue configuration facts
    // 对话配置 facts
    facts.set_global("dialogue:simple_text_active", FactValue::Bool(false));
    facts.set_global("dialogue:simple_text", FactValue::String(String::new()));
    facts.set_global("dialogue:has_typewriter", FactValue::Bool(true));

    // NOTE: dialogue_text is now managed by View's local_facts, not LayeredFactDatabase.
    // Views that use {{dialogue_text}} should define it in their `facts:` section.
    // 注意：dialogue_text 现在由 View 的 local_facts 管理，而非 LayeredFactDatabase。
    // 使用 {{dialogue_text}} 的 View 应在其 `facts:` 部分中定义它。

    // Pending dialogue configuration (set via FRE rules to trigger dialogue startup)
    // 待处理的对话配置（通过 FRE 规则设置以触发对话启动）
    //
    // To start a dialogue via FRE rules, set these facts in modifications:
    // 要通过 FRE 规则启动对话，在 modifications 中设置这些 facts：
    //   - dialogue:pending_start = true (trigger)
    //   - dialogue:pending_mortar_path = "path/to/file.mortar" (without locale prefix)
    //   - dialogue:pending_mortar_node = "node_name"
    //   - dialogue:pending_view = "path/to/view.ron" (optional, for spawning new view)
    facts.set_global("dialogue:pending_start", FactValue::Bool(false));
    facts.set_global(
        "dialogue:pending_mortar_path",
        FactValue::String(String::new()),
    );
    facts.set_global(
        "dialogue:pending_mortar_node",
        FactValue::String(String::new()),
    );
    facts.set_global("dialogue:pending_view", FactValue::String(String::new()));

    // Dialogue active state (set by handle_pending_dialogue_start_system)
    // 对话活跃状态（由 handle_pending_dialogue_start_system 设置）
    facts.set_global("dialogue:active", FactValue::Bool(false));
    facts.set_global("dialogue:has_mortar", FactValue::Bool(false));

    // Focus mode: "all_finished" or "first_finished"
    // 焦点模式："all_finished" 或 "first_finished"
    facts.set_global(
        "dialogue:focus_mode",
        FactValue::String("all_finished".to_string()),
    );

    info!("DialoguePlugin: Initialized dialogue facts (global layer)");
}
