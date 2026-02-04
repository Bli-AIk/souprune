use super::super::layout::FloatOrExpr;
use crate::app_state::battle::chapter_schema::Val;
use crate::app_state::overworld::OverworldSubState;
use bevy::prelude::*;
use std::collections::HashMap;

// Re-export PlayerDataView for backward compatibility
pub use super::player_data::PlayerDataView;

// ============================================================================
// Repeat Context - Used for repeat syntax in UI elements
// ============================================================================

/// Context for repeat variable substitution in UI elements.
/// Used when spawning repeated UI elements like HP bars.
///
/// repeat 变量替换上下文。
/// 用于生成重复的 UI 元素（如血条）时使用。
#[derive(Clone, Debug, Default)]
pub struct RepeatContext {
    /// Current iteration index
    pub index: usize,
    /// Variable bindings: name -> value
    pub variables: HashMap<String, String>,
}

impl RepeatContext {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            variables: HashMap::new(),
        }
    }

    pub fn with_item(mut self, name: &str, value: String) -> Self {
        self.variables.insert(name.to_string(), value);
        self
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn get_variable(&self, name: &str) -> Option<&str> {
        self.variables.get(name).map(|s| s.as_str())
    }
}

// ============================================================================
// SpriteDef Preprocessing - Resolve repeat variables for DynamicViewElement
// ============================================================================

/// Preprocess a SpriteDef to resolve repeat context variables (@i, $array[@i]).
/// This creates a new SpriteDef where repeat variables are replaced with concrete values,
/// allowing the update system to work without needing repeat context.
///
/// 预处理 SpriteDef 以解析 repeat 上下文变量（@i，$array[@i]）。
/// 这会创建一个新的 SpriteDef，其中 repeat 变量被替换为具体值，
/// 使更新系统无需 repeat 上下文即可工作。
pub fn preprocess_sprite_def_for_repeat(
    sprite_def: &super::super::layout::SpriteDef,
    repeat_ctx: &RepeatContext,
) -> super::super::layout::SpriteDef {
    let mut result = sprite_def.clone();

    // Preprocess transform expressions
    if let Some(ref mut transform) = result.transform {
        if let Some(ref mut translation) = transform.translation {
            translation.0 = preprocess_val_for_repeat(&translation.0, repeat_ctx);
            translation.1 = preprocess_val_for_repeat(&translation.1, repeat_ctx);
            translation.2 = preprocess_val_for_repeat(&translation.2, repeat_ctx);
        }
        if let Some(ref mut scale) = transform.scale {
            scale.0 = preprocess_val_for_repeat(&scale.0, repeat_ctx);
            scale.1 = preprocess_val_for_repeat(&scale.1, repeat_ctx);
            scale.2 = preprocess_val_for_repeat(&scale.2, repeat_ctx);
        }
    }

    result
}

/// Preprocess a single Val<f32> to resolve repeat context variables.
/// If the expression contains @i or $array[@i], they are replaced with concrete values.
///
/// 预处理单个 Val<f32> 以解析 repeat 上下文变量。
/// 如果表达式包含 @i 或 $array[@i]，它们会被替换为具体值。
fn preprocess_val_for_repeat(val: &Val<f32>, repeat_ctx: &RepeatContext) -> Val<f32> {
    match val {
        Val::Static(v) => Val::Static(*v),
        Val::Expr(expr_str) => {
            // Check if expression contains repeat variables
            if !expr_str.contains('@') && !expr_str.contains("[@") {
                // No repeat variables, return as-is
                return Val::Expr(expr_str.clone());
            }

            // Preprocess to replace @i and $array[@i] with concrete indices
            // This converts $array[@i] to $array[0], $array[1], etc.
            // so the update system can evaluate them without repeat context.
            let mut result = expr_str.clone();

            // Step 1: Handle $array[@var] dynamic index syntax FIRST
            // Convert $array[@i] to $array[N] where N is the concrete index
            let dynamic_index_regex =
                regex::Regex::new(r"\$([a-zA-Z_][a-zA-Z0-9_]*)\[@([a-zA-Z_][a-zA-Z0-9_]*)\]")
                    .unwrap();

            result = dynamic_index_regex
                .replace_all(&result, |caps: &regex::Captures| {
                    let array_name = &caps[1];
                    let index_var = &caps[2];
                    let index = if index_var == "i" || index_var == "index" {
                        repeat_ctx.index
                    } else {
                        repeat_ctx
                            .variables
                            .get(index_var)
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(0)
                    };
                    // Return $array[N] format instead of the actual value
                    // This preserves dynamic updates while resolving the repeat index
                    format!("${}[{}]", array_name, index)
                })
                .to_string();

            // Step 2: Handle @var repeat context variables (but not @time)
            let repeat_var_regex = regex::Regex::new(r"@([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
            result = repeat_var_regex
                .replace_all(&result, |caps: &regex::Captures| {
                    let var_name = &caps[1];
                    if var_name == "i" || var_name == "index" {
                        repeat_ctx.index.to_string()
                    } else if var_name == "time" {
                        // @time is handled by evalexpr context, keep as-is
                        format!("@{}", var_name)
                    } else {
                        repeat_ctx
                            .variables
                            .get(var_name)
                            .cloned()
                            .unwrap_or_else(|| "0".to_string())
                    }
                })
                .to_string();

            Val::Expr(result)
        }
    }
}

// ============================================================================
// Lambda Expression Evaluation - Used for text content generation
// ============================================================================

/// Parse and evaluate lambda expressions in text content.
/// Format: {|item, index| in $array => "template" sep "separator"}
///
/// 解析并计算文本内容中的 lambda 表达式。
/// 格式：{|item, index| in $array => "template" sep "separator"}
///
/// Examples:
/// - `{|name| in $enemy_names => "* {name}"}` - simple item iteration
/// - `{|name, i| in $enemy_names => "* {name}" sep "\n"}` - with index and separator
fn evaluate_lambda_expression(expr: &str, player_data: &PlayerDataView) -> Option<String> {
    // Regex to parse lambda expression
    // Format: |item, index| in $array => "template" sep "separator"
    let lambda_regex = regex::Regex::new(
        r#"^\|([a-zA-Z_][a-zA-Z0-9_]*)(?:,\s*([a-zA-Z_][a-zA-Z0-9_]*))?\|\s+in\s+\$([a-zA-Z_][a-zA-Z0-9_]*)\s*=>\s*"([^"]*)"\s*(?:sep\s*"([^"]*)")?$"#
    ).ok()?;

    let caps = lambda_regex.captures(expr)?;

    let item_var = &caps[1];
    let index_var = caps.get(2).map(|m| m.as_str());
    let array_name = &caps[3];
    let template = &caps[4];
    let separator = caps.get(5).map(|m| m.as_str()).unwrap_or("\n");

    // Get array from player data - try StringList first, then IntList
    let array: Vec<String> = if let Some(list) = player_data.get_fact_string_list(array_name) {
        list.clone()
    } else if let Some(list) = player_data.get_fact_int_list(array_name) {
        list.iter().map(|i| i.to_string()).collect()
    } else {
        debug!(
            "Lambda array '{}' not found, returning empty string",
            array_name
        );
        return Some(String::new());
    };

    // Generate output for each element
    let lines: Vec<String> = array
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let mut line = template.to_string();
            // Replace item variable: {item_var}
            line = line.replace(&format!("{{{}}}", item_var), item);
            // Replace index variable if present: {index_var}
            if let Some(idx_var) = index_var {
                line = line.replace(&format!("{{{}}}", idx_var), &i.to_string());
            }
            // Handle escape sequences
            line = line.replace("\\n", "\n");
            line
        })
        .collect();

    Some(lines.join(separator))
}

/// Analyze whether an expression depends on time (@time).
/// Returns true if the expression contains time-dependent elements.
///
/// 分析表达式是否依赖时间 (@time)。
/// 如果表达式包含时间依赖的元素则返回 true。
pub fn expression_depends_on_time(expr: &FloatOrExpr) -> bool {
    match expr {
        Val::Static(_) => false,
        Val::Expr(expr_str) => {
            // Check for @time variable usage
            // 检查 @time 变量的使用
            expr_str.contains("@time")
        }
    }
}

/// Check if a Vec3Tuple (translation or scale) contains time-dependent expressions.
///
/// 检查 Vec3Tuple（translation 或 scale）是否包含时间依赖的表达式。
pub fn vec3_tuple_depends_on_time(tuple: &super::super::layout::serde_types::Vec3Tuple) -> bool {
    expression_depends_on_time(&tuple.0)
        || expression_depends_on_time(&tuple.1)
        || expression_depends_on_time(&tuple.2)
}

pub fn parse_overworld_state(state_str: &str) -> Option<OverworldSubState> {
    match state_str {
        "" | "None" => None,
        name => Some(OverworldSubState::new(name)),
    }
}

fn evaluate_index_expression(expr: &str, player_data: &PlayerDataView) -> usize {
    let expr = expr.trim();

    if expr == "inventory.len()" {
        return player_data
            .get_fact_string_list("player_inventory")
            .map(|list| list.len())
            .unwrap_or(0);
    }

    if expr == "inventory_capacity" {
        return player_data
            .get_fact_int("player_inventory_capacity")
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
/// - 字符串列表输出其长度（用于表达式计算）
/// - 整数列表输出其长度（用于表达式计算）
fn format_fact_for_expr(value: &bevy_fact_rule_event::FactValue) -> String {
    use bevy_fact_rule_event::FactValue;
    match value {
        FactValue::Int(i) => i.to_string(),
        FactValue::Float(f) => format!("{}", f),
        FactValue::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        FactValue::String(s) => format!("\"{}\"", s),
        FactValue::StringList(list) => list.len().to_string(),
        FactValue::IntList(list) => list.len().to_string(),
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
fn preprocess_fact_expressions(expr: &str, player_data: &PlayerDataView) -> String {
    let mut result = expr.to_string();

    // Handle $array[index] syntax first: replace $word[number] with the array element value
    // 首先处理 $array[index] 语法：将 $word[number] 替换为数组元素值
    let array_index_regex = regex::Regex::new(r"\$([a-zA-Z_][a-zA-Z0-9_]*)\[(\d+)\]").unwrap();
    result = array_index_regex
        .replace_all(&result, |caps: &regex::Captures| {
            let array_name = &caps[1];
            let index: usize = caps[2].parse().unwrap_or(0);
            get_array_element_for_expr(player_data, array_name, index, "0")
        })
        .to_string();

    // Handle $name syntax: replace $word with the fact value (preserving type)
    // 处理 $name 语法：将 $word 替换为 fact 值（保留类型）
    let dollar_regex = regex::Regex::new(r"\$([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    result = dollar_regex
        .replace_all(&result, |caps: &regex::Captures| {
            let key = &caps[1];
            get_fact_for_expr(player_data, key, "0")
        })
        .to_string();

    // Handle fact_or('name', default) syntax
    // 处理 fact_or('name', default) 语法
    let fact_or_regex =
        regex::Regex::new(r#"fact_or\s*\(\s*['"]([^'"]+)['"]\s*,\s*([^)]+)\s*\)"#).unwrap();
    result = fact_or_regex
        .replace_all(&result, |caps: &regex::Captures| {
            let key = &caps[1];
            let default_str = caps[2].trim();
            get_fact_for_expr(player_data, key, default_str)
        })
        .to_string();

    // Handle fact('name') syntax
    // 处理 fact('name') 语法
    let fact_regex = regex::Regex::new(r#"fact\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap();
    result = fact_regex
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

    // Step 1: Handle $array[@var] dynamic index syntax FIRST (before @var replacement)
    // 步骤 1: 首先处理 $array[@var] 动态索引语法（在 @var 替换之前）
    if let Some(ctx) = repeat_ctx {
        let dynamic_index_regex =
            regex::Regex::new(r"\$([a-zA-Z_][a-zA-Z0-9_]*)\[@([a-zA-Z_][a-zA-Z0-9_]*)\]").unwrap();

        result = dynamic_index_regex
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
        let repeat_var_regex = regex::Regex::new(r"@([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
        result = repeat_var_regex
            .replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                if var_name == "i" || var_name == "index" {
                    ctx.index.to_string()
                } else if var_name == "time" {
                    // @time is handled separately by evalexpr context
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
    use evalexpr::{ContextWithMutableFunctions, DefaultNumericTypes, HashMapContext};

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

    let mut context: HashMapContext<DefaultNumericTypes> = HashMapContext::new();

    // Note: @player.* variables have been removed. Use $player_hp, $player_hp_max, etc.
    // 注意：@player.* 变量已移除。请使用 $player_hp, $player_hp_max 等。

    // Register helper functions that work with FRE data
    // 注册与 FRE 数据配合使用的辅助函数
    let _ = context.set_function(
        "inventory_is_empty".to_string(),
        evalexpr::Function::new({
            let is_empty = player_data
                .get_fact_string_list("player_inventory")
                .map(|list| list.is_empty())
                .unwrap_or(true);
            move |_arg| Ok(evalexpr::Value::Boolean(is_empty))
        }),
    );

    match evalexpr::eval_boolean_with_context(&processed_expr, &context) {
        Ok(result) => result,
        Err(e) => {
            // Try evaluating as a number and convert to bool (non-zero = true)
            match evalexpr::eval_with_context(&processed_expr, &context) {
                Ok(val) => {
                    if let Ok(f) = val.as_float() {
                        f != 0.0
                    } else if let Ok(i) = val.as_int() {
                        i != 0
                    } else if let Ok(b) = val.as_boolean() {
                        b
                    } else {
                        warn!(
                            "Failed to evaluate visible_when expression '{}' (processed: '{}'): {}",
                            expr, processed_expr, e
                        );
                        true // Default to visible on error
                    }
                }
                Err(e2) => {
                    warn!(
                        "Failed to evaluate visible_when expression '{}' (processed: '{}'): {} / {}",
                        expr, processed_expr, e, e2
                    );
                    true // Default to visible on error
                }
            }
        }
    }
}

pub fn evaluate_float_expr(
    expr: &FloatOrExpr,
    player_data: &PlayerDataView,
    time: Option<f64>,
) -> f32 {
    match expr {
        Val::Static(v) => *v,
        Val::Expr(expr_str) => {
            use evalexpr::{
                ContextWithMutableFunctions, ContextWithMutableVariables, DefaultNumericTypes,
                HashMapContext,
            };

            // Preprocess fact expressions before evaluation
            // 在求值前预处理 fact 表达式
            let processed_expr = preprocess_fact_expressions(expr_str, player_data);

            let mut context: HashMapContext<DefaultNumericTypes> = HashMapContext::new();

            // Register sin function
            let _ = context.set_function(
                "sin".to_string(),
                evalexpr::Function::new(|arg| {
                    let val: f64 = arg.as_float()?;
                    Ok(evalexpr::Value::Float(val.sin()))
                }),
            );

            // Register cos function
            let _ = context.set_function(
                "cos".to_string(),
                evalexpr::Function::new(|arg| {
                    let val: f64 = arg.as_float()?;
                    Ok(evalexpr::Value::Float(val.cos()))
                }),
            );

            // Register snap function (snap(val, step))
            let _ = context.set_function(
                "snap".to_string(),
                evalexpr::Function::new(|arg| {
                    if let Ok(tuple) = arg.as_tuple() {
                        if tuple.len() != 2 {
                            return Err(evalexpr::EvalexprError::CustomMessage(
                                "snap expects 2 arguments".to_string(),
                            ));
                        }
                        let val: f64 = tuple[0].as_float()?;
                        let step: f64 = tuple[1].as_float()?;
                        if step == 0.0 {
                            return Ok(evalexpr::Value::Float(val));
                        }
                        // Use floor to snap to the previous step (like 30fps update)
                        let snapped: f64 = (val / step).floor() * step;
                        Ok(evalexpr::Value::Float(snapped))
                    } else {
                        // If single argument, maybe return as is or error?
                        // But snap needs step.
                        Err(evalexpr::EvalexprError::CustomMessage(
                            "snap expects 2 arguments (val, step)".to_string(),
                        ))
                    }
                }),
            );

            // Register random function (random() or random(min, max))
            let _ = context.set_function(
                "random".to_string(),
                evalexpr::Function::new(|arg| {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    // Use system time as seed for randomness
                    let nanos = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .subsec_nanos();
                    // Simple pseudo-random: -1.0 to 1.0
                    let rand_val = ((nanos % 1000) as f64 / 1000.0) * 2.0 - 1.0;

                    if let Ok(tuple) = arg.as_tuple() {
                        if tuple.len() == 2 {
                            // random(min, max) - return value in range [min, max]
                            let min: f64 = tuple[0].as_float()?;
                            let max: f64 = tuple[1].as_float()?;
                            let range = max - min;
                            let result = min + (rand_val + 1.0) * 0.5 * range;
                            Ok(evalexpr::Value::Float(result))
                        } else if tuple.len() == 1 {
                            // random(range) - return value in range [-range, range]
                            let range: f64 = tuple[0].as_float()?;
                            Ok(evalexpr::Value::Float(rand_val * range))
                        } else {
                            Err(evalexpr::EvalexprError::CustomMessage(
                                "random expects 0, 1, or 2 arguments".to_string(),
                            ))
                        }
                    } else {
                        // Single argument: random(range)
                        let range: f64 = arg.as_float()?;
                        Ok(evalexpr::Value::Float(rand_val * range))
                    }
                }),
            );

            // Note: @player.* variables have been removed. Use $player_hp, $player_hp_max, etc.
            // 注意：@player.* 变量已移除。请使用 $player_hp, $player_hp_max 等。

            if let Some(t) = time {
                // debug!("Setting @time to {}", t);
                let _ = context.set_value("@time".to_string(), evalexpr::Value::Float(t));
            } else {
                let _ = context.set_value("@time".to_string(), evalexpr::Value::Float(0.0));
            }

            match evalexpr::eval_with_context(&processed_expr, &context) {
                Ok(val) => {
                    if let Ok(f) = val.as_float() {
                        let f_f64: f64 = f;
                        f_f64 as f32
                    } else if let Ok(i) = val.as_int() {
                        let i_i64: i64 = i;
                        i_i64 as f32
                    } else {
                        warn!(
                            "Failed to convert expression result to number: {}",
                            expr_str
                        );
                        0.0
                    }
                }
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
        Val::Static(v) => *v,
        Val::Expr(expr_str) => {
            use evalexpr::{
                ContextWithMutableFunctions, ContextWithMutableVariables, DefaultNumericTypes,
                HashMapContext,
            };

            // Preprocess with repeat context support
            // 使用重复上下文支持进行预处理
            let processed_expr =
                preprocess_fact_expressions_with_repeat(expr_str, player_data, repeat_ctx);

            let mut context: HashMapContext<DefaultNumericTypes> = HashMapContext::new();

            // Register sin function
            let _ = context.set_function(
                "sin".to_string(),
                evalexpr::Function::new(|arg| {
                    let val: f64 = arg.as_float()?;
                    Ok(evalexpr::Value::Float(val.sin()))
                }),
            );

            // Register cos function
            let _ = context.set_function(
                "cos".to_string(),
                evalexpr::Function::new(|arg| {
                    let val: f64 = arg.as_float()?;
                    Ok(evalexpr::Value::Float(val.cos()))
                }),
            );

            // Register if function
            let _ = context.set_function(
                "if".to_string(),
                evalexpr::Function::new(|arg| {
                    if let Ok(tuple) = arg.as_tuple() {
                        if tuple.len() == 3 {
                            let condition: bool = tuple[0].as_boolean()?;
                            if condition {
                                Ok(tuple[1].clone())
                            } else {
                                Ok(tuple[2].clone())
                            }
                        } else {
                            Err(evalexpr::EvalexprError::CustomMessage(
                                "if expects 3 arguments".to_string(),
                            ))
                        }
                    } else {
                        Err(evalexpr::EvalexprError::CustomMessage(
                            "if expects a tuple of 3 arguments".to_string(),
                        ))
                    }
                }),
            );

            // Set @time variable
            if let Some(t) = time {
                let _ = context.set_value("@time".to_string(), evalexpr::Value::Float(t));
            } else {
                let _ = context.set_value("@time".to_string(), evalexpr::Value::Float(0.0));
            }

            match evalexpr::eval_with_context(&processed_expr, &context) {
                Ok(val) => {
                    if let Ok(f) = val.as_float() {
                        f as f32
                    } else if let Ok(i) = val.as_int() {
                        i as f32
                    } else {
                        warn!(
                            "Failed to convert expression result to number: {}",
                            expr_str
                        );
                        0.0
                    }
                }
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
    use evalexpr::{
        ContextWithMutableFunctions, ContextWithMutableVariables, DefaultNumericTypes,
        HashMapContext,
    };

    let mut context: HashMapContext<DefaultNumericTypes> = HashMapContext::new();

    // Register @current variable
    if let Some(current) = current_value {
        debug!(
            "[evaluate_float_expr_with_current] Setting @current = {}",
            current
        );
        let _ = context.set_value(
            "@current".to_string(),
            evalexpr::Value::Float(current as f64),
        );
    } else {
        debug!("[evaluate_float_expr_with_current] Setting @current = 0.0 (no current value)");
        let _ = context.set_value("@current".to_string(), evalexpr::Value::Float(0.0));
    }

    // Register sin function
    let _ = context.set_function(
        "sin".to_string(),
        evalexpr::Function::new(|arg| {
            let val: f64 = arg.as_float()?;
            Ok(evalexpr::Value::Float(val.sin()))
        }),
    );

    // Register cos function
    let _ = context.set_function(
        "cos".to_string(),
        evalexpr::Function::new(|arg| {
            let val: f64 = arg.as_float()?;
            Ok(evalexpr::Value::Float(val.cos()))
        }),
    );

    // Register snap function (snap(val, step))
    let _ = context.set_function(
        "snap".to_string(),
        evalexpr::Function::new(|arg| {
            if let Ok(tuple) = arg.as_tuple() {
                if tuple.len() != 2 {
                    return Err(evalexpr::EvalexprError::CustomMessage(
                        "snap expects 2 arguments".to_string(),
                    ));
                }
                let val: f64 = tuple[0].as_float()?;
                let step: f64 = tuple[1].as_float()?;
                if step == 0.0 {
                    return Ok(evalexpr::Value::Float(val));
                }
                // Use floor to snap to the previous step (like 30fps update)
                let snapped: f64 = (val / step).floor() * step;
                Ok(evalexpr::Value::Float(snapped))
            } else {
                Err(evalexpr::EvalexprError::CustomMessage(
                    "snap expects 2 arguments (val, step)".to_string(),
                ))
            }
        }),
    );

    // Register random function (random() or random(min, max))
    let _ = context.set_function(
        "random".to_string(),
        evalexpr::Function::new(|arg| {
            use std::time::{SystemTime, UNIX_EPOCH};
            // Use system time as seed for randomness
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos();
            let random = ((nanos as f64) / (u32::MAX as f64)) * 2.0 - 1.0;

            if arg.is_empty() {
                // random() returns value in [-1, 1]
                Ok(evalexpr::Value::Float(random))
            } else if let Ok(tuple) = arg.as_tuple() {
                if tuple.len() == 1 {
                    // random(max) returns value in [-max, max]
                    let max: f64 = tuple[0].as_float()?;
                    Ok(evalexpr::Value::Float(random * max))
                } else if tuple.len() == 2 {
                    // random(min, max) returns value in [min, max]
                    let min: f64 = tuple[0].as_float()?;
                    let max: f64 = tuple[1].as_float()?;
                    let normalized = (random + 1.0) / 2.0;
                    Ok(evalexpr::Value::Float(min + normalized * (max - min)))
                } else {
                    Err(evalexpr::EvalexprError::CustomMessage(
                        "random expects 0, 1, or 2 arguments".to_string(),
                    ))
                }
            } else {
                // Single value, treat as max
                let max: f64 = arg.as_float()?;
                Ok(evalexpr::Value::Float(random * max))
            }
        }),
    );

    // Note: @player.* variables have been removed. Use $player_hp, $player_hp_max, etc.
    // 注意：@player.* 变量已移除。请使用 $player_hp, $player_hp_max 等。

    if let Some(t) = time {
        let _ = context.set_value("@time".to_string(), evalexpr::Value::Float(t));
    } else {
        let _ = context.set_value("@time".to_string(), evalexpr::Value::Float(0.0));
    }

    debug!(
        "[evaluate_float_expr_with_current] Evaluating expression: '{}'",
        expr_str
    );
    match evalexpr::eval_with_context(expr_str, &context) {
        Ok(val) => {
            if let Ok(f) = val.as_float() {
                let f_f64: f64 = f;
                debug!(
                    "[evaluate_float_expr_with_current] Result: {} (float)",
                    f_f64
                );
                f_f64 as f32
            } else if let Ok(i) = val.as_int() {
                let i_i64: i64 = i;
                debug!("[evaluate_float_expr_with_current] Result: {} (int)", i_i64);
                i_i64 as f32
            } else {
                warn!(
                    "Failed to convert expression result to number: {}",
                    expr_str
                );
                0.0
            }
        }
        Err(e) => {
            warn!("Failed to evaluate expression '{}': {}", expr_str, e);
            0.0
        }
    }
}

pub fn resolve_text_content(
    template: &str,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &PlayerDataView,
    item_registry: &crate::core::item::ItemRegistry,
) -> String {
    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{'
            && let Some(&next_ch) = chars.peek()
        {
            if next_ch == '{' {
                chars.next();
                let mut key = String::new();
                let mut found_closing = false;

                while let Some(ch) = chars.next() {
                    if ch == '}'
                        && let Some(&next_ch) = chars.peek()
                        && next_ch == '}'
                    {
                        chars.next();
                        found_closing = true;
                        break;
                    }
                    key.push(ch);
                }

                if found_closing {
                    let resolved = mortar_strings.resolve(&key);
                    result.push_str(resolved);
                } else {
                    result.push_str("{{");
                    result.push_str(&key);
                }
                continue;
            } else if next_ch == '|' {
                // Lambda syntax: {|item, i| in $array => "template" sep "separator"}
                // Lambda 语法：{|item, i| in $array => "template" sep "separator"}
                chars.next(); // consume '|'
                let mut expr = String::from("|");
                let mut brace_depth = 1;
                let mut found_closing = false;

                // Parse until matching closing brace, handling nested quotes
                let mut in_quotes = false;
                for ch in chars.by_ref() {
                    if ch == '"' && !in_quotes {
                        in_quotes = true;
                        expr.push(ch);
                    } else if ch == '"' && in_quotes {
                        in_quotes = false;
                        expr.push(ch);
                    } else if ch == '{' && !in_quotes {
                        brace_depth += 1;
                        expr.push(ch);
                    } else if ch == '}' && !in_quotes {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            found_closing = true;
                            break;
                        }
                        expr.push(ch);
                    } else {
                        expr.push(ch);
                    }
                }

                if found_closing {
                    if let Some(evaluated) = evaluate_lambda_expression(&expr, player_data) {
                        result.push_str(&evaluated);
                    } else {
                        warn!("Failed to evaluate lambda expression: {}", expr);
                        result.push_str(&format!("{{{}}})", expr));
                    }
                } else {
                    result.push_str("{|");
                    result.push_str(&expr[1..]); // skip the '|' we already added
                }
                continue;
            } else if next_ch == '$' {
                // New syntax: {$var} or {$array[index]} - direct FRE fact access
                // 新语法：{$var} 或 {$array[index]} - 直接访问 FRE 事实
                chars.next();
                let mut key = String::new();
                let mut found_closing = false;

                for ch in chars.by_ref() {
                    if ch == '}' {
                        found_closing = true;
                        break;
                    }
                    key.push(ch);
                }

                if found_closing {
                    // Check for array index syntax: array[index]
                    // 检查数组索引语法：array[index]
                    let value = if let Some(bracket_pos) = key.find('[') {
                        let array_name = &key[..bracket_pos];
                        let index_part = &key[bracket_pos + 1..];
                        if let Some(close_bracket) = index_part.find(']') {
                            let index_str = &index_part[..close_bracket];
                            // Parse index as integer
                            if let Ok(index) = index_str.parse::<usize>() {
                                // Try StringList first, then IntList
                                if let Some(list) = player_data.get_fact_string_list(array_name) {
                                    list.get(index).cloned().unwrap_or_else(|| {
                                        warn!(
                                            "Index {} out of bounds for StringList '{}'",
                                            index, array_name
                                        );
                                        format!("<{}[{}]>", array_name, index)
                                    })
                                } else if let Some(list) = player_data.get_fact_int_list(array_name)
                                {
                                    list.get(index).map(|i| i.to_string()).unwrap_or_else(|| {
                                        warn!(
                                            "Index {} out of bounds for IntList '{}'",
                                            index, array_name
                                        );
                                        format!("<{}[{}]>", array_name, index)
                                    })
                                } else {
                                    warn!("Array fact '{}' not found for index access", array_name);
                                    format!("<{}[{}]>", array_name, index)
                                }
                            } else {
                                warn!("Invalid index '{}' in array access '{}'", index_str, key);
                                format!("<{}>", key)
                            }
                        } else {
                            // Malformed bracket syntax
                            warn!("Malformed array access syntax: {}", key);
                            format!("<{}>", key)
                        }
                    } else {
                        // Regular fact access (no array index)
                        // 普通事实访问（无数组索引）
                        if let Some(fact) = player_data.get_fact(&key) {
                            match fact {
                                bevy_fact_rule_event::FactValue::Int(i) => i.to_string(),
                                bevy_fact_rule_event::FactValue::Float(f) => f.to_string(),
                                bevy_fact_rule_event::FactValue::String(s) => s.clone(),
                                bevy_fact_rule_event::FactValue::Bool(b) => b.to_string(),
                                bevy_fact_rule_event::FactValue::StringList(list) => {
                                    list.join(", ")
                                }
                                bevy_fact_rule_event::FactValue::IntList(list) => list
                                    .iter()
                                    .map(|i| i.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", "),
                            }
                        } else {
                            warn!("Fact '{}' not found for template substitution", key);
                            format!("<{}>", key)
                        }
                    };
                    result.push_str(&value);
                } else {
                    result.push_str("{$");
                    result.push_str(&key);
                }
                continue;
            } else if next_ch == '@' {
                // Legacy syntax: {@path} - uses resolve_data_path
                // 旧语法：{@path} - 使用 resolve_data_path
                chars.next();
                let mut path = String::new();
                let mut found_closing = false;

                for ch in chars.by_ref() {
                    if ch == '}' {
                        found_closing = true;
                        break;
                    }
                    path.push(ch);
                }

                if found_closing {
                    let value =
                        resolve_data_path(&path, player_data, item_registry, mortar_strings);
                    result.push_str(&value);
                } else {
                    result.push_str("{@");
                    result.push_str(&path);
                }
                continue;
            }
        }
        result.push(ch);
    }

    result
}

pub fn evaluate_condition(condition: &str, player_data: &PlayerDataView) -> bool {
    // Helper closure to get inventory
    let get_inventory = || {
        player_data
            .get_fact_string_list("player_inventory")
            .unwrap_or_default()
    };

    match condition {
        "player.inventory.is_empty" => get_inventory().is_empty(),
        "player.inventory.is_not_empty" => !get_inventory().is_empty(),
        _ => {
            if condition.starts_with("player.") {
                let parts: Vec<&str> = condition.split('.').collect();
                if parts.len() >= 3 {
                    match (parts[1], parts[2]) {
                        ("hp", "is_low") => {
                            let hp = player_data.get_fact_int("player_hp").unwrap_or(0);
                            let hp_max = player_data.get_fact_int("player_hp_max").unwrap_or(1);
                            hp < hp_max / 4
                        }
                        ("hp", "is_critical") => {
                            let hp = player_data.get_fact_int("player_hp").unwrap_or(0);
                            hp <= 1
                        }
                        ("gold", "is_zero") => {
                            player_data.get_fact_int("player_gold").unwrap_or(0) == 0
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
}

pub fn resolve_data_path(
    path: &str,
    player_data: &PlayerDataView,
    item_registry: &crate::core::item::ItemRegistry,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
) -> String {
    use crate::core::item::ItemType;

    // Helper closures for common patterns
    let get_string = |key: &str| player_data.get_fact_string(key).unwrap_or_default();
    let get_int = |key: &str| player_data.get_fact_int(key).unwrap_or(0);

    match path {
        "player.name" => get_string("player_name"),
        "player.lv" => get_int("player_lv").to_string(),
        "player.hp" => {
            let result = get_int("player_hp").to_string();
            info!("[resolve_data_path] player.hp = {}", result);
            result
        }
        "player.hp_max" => {
            let result = get_int("player_hp_max").to_string();
            info!("[resolve_data_path] player.hp_max = {}", result);
            result
        }
        "player.gold" => get_int("player_gold").to_string(),
        "player.exp" => get_int("player_exp").to_string(),
        "player.next_exp" => get_int("player_next_exp").to_string(),
        "player.attack" => get_int("player_attack").to_string(),
        "player.defense" => get_int("player_defense").to_string(),
        "player.inventory" => {
            let inventory = player_data
                .get_fact_string_list("player_inventory")
                .unwrap_or_default();
            inventory
                .iter()
                .take(8)
                .map(|item_id| {
                    if let Some(item) = item_registry.get(item_id) {
                        let key = format!("{}:{}", item.locate_file, item.locate_name);
                        mortar_strings.resolve(&key).to_string()
                    } else {
                        warn!("Item ID '{}' not found in registry!", item_id);
                        format!("UNDEFINED ({})", item_id)
                    }
                })
                .collect::<Vec<String>>()
                .join("\n")
        }
        "player.weapon" => {
            let weapon = get_string("player_weapon");
            if let Some(item) = item_registry.get(&weapon) {
                let key = format!("{}:{}", item.locate_file, item.locate_name);
                mortar_strings.resolve(&key).to_string()
            } else {
                weapon
            }
        }
        "player.weapon_atk" => {
            let weapon = get_string("player_weapon");
            if let Some(item) = item_registry.get(&weapon)
                && let ItemType::Weapon { damage, .. } = item.item_type
            {
                return damage.to_string();
            }
            "0".to_string()
        }
        "player.total_attack" => {
            let weapon = get_string("player_weapon");
            let weapon_atk = if let Some(item) = item_registry.get(&weapon) {
                if let ItemType::Weapon { damage, .. } = item.item_type {
                    damage as i64
                } else {
                    0
                }
            } else {
                0
            };
            (get_int("player_attack") + weapon_atk).to_string()
        }
        "player.armor" => {
            let armor = get_string("player_armor");
            if let Some(item) = item_registry.get(&armor) {
                let key = format!("{}:{}", item.locate_file, item.locate_name);
                mortar_strings.resolve(&key).to_string()
            } else {
                armor
            }
        }
        "player.armor_def" => {
            let armor = get_string("player_armor");
            if let Some(item) = item_registry.get(&armor)
                && let ItemType::Armor { defense } = item.item_type
            {
                return defense.to_string();
            }
            "0".to_string()
        }
        "player.total_defense" => {
            let armor = get_string("player_armor");
            let armor_def = if let Some(item) = item_registry.get(&armor) {
                if let ItemType::Armor { defense } = item.item_type {
                    defense as i64
                } else {
                    0
                }
            } else {
                0
            };
            (get_int("player_defense") + armor_def).to_string()
        }
        _ => format!("<unknown:{}>", path),
    }
}

/// Resolve a Val<f32> to an actual f32 value.
///
/// 将 Val<f32> 解析为实际的 f32 值。
pub fn resolve_val_f32(
    val: &crate::app_state::battle::chapter_schema::Val<f32>,
    current_value: Option<f32>,
    player_data: &PlayerDataView,
    time: Option<f64>,
) -> f32 {
    match val {
        crate::app_state::battle::chapter_schema::Val::Static(v) => *v,
        crate::app_state::battle::chapter_schema::Val::Expr(expr_str) => {
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

/// Resolve a Val<bool> to an actual bool value.
///
/// 将 Val<bool> 解析为实际的 bool 值。
pub fn resolve_val_bool(
    val: &crate::app_state::battle::chapter_schema::Val<bool>,
    player_data: &PlayerDataView,
) -> bool {
    match val {
        crate::app_state::battle::chapter_schema::Val::Static(v) => *v,
        crate::app_state::battle::chapter_schema::Val::Expr(expr_str) => {
            // Evaluate condition expression
            evaluate_condition(expr_str, player_data)
        }
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
            .get_fact_string_list("player_inventory")
            .unwrap_or_default()
    };

    // Handle "index == N" pattern with optional additional conditions
    if condition.starts_with("index == ")
        && let Some(rest) = condition.strip_prefix("index == ")
    {
        // Split by &&
        let parts: Vec<&str> = rest.split("&&").map(|s| s.trim()).collect();

        // First part should be the index number
        if let Some(index_part) = parts.first()
            && let Ok(target_index) = index_part.parse::<usize>()
        {
            if index != target_index {
                return false;
            }

            // Check additional conditions
            for part in parts.iter().skip(1) {
                if *part == "!player.inventory.is_empty" {
                    if get_inventory().is_empty() {
                        return false;
                    }
                } else if *part == "player.inventory.is_empty" && !get_inventory().is_empty() {
                    return false;
                }
                // Add more conditions as needed
            }

            return true;
        }
    }

    // For other conditions, delegate to the existing evaluate_condition
    evaluate_condition(condition, player_data)
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
