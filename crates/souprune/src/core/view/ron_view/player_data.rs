//! Fact data view for accessing facts from LayeredFactDatabase.
//!
//! 事实数据视图，用于从 LayeredFactDatabase 访问事实。
//!
//! This module provides a unified view that checks local facts first, then layered database.
//! All fact keys and default values should be defined in Mod configuration files.
//!
//! 本模块提供统一视图，先检查局部事实，再检查分层数据库。
//! 所有事实键名和默认值应在 Mod 配置文件中定义。

use bevy::prelude::*;
use bevy_fact_rule_event::{FactDatabase, FactValue, LayeredFactDatabase};
use std::collections::HashMap;

/// Resolver function for computed data paths (e.g. "player.total_attack").
/// Takes the fact database, optional local facts, and mortar string table.
///
/// 计算数据路径的解析器函数（如 "player.total_attack"）。
type DataPathResolverFn = Box<
    dyn Fn(
            &LayeredFactDatabase,
            Option<&FactDatabase>,
            &crate::extra::mortar::MortarStringTable,
        ) -> String
        + Send
        + Sync,
>;

/// Resolver function for view conditions (e.g. "player.hp.is_low").
/// Takes the fact database and optional local facts.
///
/// 视图条件的解析器函数（如 "player.hp.is_low"）。
type ConditionResolverFn =
    Box<dyn Fn(&LayeredFactDatabase, Option<&FactDatabase>) -> bool + Send + Sync>;

/// Registry of computed data path resolvers.
/// Preset modules register game-specific resolvers here.
///
/// 计算数据路径解析器注册表。预设模块在此注册游戏特定的解析器。
#[derive(Resource)]
pub struct DataPathResolvers {
    resolvers: HashMap<String, DataPathResolverFn>,
}

impl Default for DataPathResolvers {
    fn default() -> Self {
        Self {
            resolvers: HashMap::new(),
        }
    }
}

impl DataPathResolvers {
    pub fn register(
        &mut self,
        path: impl Into<String>,
        resolver: impl Fn(
            &LayeredFactDatabase,
            Option<&FactDatabase>,
            &crate::extra::mortar::MortarStringTable,
        ) -> String
        + Send
        + Sync
        + 'static,
    ) {
        self.resolvers.insert(path.into(), Box::new(resolver));
    }

    pub fn resolve(
        &self,
        path: &str,
        db: &LayeredFactDatabase,
        local_facts: Option<&FactDatabase>,
        mortar_strings: &crate::extra::mortar::MortarStringTable,
    ) -> Option<String> {
        self.resolvers
            .get(path)
            .map(|f| f(db, local_facts, mortar_strings))
    }
}

/// Registry of view condition resolvers.
/// Preset modules register game-specific conditions here.
///
/// 视图条件解析器注册表。预设模块在此注册游戏特定的条件。
#[derive(Resource)]
pub struct ConditionResolvers {
    resolvers: HashMap<String, ConditionResolverFn>,
}

impl Default for ConditionResolvers {
    fn default() -> Self {
        Self {
            resolvers: HashMap::new(),
        }
    }
}

impl ConditionResolvers {
    pub fn register(
        &mut self,
        condition: impl Into<String>,
        resolver: impl Fn(&LayeredFactDatabase, Option<&FactDatabase>) -> bool + Send + Sync + 'static,
    ) {
        self.resolvers.insert(condition.into(), Box::new(resolver));
    }

    pub fn resolve(
        &self,
        condition: &str,
        db: &LayeredFactDatabase,
        local_facts: Option<&FactDatabase>,
    ) -> Option<bool> {
        self.resolvers.get(condition).map(|f| f(db, local_facts))
    }
}

/// Resolver function for expression functions used in fasteval (e.g. "inventory_is_empty()").
/// Returns a float value for use in mathematical expressions.
///
/// 表达式函数的解析器（如 "inventory_is_empty()"），用于 fasteval。
/// 返回浮点值，用于数学表达式中。
type ExprFunctionResolverFn =
    Box<dyn Fn(&LayeredFactDatabase, Option<&FactDatabase>) -> f64 + Send + Sync>;

/// Registry of expression function resolvers for `visible_when` and other fasteval expressions.
/// Maps function names (without parens) to resolvers returning f64 values.
///
/// `visible_when` 及其他 fasteval 表达式中的函数解析器注册表。
/// 将函数名（无括号）映射到返回 f64 值的解析器。
#[derive(Resource)]
pub struct ExprFunctionResolvers {
    resolvers: HashMap<String, ExprFunctionResolverFn>,
}

impl Default for ExprFunctionResolvers {
    fn default() -> Self {
        Self {
            resolvers: HashMap::new(),
        }
    }
}

impl ExprFunctionResolvers {
    pub fn register(
        &mut self,
        name: impl Into<String>,
        resolver: impl Fn(&LayeredFactDatabase, Option<&FactDatabase>) -> f64 + Send + Sync + 'static,
    ) {
        self.resolvers.insert(name.into(), Box::new(resolver));
    }

    /// Evaluate a function by name. Returns None if not registered.
    pub fn evaluate(
        &self,
        name: &str,
        db: &LayeredFactDatabase,
        local_facts: Option<&FactDatabase>,
    ) -> Option<f64> {
        self.resolvers.get(name).map(|f| f(db, local_facts))
    }

    /// Preprocess expression string: replace all `func_name()` with evaluated values.
    pub fn preprocess_expr(
        &self,
        expr: &str,
        db: &LayeredFactDatabase,
        local_facts: Option<&FactDatabase>,
    ) -> String {
        let mut result = expr.to_string();
        for (name, resolver) in &self.resolvers {
            let pattern = format!("{}()", name);
            if result.contains(&pattern) {
                let val = resolver(db, local_facts);
                result = result.replace(&pattern, &format!("{}", val as i64));
            }
        }
        result
    }
}

/// Helper struct to read facts from LayeredFactDatabase with optional local facts.
/// This provides a unified view for the expression evaluation system.
///
/// 从 LayeredFactDatabase 读取事实的辅助结构体，支持可选的局部事实。
/// 为表达式求值系统提供统一视图。
///
/// ## Fact Resolution Priority
/// 1. local_facts (View-specific facts from ViewRoot)
/// 2. scene facts (from LayeredFactDatabase)
/// 3. global facts (from LayeredFactDatabase)
///
/// ## 事实解析优先级
/// 1. local_facts（来自 ViewRoot 的 View 特定事实）
/// 2. scene 事实（来自 LayeredFactDatabase）
/// 3. global 事实（来自 LayeredFactDatabase）
///
/// ## Usage Example / 使用示例
/// ```ignore
/// let player_data = PlayerDataView::new(&layered_db);
///
/// // Get fact value directly / 直接获取事实值
/// let hp = player_data.get_fact_int("player:hp").unwrap_or(0);
/// let name = player_data.get_fact_string("player:name").unwrap_or_default();
///
/// // Get fact with fallback and warning / 获取事实值，缺失时发出警告
/// let hp = player_data.get_fact_int_or("player:hp", 0);
/// ```
pub struct PlayerDataView<'a> {
    db: &'a LayeredFactDatabase,
    /// Optional local facts from ViewRoot (View-specific facts)
    /// 来自 ViewRoot 的可选局部事实（View 特定的事实）
    local_facts: Option<&'a FactDatabase>,
    data_path_resolvers: Option<&'a DataPathResolvers>,
    condition_resolvers: Option<&'a ConditionResolvers>,
    expr_function_resolvers: Option<&'a ExprFunctionResolvers>,
}

impl<'a> PlayerDataView<'a> {
    pub fn new(db: &'a LayeredFactDatabase) -> Self {
        Self {
            db,
            local_facts: None,
            data_path_resolvers: None,
            condition_resolvers: None,
            expr_function_resolvers: None,
        }
    }

    /// Create a view with local facts from a ViewRoot.
    ///
    /// 创建一个带有来自 ViewRoot 局部事实的视图。
    pub fn with_local_facts(db: &'a LayeredFactDatabase, local_facts: &'a FactDatabase) -> Self {
        Self {
            db,
            local_facts: Some(local_facts),
            data_path_resolvers: None,
            condition_resolvers: None,
            expr_function_resolvers: None,
        }
    }

    /// Set data path resolvers for this view.
    pub fn set_data_path_resolvers(&mut self, resolvers: &'a DataPathResolvers) {
        self.data_path_resolvers = Some(resolvers);
    }

    /// Set condition resolvers for this view.
    pub fn set_condition_resolvers(&mut self, resolvers: &'a ConditionResolvers) {
        self.condition_resolvers = Some(resolvers);
    }

    /// Builder: attach all resolver registries.
    pub fn with_resolvers(
        mut self,
        data_path: Option<&'a DataPathResolvers>,
        conditions: Option<&'a ConditionResolvers>,
    ) -> Self {
        self.data_path_resolvers = data_path;
        self.condition_resolvers = conditions;
        self
    }

    /// Builder: attach expression function resolvers.
    pub fn with_expr_functions(mut self, resolvers: Option<&'a ExprFunctionResolvers>) -> Self {
        self.expr_function_resolvers = resolvers;
        self
    }

    /// Resolve a computed data path using registered resolvers.
    pub fn resolve_data_path(
        &self,
        path: &str,
        mortar_strings: &crate::extra::mortar::MortarStringTable,
    ) -> Option<String> {
        self.data_path_resolvers
            .and_then(|r| r.resolve(path, self.db, self.local_facts, mortar_strings))
    }

    /// Resolve a condition using registered resolvers.
    pub fn resolve_condition(&self, condition: &str) -> Option<bool> {
        self.condition_resolvers
            .and_then(|r| r.resolve(condition, self.db, self.local_facts))
    }

    /// Preprocess expression string by replacing registered function calls.
    ///
    /// 通过替换已注册的函数调用来预处理表达式字符串。
    pub fn preprocess_expr_functions(&self, expr: &str) -> String {
        if let Some(resolvers) = self.expr_function_resolvers {
            resolvers.preprocess_expr(expr, self.db, self.local_facts)
        } else {
            expr.to_string()
        }
    }

    /// Get the underlying LayeredFactDatabase reference.
    ///
    /// 获取底层 LayeredFactDatabase 引用。
    pub fn db(&self) -> &'a LayeredFactDatabase {
        self.db
    }

    /// Get optional local facts reference.
    ///
    /// 获取可选的局部事实引用。
    pub fn local_facts(&self) -> Option<&'a FactDatabase> {
        self.local_facts
    }

    /// Get a fact value with priority: local_facts -> scene -> global.
    ///
    /// 获取事实值，优先级为：local_facts -> scene -> global。
    pub fn get_fact(&self, key: &str) -> Option<&FactValue> {
        // First check local facts
        if let Some(local) = self.local_facts
            && let Some(value) = local.get_by_str(key)
        {
            return Some(value);
        }
        // Then check layered database (scene -> global)
        self.db.get_by_str(key)
    }

    /// Get a fact value as f64. Returns None if fact doesn't exist.
    ///
    /// 获取事实值为 f64。如果事实不存在则返回 None。
    pub fn get_fact_float(&self, key: &str) -> Option<f64> {
        self.get_fact(key).map(|value| match value {
            FactValue::Float(f) => *f,
            FactValue::Int(i) => *i as f64,
            FactValue::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            FactValue::String(_) => 0.0,
            FactValue::StringList(list) => list.len() as f64,
            FactValue::IntList(list) => list.len() as f64,
            FactValue::FloatList(list) => list.len() as f64,
            FactValue::BoolList(list) => list.len() as f64,
        })
    }

    /// Get a fact value as f64 with fallback default.
    /// Logs a warning if fact doesn't exist.
    ///
    /// 获取事实值为 f64，带有回退默认值。
    /// 如果事实不存在则记录警告。
    pub fn get_fact_float_or(&self, key: &str, default: f64) -> f64 {
        match self.get_fact_float(key) {
            Some(v) => v,
            None => {
                warn!("Fact '{}' not found, using fallback value {}", key, default);
                default
            }
        }
    }

    /// Get a fact value as i64. Returns None if fact doesn't exist.
    ///
    /// 获取事实值为 i64。如果事实不存在则返回 None。
    pub fn get_fact_int(&self, key: &str) -> Option<i64> {
        self.get_fact(key).and_then(|value| match value {
            FactValue::Int(i) => Some(*i),
            FactValue::Float(f) => Some(*f as i64),
            FactValue::Bool(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        })
    }

    /// Get a fact value as i64 with fallback default.
    /// Logs a warning if fact doesn't exist.
    ///
    /// 获取事实值为 i64，带有回退默认值。
    /// 如果事实不存在则记录警告。
    pub fn get_fact_int_or(&self, key: &str, default: i64) -> i64 {
        match self.get_fact_int(key) {
            Some(v) => v,
            None => {
                warn!("Fact '{}' not found, using fallback value {}", key, default);
                default
            }
        }
    }

    /// Get a fact value as String. Returns None if fact doesn't exist.
    ///
    /// 获取事实值为 String。如果事实不存在则返回 None。
    pub fn get_fact_string(&self, key: &str) -> Option<String> {
        self.get_fact(key).and_then(|value| match value {
            FactValue::String(s) => Some(s.clone()),
            FactValue::Int(i) => Some(i.to_string()),
            FactValue::Float(f) => Some(f.to_string()),
            FactValue::Bool(b) => Some(b.to_string()),
            FactValue::StringList(_)
            | FactValue::IntList(_)
            | FactValue::FloatList(_)
            | FactValue::BoolList(_) => None,
        })
    }

    /// Get a fact value as Vec<String>. Returns None if fact doesn't exist.
    ///
    /// 获取事实值为 Vec<String>。如果事实不存在则返回 None。
    pub fn get_fact_string_list(&self, key: &str) -> Option<Vec<String>> {
        self.get_fact(key).and_then(|value| match value {
            FactValue::StringList(list) => Some(list.clone()),
            _ => None,
        })
    }

    /// Get a fact value as Vec<i64>. Returns None if fact doesn't exist.
    ///
    /// 获取事实值为 Vec<i64>。如果事实不存在则返回 None。
    pub fn get_fact_int_list(&self, key: &str) -> Option<Vec<i64>> {
        self.get_fact(key).and_then(|value| match value {
            FactValue::IntList(list) => Some(list.clone()),
            _ => None,
        })
    }

    /// Get the length of an array fact (StringList or IntList).
    /// The key can be in the format "$name" or just "name".
    ///
    /// 获取数组事实的长度（StringList 或 IntList）。
    /// 键可以是 "$name" 格式或仅 "name"。
    pub fn get_array_length(&self, key: &str) -> Option<usize> {
        // Strip leading $ if present
        let clean_key = key.strip_prefix('$').unwrap_or(key);

        self.get_fact(clean_key).and_then(|value| match value {
            FactValue::StringList(list) => Some(list.len()),
            FactValue::IntList(list) => Some(list.len()),
            _ => None,
        })
    }
}
