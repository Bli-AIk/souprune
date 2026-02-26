use super::super::layout::FloatOrExpr;
use super::super::layout::view_schema::MaterialParamValue;
use crate::app_state::battle::chapter_schema::Val;
use crate::app_state::overworld::OverworldSubState;
use crate::core::view::expr_eval::create_eval_callback;
use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap};

// Re-export PlayerDataView for backward compatibility
pub use super::player_data::PlayerDataView;

// Re-export evaluation functions for backward compatibility
#[allow(unused_imports)]
pub use super::evaluation::{
    evaluate_condition, evaluate_dynamic_color, evaluate_fact_expression, evaluate_float_expr,
    evaluate_float_expr_with_current, evaluate_float_expr_with_repeat,
    evaluate_transition_condition_unified, evaluate_visible_when,
    preprocess_fact_expressions_with_repeat, resolve_val_bool, resolve_val_f32,
};

// Import for internal use
use super::evaluation::preprocess_fact_expressions;

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

    // Preprocess material parameter expressions
    if let Some(ref mut material) = result.material {
        let mut new_params = HashMap::new();
        for (name, value) in &material.params {
            let new_value = preprocess_material_param_for_repeat(value, repeat_ctx);
            new_params.insert(name.clone(), new_value);
        }
        material.params = new_params;
    }

    result
}

/// Preprocess a MaterialParamValue to resolve repeat context variables.
///
/// 预处理 MaterialParamValue 以解析 repeat 上下文变量。
fn preprocess_material_param_for_repeat(
    value: &MaterialParamValue,
    repeat_ctx: &RepeatContext,
) -> MaterialParamValue {
    match value {
        MaterialParamValue::Static(v) => MaterialParamValue::Static(*v),
        MaterialParamValue::Expr(expr_str) => {
            // Check if expression contains repeat variables
            if !expr_str.contains('@') && !expr_str.contains("[@") {
                return MaterialParamValue::Expr(expr_str.clone());
            }

            // Reuse the same preprocessing logic as Val<f32>
            let val = Val::Expr(expr_str.clone());
            let processed = preprocess_val_for_repeat(&val, repeat_ctx);
            match processed {
                Val::Static(v) => {
                    trace!(
                        "[MaterialParam Preprocess] '{}' -> Static({}), repeat index: {}",
                        expr_str, v, repeat_ctx.index
                    );
                    MaterialParamValue::Static(v)
                }
                Val::Expr(s) => {
                    trace!(
                        "[MaterialParam Preprocess] '{}' -> Expr('{}'), repeat index: {}",
                        expr_str, s, repeat_ctx.index
                    );
                    MaterialParamValue::Expr(s)
                }
            }
        }
    }
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
            // Note: Supports namespace format like $player:inventory[@i]
            let dynamic_index_regex =
                regex::Regex::new(r"\$([a-zA-Z_][a-zA-Z0-9_:]*)\[@([a-zA-Z_][a-zA-Z0-9_]*)\]")
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
                        // @time is handled by fasteval context, keep as-is
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
/// Or with range: {|item, index| in $array[$offset..$offset+$limit] => "template" sep "separator"}
///
/// 解析并计算文本内容中的 lambda 表达式。
/// 格式：{|item, index| in $array => "template" sep "separator"}
/// 或带范围：{|item, index| in $array[$offset..$offset+$limit] => "template" sep "separator"}
///
/// Examples:
/// - `{|name| in $enemy_names => "* {name}"}` - simple item iteration
/// - `{|name, i| in $enemy_names => "* {name}" sep "\n"}` - with index and separator
/// - `{|name, i| in $enemy_names[$enemy_view_offset..$enemy_view_offset+$enemy_display_limit] => "* {name}"}` - with range
fn evaluate_lambda_expression(expr: &str, player_data: &PlayerDataView) -> Option<String> {
    // Extended regex with optional range and step syntax
    // Format: |item, index| in $array or $array[$start..$end] or $array[$start..$end step $step] => "template" sep "separator"
    // 扩展正则表达式，支持可选范围和步进语法
    // 格式：|item, index| in $array 或 $array[$start..$end] 或 $array[$start..$end step $step] => "template" sep "separator"
    let lambda_regex_with_range = regex::Regex::new(
        r#"^\|([a-zA-Z_][a-zA-Z0-9_]*)(?:,\s*([a-zA-Z_][a-zA-Z0-9_]*))?\|\s+in\s+\$([a-zA-Z_][a-zA-Z0-9_]*)(?:\[([^\]]+)\.\.([^\]\s]+)(?:\s+step\s+(\d+))?\])?\s*=>\s*"([^"]*)"\s*(?:sep\s*"([^"]*)")?$"#
    ).ok()?;

    let caps = lambda_regex_with_range.captures(expr)?;

    let item_var = &caps[1];
    let index_var = caps.get(2).map(|m| m.as_str());
    let array_name = &caps[3];
    let start_expr = caps.get(4).map(|m| m.as_str());
    let end_expr = caps.get(5).map(|m| m.as_str());
    let step_val: usize = caps
        .get(6)
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(1);
    let template = &caps[7];
    let separator = caps.get(8).map(|m| m.as_str()).unwrap_or("\n");

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

    // Calculate range (start and end indices)
    // 计算范围（起始和结束索引）
    let (start_idx, end_idx) = if let (Some(start), Some(end)) = (start_expr, end_expr) {
        // Evaluate start and end expressions
        let start_val = evaluate_lambda_range_expr(start, player_data).unwrap_or(0);
        let end_val = evaluate_lambda_range_expr(end, player_data).unwrap_or(array.len());
        (start_val.max(0), end_val.min(array.len()))
    } else {
        // No range specified - use entire array
        (0, array.len())
    };

    // Generate output for elements in range with step
    // 为范围内的元素生成输出（带步进）
    let lines: Vec<String> = array
        .iter()
        .enumerate()
        .filter(|(i, _)| *i >= start_idx && *i < end_idx && (*i - start_idx) % step_val == 0)
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

/// Evaluate a range expression for lambda (supports $fact_name and simple arithmetic)
/// 计算 lambda 的范围表达式（支持 $fact_name 和简单算术）
fn evaluate_lambda_range_expr(expr: &str, player_data: &PlayerDataView) -> Option<usize> {
    // Preprocess fact expressions
    let processed_expr = preprocess_fact_expressions(expr, player_data);

    let vars: BTreeMap<String, f64> = BTreeMap::new();
    let mut cb = create_eval_callback(&vars);

    match fasteval::ez_eval(&processed_expr, &mut cb) {
        Ok(val) => Some((val as i64).max(0) as usize),
        Err(e) => {
            warn!(
                "[evaluate_lambda_range_expr] Failed to evaluate '{}' (processed: '{}'): {}",
                expr, processed_expr, e
            );
            None
        }
    }
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
                    // Preprocess $variable in the key before resolving
                    // 在解析前预处理键中的 $variable
                    let processed_key = preprocess_fact_expressions(&key, player_data);
                    // Remove quotes from string values (format_fact_for_expr adds them for expressions)
                    // 移除字符串值的引号（format_fact_for_expr 为表达式添加的）
                    let processed_key = processed_key.replace('"', "");

                    // First try to resolve from facts (for dynamic values like dialogue_text)
                    // 首先尝试从 facts 解析（用于动态值如 dialogue_text）
                    let resolved = if let Some(fact_value) = player_data.get_fact(&processed_key) {
                        // Convert fact value to string
                        match fact_value {
                            bevy_fact_rule_event::FactValue::String(s) => s.to_string(),
                            bevy_fact_rule_event::FactValue::Int(i) => i.to_string(),
                            bevy_fact_rule_event::FactValue::Float(f) => f.to_string(),
                            bevy_fact_rule_event::FactValue::Bool(b) => b.to_string(),
                            _ => mortar_strings.resolve(&processed_key).to_string(),
                        }
                    } else {
                        // Fallback to mortar string table for localization
                        // 回退到 mortar 字符串表用于本地化
                        mortar_strings.resolve(&processed_key).to_string()
                    };

                    // Recursively resolve if the result still contains {{...}} markers
                    // 如果结果仍包含 {{...}} 标记，递归解析
                    let final_resolved = if resolved.contains("{{") && resolved.contains("}}") {
                        resolve_text_content(&resolved, mortar_strings, player_data, item_registry)
                    } else {
                        resolved
                    };
                    result.push_str(&final_resolved);
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
                        // Recursively resolve any localization markers in lambda output
                        // 递归处理 lambda 输出中的本地化标记
                        let resolved = resolve_text_content(
                            &evaluated,
                            mortar_strings,
                            player_data,
                            item_registry,
                        );
                        result.push_str(&resolved);
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
                                bevy_fact_rule_event::FactValue::FloatList(list) => list
                                    .iter()
                                    .map(|f| f.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", "),
                                bevy_fact_rule_event::FactValue::BoolList(list) => list
                                    .iter()
                                    .map(|b| b.to_string())
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
        "player.name" => get_string("player:name"),
        "player.lv" => get_int("player:lv").to_string(),
        "player.hp" => {
            let result = get_int("player:hp").to_string();
            info!("[resolve_data_path] player.hp = {}", result);
            result
        }
        "player.hp_max" => {
            let result = get_int("player:hp_max").to_string();
            info!("[resolve_data_path] player.hp_max = {}", result);
            result
        }
        "player.gold" => get_int("player:gold").to_string(),
        "player.exp" => get_int("player:exp").to_string(),
        "player.next_exp" => get_int("player:next_exp").to_string(),
        "player.attack" => get_int("player:attack").to_string(),
        "player.defense" => get_int("player:defense").to_string(),
        "player.inventory" => {
            let inventory = player_data
                .get_fact_string_list("player:inventory")
                .unwrap_or_default();
            inventory
                .iter()
                .take(8)
                .map(|item_id| {
                    if let Some(item) = item_registry.get(item_id) {
                        let key = format!("{}:{}", item.locale_file, item.locale_name);
                        mortar_strings.resolve(&key).to_string()
                    } else {
                        // Use trace level to avoid log spam for undefined items
                        trace!("Item ID '{}' not found in registry", item_id);
                        format!("UNDEFINED ({})", item_id)
                    }
                })
                .collect::<Vec<String>>()
                .join("\n")
        }
        "player.weapon" => {
            let weapon = get_string("player:weapon");
            if let Some(item) = item_registry.get(&weapon) {
                let key = format!("{}:{}", item.locale_file, item.locale_name);
                mortar_strings.resolve(&key).to_string()
            } else {
                weapon
            }
        }
        "player.weapon_atk" => {
            let weapon = get_string("player:weapon");
            if let Some(item) = item_registry.get(&weapon)
                && let ItemType::Weapon { damage, .. } = item.item_type
            {
                return damage.to_string();
            }
            "0".to_string()
        }
        "player.total_attack" => {
            let weapon = get_string("player:weapon");
            let weapon_atk = if let Some(item) = item_registry.get(&weapon) {
                if let ItemType::Weapon { damage, .. } = item.item_type {
                    damage as i64
                } else {
                    0
                }
            } else {
                0
            };
            (get_int("player:attack") + weapon_atk).to_string()
        }
        "player.armor" => {
            let armor = get_string("player:armor");
            if let Some(item) = item_registry.get(&armor) {
                let key = format!("{}:{}", item.locale_file, item.locale_name);
                mortar_strings.resolve(&key).to_string()
            } else {
                armor
            }
        }
        "player.armor_def" => {
            let armor = get_string("player:armor");
            if let Some(item) = item_registry.get(&armor)
                && let ItemType::Armor { defense } = item.item_type
            {
                return defense.to_string();
            }
            "0".to_string()
        }
        "player.total_defense" => {
            let armor = get_string("player:armor");
            let armor_def = if let Some(item) = item_registry.get(&armor) {
                if let ItemType::Armor { defense } = item.item_type {
                    defense as i64
                } else {
                    0
                }
            } else {
                0
            };
            (get_int("player:defense") + armor_def).to_string()
        }
        _ => format!("<unknown:{}>", path),
    }
}

