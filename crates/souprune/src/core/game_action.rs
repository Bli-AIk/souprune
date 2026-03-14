//! Game-specific FRE action definitions for SoupRune.
//!
//! SoupRune 特定的 FRE 动作定义。

use bevy_fact_rule_event::{ActionDef, LocalFactValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Game-specific action definitions for SoupRune.
/// This enum defines all actions that can appear in `.fre.ron` rule files.
///
/// SoupRune 的游戏特定动作定义。
/// 此枚举定义了所有可以出现在 `.fre.ron` 规则文件中的动作。
#[derive(Debug, Clone, Serialize, Deserialize, bevy::reflect::TypePath)]
pub enum GameActionDef {
    // -- Core actions --
    Log {
        message: String,
    },
    SetLocalFact(String, LocalFactValue),
    EmitEvent(String),
    Custom {
        action_type: String,
        params: HashMap<String, String>,
    },
    // -- Game-specific actions --
    PlaySound(String),
    PlaySoundFullPath(String),
    CloseView,
    SwitchState(String),
    StartDialogue {
        mortar: String,
        node: String,
        #[serde(default)]
        view: Option<String>,
        #[serde(default = "default_true")]
        typewriter: bool,
        #[serde(default = "default_true")]
        focus: bool,
        #[serde(default)]
        voice: Option<String>,
    },
    UseItem {
        index_expr: String,
    },
    CheckItem {
        index_expr: String,
    },
    DropItem {
        index_expr: String,
    },
}

fn default_true() -> bool {
    true
}

impl ActionDef for GameActionDef {
    fn action_type(&self) -> &str {
        match self {
            Self::Log { .. } => "Log",
            Self::SetLocalFact(_, _) => "SetLocalFact",
            Self::EmitEvent(_) => "EmitEvent",
            Self::Custom { action_type, .. } => action_type.as_str(),
            Self::PlaySound(_) => "PlaySound",
            Self::PlaySoundFullPath(_) => "PlaySoundFullPath",
            Self::CloseView => "CloseView",
            Self::SwitchState(_) => "SwitchState",
            Self::StartDialogue { .. } => "StartDialogue",
            Self::UseItem { .. } => "UseItem",
            Self::CheckItem { .. } => "CheckItem",
            Self::DropItem { .. } => "DropItem",
        }
    }
}

/// Type aliases for FRE types parameterized with GameActionDef.
pub type GameFreAsset = bevy_fact_rule_event::FreAsset<GameActionDef>;
pub type GameRuleDef = bevy_fact_rule_event::RuleDef<GameActionDef>;
pub type GameRule = bevy_fact_rule_event::Rule<GameActionDef>;
pub type GameRuleRegistry = bevy_fact_rule_event::LayeredRuleRegistry<GameActionDef>;
pub type GameActionHandlerRegistry = bevy_fact_rule_event::ActionHandlerRegistry<GameActionDef>;
