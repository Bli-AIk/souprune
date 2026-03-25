//! Defines fact-oriented conditions, mutations, and bindings used by sequence chapters.
//!
//! 定义序列章节里围绕 facts 使用的条件、修改与数据绑定结构。
//!
//! Captures the fact-language portion of the sequence schema: how a
//! chapter tests facts, how it mutates them, and how external data such as FRE
//! files or local layers can be bound into a sequence-driven workflow.
//!
//! 承载的是 sequence schema 里和 facts 有关的那部分语言：章节如何判断
//! facts、如何修改它们，以及如何把 FRE 文件或局部层之类的外部数据绑定进
//! 序列驱动的流程里。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FactCondition {
    Equals { key: String, value: FactValueMatch },
    GreaterThan { key: String, value: i64 },
    LessThan { key: String, value: i64 },
    GreaterOrEqual { key: String, value: i64 },
    LessOrEqual { key: String, value: i64 },
    Exists(String),
    NotExists(String),
    IsTrue(String),
    IsFalse(String),
    And(Vec<FactCondition>),
    Or(Vec<FactCondition>),
    Not(Box<FactCondition>),
    Always,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FactValueMatch {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Expr(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FactModificationDef {
    Set { key: String, value: FactValueMatch },
    Increment { key: String, amount: i64 },
    Remove(String),
    Toggle(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AggregateRule {
    Collect(String),
    CollectKeys(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DataBinding {
    File(String),
    Files(Vec<String>),
    LocalLayer,
    Expr(String),
}
