//! # fre.rs
//!
//! FreAsset schema types for `.fre.ron` files.
//! Mirrors `bevy_fact_rule_event::asset` without Bevy dependency.
//!
//! `.fre.ron` 文件的 FRE 资源 Schema 类型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Fact Value Types
// ============================================================================

/// Serializable fact value for RON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactValueDef {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    StringList(Vec<String>),
    IntList(Vec<i64>),
    Enum(String),
}

/// Serializable modification definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FreFactModificationDef {
    Set { key: String, value: FactValueDef },
    Increment { key: String, amount: i64 },
    Add { key: String, value: f64 },
    Sub { key: String, value: f64 },
    Mul { key: String, value: f64 },
    Div { key: String, value: f64 },
    Mod { key: String, value: i64 },
    Clamp { key: String, min: f64, max: f64 },
    Wrap { key: String, min: i64, max: i64 },
    Eval { key: String, expr: String },
    Remove(String),
    Toggle(String),
}

// ============================================================================
// Action / Event Types
// ============================================================================

/// Kind of action event (press state).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionEventKind {
    JustPressed,
    Pressed,
    JustReleased,
}

/// Serializable event definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleEventDef {
    Event(String),
    ActionEvent {
        action: String,
        kind: ActionEventKind,
    },
}

impl Default for RuleEventDef {
    fn default() -> Self {
        RuleEventDef::Event(String::new())
    }
}

/// Serializable action definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleActionDef {
    Log {
        message: String,
    },
    PlaySound(String),
    PlaySoundFullPath(String),
    SetLocalFact(String, LocalFactValue),
    CloseView,
    SwitchState(String),
    EmitEvent(String),
    Custom {
        action_type: String,
        #[serde(default, serialize_with = "crate::ordered_map::serialize_ordered_map")]
        params: HashMap<String, String>,
    },
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

/// Value for SetLocalFact action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocalFactValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Expr(String),
    Enum(String),
}

// ============================================================================
// Rule Definition
// ============================================================================

/// A single rule definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDef {
    #[serde(default)]
    pub id: String,
    pub event: RuleEventDef,
    #[serde(default)]
    pub conditions: Vec<String>,
    #[serde(default)]
    pub actions: Vec<RuleActionDef>,
    #[serde(default)]
    pub modifications: Vec<FreFactModificationDef>,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub priority: i32,
    #[serde(default = "default_consume_event")]
    pub consume_event: bool,
}

/// Rule scope.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleScopeDef {
    Global,
    #[default]
    Local,
    View,
}

/// FRE asset — top-level `.fre.ron` schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreAsset {
    #[serde(default)]
    pub scope: RuleScopeDef,
    #[serde(default, serialize_with = "crate::ordered_map::serialize_ordered_map")]
    pub enums: HashMap<String, Vec<String>>,
    #[serde(default, serialize_with = "crate::ordered_map::serialize_ordered_map")]
    pub facts: HashMap<String, FactValueDef>,
    #[serde(default)]
    pub rules: Vec<RuleDef>,
}

// ============================================================================
// Default helpers
// ============================================================================

fn default_enabled() -> bool {
    true
}

fn default_true() -> bool {
    true
}

fn default_consume_event() -> bool {
    true
}
