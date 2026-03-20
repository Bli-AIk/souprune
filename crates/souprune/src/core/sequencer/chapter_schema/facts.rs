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
