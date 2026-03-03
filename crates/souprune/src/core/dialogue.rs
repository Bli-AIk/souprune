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
pub use components::TypewriterVoice;
#[allow(deprecated)]
pub use config::DialogueBlockingConfig;
pub use config::DialogueInputConfig;
pub use systems::DialogueControllerEntity;

use bevy::prelude::*;
use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};

use crate::core::fre_facts;

/// Plugin that integrates dialogue systems with Mortar, Typewriter, and FRE.
///
/// 整合 Mortar、Typewriter 和 FRE 的对话系统插件。
pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        // Add TypewriterPlugin as a dependency
        // 添加 TypewriterPlugin 作为依赖
        app.add_plugins(bevy_ecs_typewriter::TypewriterPlugin);

        #[allow(deprecated)]
        app.init_resource::<DialogueInputConfig>()
            .init_resource::<DialogueBlockingConfig>()
            .register_type::<MortarController>()
            .register_type::<TypewriterVoice>()
            .add_systems(Startup, init_dialogue_facts)
            .add_systems(
                schedule,
                (
                    // Must run first to set dialogue:active before spawn_dialogue_controller_system
                    // 必须先运行以在 spawn_dialogue_controller_system 之前设置 dialogue:active
                    systems::handle_pending_dialogue_start_system
                        .run_if(systems::has_pending_dialogue_start),
                    // Lifecycle systems
                    systems::spawn_dialogue_controller_system,
                    systems::despawn_dialogue_controller_system
                        .run_if(systems::should_check_dialogue_despawn),
                    // Sync systems
                    systems::sync_mortar_text_to_typewriter_system,
                    systems::sync_typewriter_text_to_facts_system,
                    systems::sync_typewriter_state_to_facts_system,
                    // Depth-based pause/resume - must run before voice to prevent sound on pause frame
                    // 基于 depth 的暂停/恢复 - 必须在 voice 之前运行以避免暂停帧播放音效
                    systems::replay_typewriter_on_depth_resume_system,
                    // Handle dialogue:stop event - stops typewriter on FRE event
                    // 处理 dialogue:stop 事件 - 响应 FRE 事件停止打字机
                    systems::handle_dialogue_stop_event_system,
                    // Voice system - plays sound on char advance
                    systems::typewriter_voice_system,
                    // Input handling systems
                    systems::dialogue_advance_system.run_if(systems::has_fact_events),
                    systems::emit_pending_dialogue_ended_system
                        .run_if(systems::has_pending_dialogue_ended),
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
    facts.set_global(fre_facts::DIALOGUE_HAS_FOCUS, FactValue::Bool(false));

    // Typewriter state facts
    // 打字机状态 facts
    facts.set_global(
        fre_facts::DIALOGUE_TYPEWRITER_PLAYING,
        FactValue::Bool(false),
    );
    facts.set_global(
        fre_facts::DIALOGUE_ALL_TYPEWRITERS_FINISHED,
        FactValue::Bool(true),
    );
    facts.set_global(
        fre_facts::DIALOGUE_ANY_TYPEWRITER_FINISHED,
        FactValue::Bool(true),
    );

    // Dialogue configuration facts
    // 对话配置 facts
    facts.set_global(
        fre_facts::DIALOGUE_SIMPLE_TEXT_ACTIVE,
        FactValue::Bool(false),
    );
    facts.set_global(
        fre_facts::DIALOGUE_SIMPLE_TEXT,
        FactValue::String(String::new()),
    );
    facts.set_global(fre_facts::DIALOGUE_HAS_TYPEWRITER, FactValue::Bool(true));

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
    facts.set_global(fre_facts::DIALOGUE_PENDING_START, FactValue::Bool(false));
    facts.set_global(
        fre_facts::DIALOGUE_PENDING_MORTAR_PATH,
        FactValue::String(String::new()),
    );
    facts.set_global(
        fre_facts::DIALOGUE_PENDING_MORTAR_NODE,
        FactValue::String(String::new()),
    );
    facts.set_global(
        fre_facts::DIALOGUE_PENDING_VIEW,
        FactValue::String(String::new()),
    );

    // Dialogue active state (set by handle_pending_dialogue_start_system)
    // 对话活跃状态（由 handle_pending_dialogue_start_system 设置）
    facts.set_global(fre_facts::DIALOGUE_ACTIVE, FactValue::Bool(false));
    facts.set_global(fre_facts::DIALOGUE_HAS_MORTAR, FactValue::Bool(false));

    // Focus mode: "all_finished" or "first_finished"
    // 焦点模式："all_finished" 或 "first_finished"
    facts.set_global(
        fre_facts::DIALOGUE_FOCUS_MODE,
        FactValue::String("all_finished".to_string()),
    );

    info!("DialoguePlugin: Initialized dialogue facts (global layer)");
}
