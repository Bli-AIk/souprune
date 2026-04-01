//! Evaluates expressions and derived values used by RON-defined View layouts.
//!
//! 负责求值 RON 定义的 View 布局里用到的表达式与派生值。
//!
//! Acts as the calculation layer for dynamic view data. It preprocesses
//! fact references, repeat variables, array lookups, and helper functions so
//! layout fields such as position, color, conditionals, and text bindings can
//! react to the current game state.
//!
//! 动态 View 数据的计算层。它预处理 fact 引用、repeat 变量、
//! 数组查找和辅助函数，让布局里的位置、颜色、条件判断以及文本绑定等字段
//! 可以根据当前游戏状态实时变化。

use super::super::layout::FloatOrExpr;
use super::parsing::{PlayerDataView, RepeatContext};
use crate::core::sequencer::chapter_schema::Value;
use crate::core::view::expr_eval::{create_eval_callback, preprocess_varname};
use bevy::prelude::*;
use std::collections::BTreeMap;
use std::sync::LazyLock;

// ============================================================================
// Cached Regex Patterns (compiled once)
// ============================================================================

pub(super) static ARRAY_INDEX_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\$([a-zA-Z_][a-zA-Z0-9_:]*)\[(\d+)\]").unwrap());

pub(super) static DOLLAR_VAR_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\$([a-zA-Z_][a-zA-Z0-9_:]*)").unwrap());

pub(super) static FACT_OR_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"fact_or\s*\(\s*['"]([^'"]+)['"]\s*,\s*([^)]+)\s*\)"#).unwrap()
});

pub(super) static FACT_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"fact\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap());

pub(super) static DYNAMIC_INDEX_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"\$([a-zA-Z_][a-zA-Z0-9_:]*)\[@([a-zA-Z_][a-zA-Z0-9_]*)\]").unwrap()
});

pub(super) static REPEAT_VAR_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"@([a-zA-Z_][a-zA-Z0-9_]*)").unwrap());

/// `max_strlen($list_name)` — maximum display width of strings in a StringList.
pub(super) static MAX_STRLEN_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"max_strlen\(\$([a-zA-Z_][a-zA-Z0-9_:]*)\)").unwrap());

// ============================================================================
// Fact Expression Preprocessing
// ============================================================================

/// Format a fact value for use in expressions.
/// - Integers are output as is
/// - Floats are output with decimal point
/// - Booleans are output as "true" or "false"
/// - Strings are output as quoted strings
///
/// 格式化 fact 值以在表达式中使用。
/// - 整数直接输出
/// - 浮点数带小数点输出
/// - 布尔值输出为 "true" 或 "false"
/// - 字符串作为带引号的字符串输出
/// - 列表输出其长度（用于表达式计算）
fn format_fact_for_expr(value: &bevy_fact_rule_event::FactValue) -> String {
    use bevy_fact_rule_event::FactValue;
    match value {
        FactValue::Int(i) => i.to_string(),
        FactValue::Float(f) => format!("{}", f),
        FactValue::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        FactValue::String(s) => format!("\"{}\"", s),
        FactValue::StringList(list) => list.len().to_string(),
        FactValue::IntList(list) => list.len().to_string(),
        FactValue::FloatList(list) => list.len().to_string(),
        FactValue::BoolList(list) => list.len().to_string(),
    }
}

/// Get a fact value formatted for expression evaluation.
/// Returns the default value string if fact not found.
///
/// 获取格式化用于表达式求值的 fact 值。
/// 如果找不到 fact 则返回默认值字符串。
fn get_fact_for_expr(player_data: &PlayerDataView, key: &str, default: &str) -> String {
    if let Some(value) = player_data.get_fact(key) {
        format_fact_for_expr(value)
    } else {
        default.to_string()
    }
}

/// Get an array element value for expression substitution.
/// Returns the element at the given index, or the default if not found.
///
/// 获取数组元素值用于表达式替换。
/// 返回给定索引处的元素，如果未找到则返回默认值。
fn get_array_element_for_expr(
    player_data: &PlayerDataView,
    array_name: &str,
    index: usize,
    default: &str,
) -> String {
    // Try IntList first
    if let Some(list) = player_data.get_fact_int_list(array_name)
        && let Some(value) = list.get(index)
    {
        return value.to_string();
    }
    // Then try StringList
    if let Some(list) = player_data.get_fact_string_list(array_name)
        && let Some(value) = list.get(index)
    {
        return value.clone();
    }
    // Debug level since out-of-bounds access is common for template elements
    // that are conditionally visible based on array length
    debug!(
        "Array element '{}[{}]' not found, using default '{}'",
        array_name, index, default
    );
    default.to_string()
}

/// String display width: ASCII = 1, non-ASCII = 2.
///
/// 字符串显示宽度：半角=1，全角=2。
fn unicode_display_width(s: &str) -> usize {
    s.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

/// Compute the maximum display width across all strings in a StringList fact.
///
/// 计算 StringList 事实中所有字符串的最大显示宽度。
fn compute_max_strlen(player_data: &PlayerDataView, list_name: &str) -> f64 {
    player_data
        .get_fact_string_list(list_name)
        .map(|list| {
            list.iter()
                .map(|s| unicode_display_width(s))
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0) as f64
}

/// Preprocess `max_strlen($list)` calls by replacing them with numeric values.
///
/// 预处理 `max_strlen($list)` 调用，替换为数值。
fn preprocess_list_aggregates(expr: &str, player_data: &PlayerDataView) -> String {
    MAX_STRLEN_RE
        .replace_all(expr, |caps: &regex::Captures| {
            let list_name = &caps[1];
            compute_max_strlen(player_data, list_name).to_string()
        })
        .to_string()
}

/// Preprocess expression string to handle `$name` and `fact('name')` syntax.
/// - `$name` is converted to the actual fact value
/// - `fact('name')` is converted to the actual fact value
/// - `fact_or('name', default)` is converted to the actual fact value or default
///
/// 预处理表达式字符串以处理 `$name` 和 `fact('name')` 语法。
/// - `$name` 会被转换为实际的 fact 值
/// - `$array[index]` 会被转换为数组中对应索引的值
/// - `fact('name')` 会被转换为实际的 fact 值
/// - `fact_or('name', default)` 会被转换为实际的 fact 值或默认值
pub(crate) fn preprocess_fact_expressions(expr: &str, player_data: &PlayerDataView) -> String {
    let mut result = expr.to_string();

    // Step 0: List aggregate functions (before $var replacement)
    // 步骤 0: 列表聚合函数（在 $var 替换前处理）
    result = preprocess_list_aggregates(&result, player_data);

    // Handle $array[index] syntax first: replace $word[number] with the array element value
    // 首先处理 $array[index] 语法：将 $word[number] 替换为数组元素值
    result = ARRAY_INDEX_RE
        .replace_all(&result, |caps: &regex::Captures| {
            let array_name = &caps[1];
            let index: usize = caps[2].parse().unwrap_or(0);
            get_array_element_for_expr(player_data, array_name, index, "0")
        })
        .to_string();

    // Handle $name syntax: replace $word with the fact value (preserving type)
    // 处理 $name 语法：将 $word 替换为 fact 值（保留类型）
    // Note: Supports namespace format like $player:hp
    result = DOLLAR_VAR_RE
        .replace_all(&result, |caps: &regex::Captures| {
            let key = &caps[1];
            get_fact_for_expr(player_data, key, "0")
        })
        .to_string();

    // Handle fact_or('name', default) syntax
    // 处理 fact_or('name', default) 语法
    result = FACT_OR_RE
        .replace_all(&result, |caps: &regex::Captures| {
            let key = &caps[1];
            let default_str = caps[2].trim();
            get_fact_for_expr(player_data, key, default_str)
        })
        .to_string();

    // Handle fact('name') syntax
    // 处理 fact('name') 语法
    result = FACT_RE
        .replace_all(&result, |caps: &regex::Captures| {
            let key = &caps[1];
            get_fact_for_expr(player_data, key, "0")
        })
        .to_string();

    result
}

/// Preprocess expression string with support for repeat context variables.
/// Extends `preprocess_fact_expressions` with:
/// - `@var` syntax for repeat context variables
/// - `$array[@i]` dynamic index syntax
///
/// 预处理表达式字符串，支持重复上下文变量。
/// 扩展 `preprocess_fact_expressions` 以支持：
/// - `@var` 语法用于重复上下文变量
/// - `$array[@i]` 动态索引语法
pub fn preprocess_fact_expressions_with_repeat(
    expr: &str,
    player_data: &PlayerDataView,
    repeat_ctx: Option<&RepeatContext>,
) -> String {
    let mut result = expr.to_string();

    // Step 0: List aggregate functions (before any variable replacement)
    // 步骤 0: 列表聚合函数（在任何变量替换之前）
    result = preprocess_list_aggregates(&result, player_data);

    // Step 1: Handle $array[@var] dynamic index syntax FIRST (before @var replacement)
    // 步骤 1: 首先处理 $array[@var] 动态索引语法（在 @var 替换之前）
    // Note: Supports namespace format like $player:inventory[@i]
    if let Some(ctx) = repeat_ctx {
        result = DYNAMIC_INDEX_RE
            .replace_all(&result, |caps: &regex::Captures| {
                let array_name = &caps[1];
                let index_var = &caps[2];
                let index = if index_var == "i" || index_var == "index" {
                    ctx.index
                } else {
                    ctx.variables
                        .get(index_var)
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0)
                };
                get_array_element_for_expr(player_data, array_name, index, "0")
            })
            .to_string();
    }

    // Step 2: Handle @var repeat context variables
    // 步骤 2: 处理 @var 重复上下文变量
    if let Some(ctx) = repeat_ctx {
        result = REPEAT_VAR_RE
            .replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                if var_name == "i" || var_name == "index" {
                    ctx.index.to_string()
                } else if var_name == "time" {
                    // @time is handled separately by fasteval context
                    format!("@{}", var_name)
                } else {
                    ctx.variables
                        .get(var_name)
                        .cloned()
                        .unwrap_or_else(|| "0".to_string())
                }
            })
            .to_string();
    }

    // Step 3: Continue with existing processing
    // 步骤 3: 继续现有处理
    preprocess_fact_expressions(&result, player_data)
}

// ============================================================================
// Expression Evaluation Functions
// ============================================================================

fn evaluate_index_expression(expr: &str, player_data: &PlayerDataView) -> usize {
    let expr = expr.trim();

    if expr == "inventory.len()" {
        return player_data
            .get_fact_string_list("player:inventory")
            .map(|list| list.len())
            .unwrap_or(0);
    }

    if expr == "inventory_capacity" {
        return player_data
            .get_fact_int("player:inventory_capacity")
            .unwrap_or(8) as usize;
    }

    if expr.starts_with("min(") && expr.ends_with(")") {
        let inner = &expr[4..expr.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let a = evaluate_index_expression(parts[0], player_data);
            let b = evaluate_index_expression(parts[1], player_data);
            return a.min(b);
        }
    }

    if expr.starts_with("max(") && expr.ends_with(")") {
        let inner = &expr[4..expr.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let a = evaluate_index_expression(parts[0], player_data);
            let b = evaluate_index_expression(parts[1], player_data);
            return a.max(b);
        }
    }

    if let Ok(value) = expr.parse::<usize>() {
        return value;
    }

    warn!(
        "Unable to evaluate index expression: {}, defaulting to 1",
        expr
    );
    1
}

/// Evaluate a `visible_when` expression to determine visibility.
/// Returns true if the element should be visible, false otherwise.
///
/// Supports:
/// - `$name` syntax (replaced with fact value)
/// - `fact('name')` function
/// - `fact_or('name', default)` function
/// - Comparison operators: ==, !=, <, >, <=, >=
/// - Boolean operators: &&, ||, !
/// - Numeric literals and boolean literals (true, false)
///
/// Examples:
/// - "fact('depth') == 1"
/// - "$selection == 0"
/// - "fact_or('active', true)"
/// - "$depth >= 1 && $depth <= 2"
///
/// 计算 `visible_when` 表达式以确定可见性。
/// 如果元素应该可见则返回 true，否则返回 false。
pub fn evaluate_visible_when(expr: &str, player_data: &PlayerDataView) -> bool {
    // Handle empty expression as always visible
    let expr = expr.trim();
    if expr.is_empty() || expr == "true" {
        return true;
    }
    if expr == "false" {
        return false;
    }

    // Preprocess fact expressions - this handles $var syntax
    // 预处理事实表达式 - 处理 $var 语法
    let processed_expr = preprocess_fact_expressions(expr, player_data);

    // Preprocess special functions
    // 预处理特殊函数
    let is_empty = player_data
        .get_fact_string_list("player:inventory")
        .map(|list| list.is_empty())
        .unwrap_or(true);
    let processed_expr =
        processed_expr.replace("inventory_is_empty()", if is_empty { "1" } else { "0" });

    // Preprocess boolean literals for fasteval compatibility
    // 预处理布尔字面量以兼容 fasteval
    let processed_expr = processed_expr.replace("true", "1").replace("false", "0");

    // Preprocess variable names for fasteval compatibility
    let processed_expr = preprocess_varname(&processed_expr);

    let vars: BTreeMap<String, f64> = BTreeMap::new();
    let mut cb = create_eval_callback(&vars);

    match fasteval::ez_eval(&processed_expr, &mut cb) {
        Ok(val) => val != 0.0,
        Err(e) => {
            warn!(
                "Failed to evaluate visible_when expression '{}' (processed: '{}'): {}",
                expr, processed_expr, e
            );
            true // Default to visible on error
        }
    }
}

pub fn evaluate_float_expr(
    expr: &FloatOrExpr,
    player_data: &PlayerDataView,
    time: Option<f64>,
) -> f32 {
    match expr {
        Value::Static(v) => *v,
        Value::Expr(expr_str) => {
            // Preprocess fact expressions before evaluation
            // 在求值前预处理 fact 表达式
            let processed_expr = preprocess_fact_expressions(expr_str, player_data);

            // Preprocess variable names for fasteval compatibility
            let processed_expr = preprocess_varname(&processed_expr);

            let mut vars: BTreeMap<String, f64> = BTreeMap::new();

            // Set @time variable
            vars.insert("__at_time".to_string(), time.unwrap_or(0.0));

            let mut cb = create_eval_callback(&vars);

            match fasteval::ez_eval(&processed_expr, &mut cb) {
                Ok(val) => val as f32,
                Err(e) => {
                    warn!("Failed to evaluate expression '{}': {}", expr_str, e);
                    0.0
                }
            }
        }
    }
}

/// Evaluate a float expression with optional repeat context support.
/// This handles @var and $array[@i] syntax when a repeat context is provided.
///
/// 计算支持可选重复上下文的浮点表达式。
/// 当提供重复上下文时，处理 @var 和 $array[@i] 语法。
pub fn evaluate_float_expr_with_repeat(
    expr: &FloatOrExpr,
    player_data: &PlayerDataView,
    time: Option<f64>,
    repeat_ctx: Option<&RepeatContext>,
) -> f32 {
    match expr {
        Value::Static(v) => *v,
        Value::Expr(expr_str) => {
            // Preprocess with repeat context support
            // 使用重复上下文支持进行预处理
            let processed_expr =
                preprocess_fact_expressions_with_repeat(expr_str, player_data, repeat_ctx);

            // Preprocess variable names for fasteval compatibility
            let processed_expr = preprocess_varname(&processed_expr);

            let mut vars: BTreeMap<String, f64> = BTreeMap::new();

            // Set @time variable
            vars.insert("__at_time".to_string(), time.unwrap_or(0.0));

            let mut cb = create_eval_callback(&vars);

            match fasteval::ez_eval(&processed_expr, &mut cb) {
                Ok(val) => val as f32,
                Err(e) => {
                    warn!(
                        "Failed to evaluate expression '{}' (processed: '{}'): {}",
                        expr_str, processed_expr, e
                    );
                    0.0
                }
            }
        }
    }
}

/// Evaluate a float expression string with support for @current variable.
/// This is used by the tween system where @current represents the current value.
///
/// 计算支持 @current 变量的浮点表达式字符串。
/// 这由 tween 系统使用，其中 @current 代表当前值。
pub fn evaluate_float_expr_with_current(
    expr_str: &str,
    current_value: Option<f32>,
    _player_data: &PlayerDataView,
    time: Option<f64>,
) -> f32 {
    // Preprocess variable names for fasteval compatibility
    let processed_expr = preprocess_varname(expr_str);

    let mut vars: BTreeMap<String, f64> = BTreeMap::new();

    // Register @current variable
    if let Some(current) = current_value {
        debug!(
            "[evaluate_float_expr_with_current] Setting @current = {}",
            current
        );
        vars.insert("__at_current".to_string(), current as f64);
    } else {
        debug!("[evaluate_float_expr_with_current] Setting @current = 0.0 (no current value)");
        vars.insert("__at_current".to_string(), 0.0);
    }

    // Set @time variable
    vars.insert("__at_time".to_string(), time.unwrap_or(0.0));

    debug!(
        "[evaluate_float_expr_with_current] Evaluating expression: '{}'",
        expr_str
    );

    let mut cb = create_eval_callback(&vars);

    match fasteval::ez_eval(&processed_expr, &mut cb) {
        Ok(val) => {
            debug!("[evaluate_float_expr_with_current] Result: {} (float)", val);
            val as f32
        }
        Err(e) => {
            warn!("Failed to evaluate expression '{}': {}", expr_str, e);
            0.0
        }
    }
}

// ============================================================================
// Condition Evaluation
// ============================================================================

pub fn evaluate_condition(condition: &str, player_data: &PlayerDataView) -> bool {
    // Helper closure to get inventory
    let get_inventory = || {
        player_data
            .get_fact_string_list("player:inventory")
            .unwrap_or_default()
    };

    match condition {
        "player.inventory.is_empty" => get_inventory().is_empty(),
        "player.inventory.is_not_empty" => !get_inventory().is_empty(),
        _ if condition.starts_with("player.") => {
            let parts: Vec<&str> = condition.split('.').collect();
            evaluate_player_property(&parts, player_data)
        }
        _ => false,
    }
}

/// Evaluate a player property condition like "player.hp.is_low".
fn evaluate_player_property(parts: &[&str], player_data: &PlayerDataView) -> bool {
    if parts.len() < 3 {
        return false;
    }
    match (parts[1], parts[2]) {
        ("hp", "is_low") => {
            let hp = player_data.get_fact_int("player:hp").unwrap_or(0);
            let hp_max = player_data.get_fact_int("player:hp_max").unwrap_or(1);
            hp < hp_max / 4
        }
        ("hp", "is_critical") => {
            let hp = player_data.get_fact_int("player:hp").unwrap_or(0);
            hp <= 1
        }
        ("gold", "is_zero") => player_data.get_fact_int("player:gold").unwrap_or(0) == 0,
        _ => false,
    }
}

/// Evaluate transition condition for InteractiveLayer.
///
/// 为 InteractiveLayer 评估转换条件。
///
/// Supports conditions like:
/// - "index == 0"
/// - "index == 1 && !player.inventory.is_empty"
/// - "index == 0 && player.inventory.is_empty"
///
/// 支持的条件格式：
/// - "index == 0"
/// - "index == 1 && !player.inventory.is_empty"
/// - "index == 0 && player.inventory.is_empty"
pub fn evaluate_transition_condition_unified(
    condition: &str,
    index: usize,
    player_data: &PlayerDataView,
) -> bool {
    let condition = condition.trim();

    // Helper to get inventory
    let get_inventory = || {
        player_data
            .get_fact_string_list("player:inventory")
            .unwrap_or_default()
    };

    // Handle "index == N" pattern with optional additional conditions
    let Some(rest) = condition.strip_prefix("index == ") else {
        // For other conditions, delegate to the existing evaluate_condition
        return evaluate_condition(condition, player_data);
    };

    // Split by &&
    let parts: Vec<&str> = rest.split("&&").map(|s| s.trim()).collect();

    // First part should be the index number
    let Some(index_part) = parts.first() else {
        return false;
    };
    let Ok(target_index) = index_part.parse::<usize>() else {
        return false;
    };

    if index != target_index {
        return false;
    }

    // Check additional conditions
    for part in parts.iter().skip(1) {
        if *part == "!player.inventory.is_empty" && get_inventory().is_empty() {
            return false;
        }
        if *part == "player.inventory.is_empty" && !get_inventory().is_empty() {
            return false;
        }
        // Add more conditions as needed
    }

    true
}

/// Evaluate a DynamicColor tuple by evaluating each component expression.
/// Returns (r, g, b, a) as f32 values.
///
/// 评估 DynamicColor 元组，对每个组件表达式求值。
/// 返回 (r, g, b, a) 作为 f32 值。
pub fn evaluate_dynamic_color(
    color: &super::super::layout::serde_types::DynamicColor,
    player_data: &PlayerDataView,
    time: Option<f64>,
) -> (f32, f32, f32, f32) {
    (
        evaluate_float_expr(&color.0, player_data, time),
        evaluate_float_expr(&color.1, player_data, time),
        evaluate_float_expr(&color.2, player_data, time),
        evaluate_float_expr(&color.3, player_data, time),
    )
}

/// Evaluate a fact expression string and return the result as f32.
/// Supports $fact_name syntax for reading from the fact database.
///
/// 评估一个 fact 表达式字符串并返回 f32 结果。
/// 支持 $fact_name 语法从 fact 数据库读取。
pub fn evaluate_fact_expression(expr: &str, player_data: &PlayerDataView) -> Option<f32> {
    // Preprocess fact expressions before evaluation
    let processed_expr = preprocess_fact_expressions(expr, player_data);

    let vars: BTreeMap<String, f64> = BTreeMap::new();
    let mut cb = create_eval_callback(&vars);

    match fasteval::ez_eval(&processed_expr, &mut cb) {
        Ok(val) => Some(val as f32),
        Err(e) => {
            warn!(
                "[evaluate_fact_expression] Failed to evaluate '{}' (processed: '{}'): {}",
                expr, processed_expr, e
            );
            None
        }
    }
}

// ============================================================================
// Val Resolution
// ============================================================================

/// Resolve a Value<f32> to an actual f32 value.
///
/// 将 Value<f32> 解析为实际的 f32 值。
pub fn resolve_val_f32(
    val: &crate::core::sequencer::chapter_schema::Value<f32>,
    current_value: Option<f32>,
    player_data: &PlayerDataView,
    time: Option<f64>,
) -> f32 {
    match val {
        crate::core::sequencer::chapter_schema::Value::Static(v) => *v,
        crate::core::sequencer::chapter_schema::Value::Expr(expr_str) => {
            // Special case: pure @current
            if expr_str == "@current" {
                return current_value.unwrap_or(0.0);
            }

            // For expressions containing @current, use the full expression evaluator
            // with @current set as a variable
            evaluate_float_expr_with_current(expr_str, current_value, player_data, time)
        }
    }
}

/// Resolve a Value<bool> to an actual bool value.
///
/// 将 Value<bool> 解析为实际的 bool 值。
pub fn resolve_val_bool(
    val: &crate::core::sequencer::chapter_schema::Value<bool>,
    player_data: &PlayerDataView,
) -> bool {
    match val {
        crate::core::sequencer::chapter_schema::Value::Static(v) => *v,
        crate::core::sequencer::chapter_schema::Value::Expr(expr_str) => {
            // Evaluate condition expression
            evaluate_condition(expr_str, player_data)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fact_rule_event::{FactDatabase, FactValue, LayeredFactDatabase};

    fn make_player_data_with_names(names: Vec<&str>) -> (LayeredFactDatabase, FactDatabase) {
        let mut layered = LayeredFactDatabase::default();
        let local = FactDatabase::default();
        layered.set(
            "enemy_names",
            FactValue::StringList(names.into_iter().map(|s| s.to_string()).collect()),
        );
        (layered, local)
    }

    #[test]
    fn test_unicode_display_width_ascii() {
        assert_eq!(unicode_display_width("Mush"), 4);
        assert_eq!(unicode_display_width("Froggit"), 7);
        assert_eq!(unicode_display_width(""), 0);
    }

    #[test]
    fn test_unicode_display_width_cjk() {
        assert_eq!(unicode_display_width("蘑菇怪"), 6);
        assert_eq!(unicode_display_width("AB蘑菇"), 6);
    }

    #[test]
    fn test_max_strlen_basic() {
        let (layered, local) = make_player_data_with_names(vec!["Mush", "Froggit", "Soup"]);
        let pv = PlayerDataView::with_local_facts(&layered, &local);
        assert_eq!(compute_max_strlen(&pv, "enemy_names"), 7.0);
    }

    #[test]
    fn test_max_strlen_empty_list() {
        let (layered, local) = make_player_data_with_names(vec![]);
        let pv = PlayerDataView::with_local_facts(&layered, &local);
        assert_eq!(compute_max_strlen(&pv, "enemy_names"), 0.0);
    }

    #[test]
    fn test_max_strlen_missing_fact() {
        let layered = LayeredFactDatabase::default();
        let local = FactDatabase::default();
        let pv = PlayerDataView::with_local_facts(&layered, &local);
        assert_eq!(compute_max_strlen(&pv, "nonexistent"), 0.0);
    }

    #[test]
    fn test_max_strlen_preprocessing() {
        let (layered, local) = make_player_data_with_names(vec!["Mush", "Froggit"]);
        let pv = PlayerDataView::with_local_facts(&layered, &local);
        let result = preprocess_list_aggregates("max_strlen($enemy_names)", &pv);
        assert_eq!(result, "7");
    }

    #[test]
    fn test_max_strlen_in_expression() {
        let (layered, local) = make_player_data_with_names(vec!["Hello", "Hi"]);
        let pv = PlayerDataView::with_local_facts(&layered, &local);
        // "Hello" = 5 chars, 15 * 5 - 125 = -50
        let result = preprocess_fact_expressions("15 * max_strlen($enemy_names) - 125", &pv);
        assert_eq!(result, "15 * 5 - 125");
    }
}
