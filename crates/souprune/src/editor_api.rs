//! Stable editor-facing API surface.
//!
//! The editor should depend on this module or on shared schema crates,
//! not on deep internal framework paths.

pub mod app {
    pub use crate::app_state::app_setup::ResolutionScale;
    pub use crate::app_state::{AppState, SequenceMode, SequenceSubState};
}

pub mod camera {
    pub use crate::core::camera::MainGameCamera;
}

pub mod debug {
    pub use crate::extra::debug::{ColliderGizmos, RuleTriggerHistory, setup_collider_debug};
}

pub mod fre_bridge {
    pub use crate::core::fre_bridge::process_view_actions_system;
}

pub mod game_action {
    pub use crate::core::game_action::{
        GameActionDef, GameFreAsset, GameRule, GameRuleDef, GameRuleRegistry,
    };
}

pub mod input {
    pub use crate::core::input::InputConfig;
    pub use crate::core::input::touch::TouchOverlayEnabled;
}

pub mod mortar {
    pub use crate::extra::mortar::MortarStringTable;
}

pub mod multi_source {
    pub use crate::extra::multi_source::MultiSourceAssetReader;
}

pub mod sequencer {
    pub use crate::core::sequencer::{
        CurrentSequenceFlow, SequenceAsset, SequenceContext, SequenceExecutionState,
        SequenceRulesHandle, runtime_asset_from_schema,
    };
}

pub mod state_config {
    pub use crate::core::state_config::LoadedStateConfig;
}

pub mod values {
    pub use crate::core::sequencer::chapter_schema::Value as RuntimeExprValue;
}

pub mod view {
    pub use crate::core::view::components::{ActiveView, ViewRoot};
    pub use crate::core::view::layout::serde_types::{
        SerializableColor, SerializableTransform, SerializableVec2, SerializableVec3, ViewFontDef,
    };
    pub use crate::core::view::layout::{
        DataRequirement, InitialFactValue, RepeatDef, SpriteDef, StateSpriteConfig, TextDef,
        ViewBoxLogicDef, ViewLayoutAsset, ViewNodeDef, runtime_sdf_structure_from_schema,
        runtime_view_layout_from_schema,
    };
    pub use crate::core::view::reconcile::{
        SpawnContext, ViewElementSpec, build_text_config, spawn_sprite_entity, spawn_text_entity,
        spawn_viewbox_entity,
    };
    pub use crate::core::view::ron_view::parsing::{PlayerDataView, RepeatContext};
    pub use crate::core::view::ron_view::spawn_helpers::load_fre_into_view_root;
    pub use crate::core::view::ron_view::update_fact_dependent_ui_elements;
    pub use crate::core::view::sdf_view_shape::update_sdf_view_shape_system;
    pub use crate::core::view::text::show_text_when_ready_system;
    pub use crate::core::view::visible_when::evaluate_visible_when_system;
    pub use crate::core::visual::Visual;
}
