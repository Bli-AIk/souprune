//! FRE action definitions for SoupRune.
//!
//! SoupRune 的 FRE 动作定义。
//!
//! 只包含引擎核心动作。游戏特定动作（如 UseItem、CheckItem、DropItem）
//! 通过 `Custom` 变体 + [`ViewActionExtensions`] 分发机制处理，
//! 不再作为枚举变体出现在此处。
//!
//! [`ViewActionExtensions`]: crate::core::fre_bridge::extensions::ViewActionExtensions

use bevy_fact_rule_event::{ActionDef, LocalFactValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core FRE action definitions.
///
/// Game-specific actions use the `Custom` variant with a registered
/// handler in `ViewActionExtensions`.
#[derive(Debug, Clone, Serialize, Deserialize, bevy::reflect::TypePath)]
pub enum GameActionDef {
    Log {
        message: String,
    },
    SetLocalFact(String, LocalFactValue),
    EmitEvent(String),
    Custom {
        action_type: String,
        #[serde(default)]
        params: HashMap<String, String>,
    },
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
        }
    }
}

/// Type aliases for FRE types parameterized with GameActionDef.
pub type GameFreAsset = bevy_fact_rule_event::FreAsset<GameActionDef>;
pub type GameRuleDef = bevy_fact_rule_event::RuleDef<GameActionDef>;
pub type GameRule = bevy_fact_rule_event::Rule<GameActionDef>;
pub type GameRuleRegistry = bevy_fact_rule_event::LayeredRuleRegistry<GameActionDef>;
pub type GameActionHandlerRegistry = bevy_fact_rule_event::ActionHandlerRegistry<GameActionDef>;
