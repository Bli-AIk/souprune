//! # text_templates.rs
//!
//! # text_templates.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Implements the text-template language used by RON Views. It resolves double-brace
//! substitutions, lambda-style list rendering, array access, and nested fact lookups so authored
//! layout text can become concrete runtime strings.
//!
//! 实现了 RON View 使用的文本模板语言。它负责解析双花括号替换、lambda 风格的
//! 列表渲染、数组索引和嵌套事实访问，让作者写下的布局文本在运行时变成最终字符串。

use super::super::evaluation::preprocess_fact_expressions;
use super::PlayerDataView;
use crate::core::view::expr_eval::create_eval_callback;
use bevy::prelude::{debug, warn};
use std::collections::BTreeMap;
use std::sync::LazyLock;

static LAMBDA_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"^\|([a-zA-Z_][a-zA-Z0-9_]*)(?:,\s*([a-zA-Z_][a-zA-Z0-9_]*))?\|\s+in\s+\$([a-zA-Z_][a-zA-Z0-9_]*)(?:\[([^\]]+)\.\.([^\]\s]+)(?:\s+step\s+(\d+))?\])?\s*=>\s*"([^"]*)"\s*(?:sep\s*"([^"]*)")?$"#
    ).unwrap()
});

fn evaluate_lambda_expression(expr: &str, player_data: &PlayerDataView) -> Option<String> {
    let caps = LAMBDA_RE.captures(expr)?;

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

    let (start_idx, end_idx) = if let (Some(start), Some(end)) = (start_expr, end_expr) {
        let start_val = evaluate_lambda_range_expr(start, player_data).unwrap_or(0);
        let end_val = evaluate_lambda_range_expr(end, player_data).unwrap_or(array.len());
        (start_val, end_val.min(array.len()))
    } else {
        (0, array.len())
    };

    let lines: Vec<String> = array
        .iter()
        .enumerate()
        .filter(|(i, _)| *i >= start_idx && *i < end_idx && (*i - start_idx) % step_val == 0)
        .map(|(i, item)| {
            let mut line = template.to_string();
            line = line.replace(&format!("{{{}}}", item_var), item);
            if let Some(idx_var) = index_var {
                line = line.replace(&format!("{{{}}}", idx_var), &i.to_string());
            }
            line.replace("\\n", "\n")
        })
        .collect();

    Some(lines.join(separator))
}

fn evaluate_lambda_range_expr(expr: &str, player_data: &PlayerDataView) -> Option<usize> {
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

fn resolve_double_brace_template(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &PlayerDataView,
) -> String {
    let mut key = String::new();
    let mut found_closing = false;

    while let Some(ch) = chars.next() {
        if ch == '}' && chars.peek() == Some(&'}') {
            chars.next();
            found_closing = true;
            break;
        }
        key.push(ch);
    }

    if !found_closing {
        return format!("{{{{{}", key);
    }

    if let Some(path) = key.strip_prefix("data:") {
        return resolve_data_path(path, player_data, mortar_strings);
    }

    let processed_key = preprocess_fact_expressions(&key, player_data).replace('"', "");
    let resolved = if let Some(fact_value) = player_data.get_fact(&processed_key) {
        match fact_value {
            bevy_fact_rule_event::FactValue::String(s) => s.to_string(),
            bevy_fact_rule_event::FactValue::Int(i) => i.to_string(),
            bevy_fact_rule_event::FactValue::Float(f) => f.to_string(),
            bevy_fact_rule_event::FactValue::Bool(b) => b.to_string(),
            _ => mortar_strings.resolve(&processed_key).to_string(),
        }
    } else {
        mortar_strings.resolve(&processed_key).to_string()
    };

    if resolved.contains("{{") && resolved.contains("}}") {
        resolve_text_content(&resolved, mortar_strings, player_data)
    } else {
        resolved
    }
}

fn resolve_lambda_template(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &PlayerDataView,
) -> String {
    let mut expr = String::from("|");
    let mut brace_depth = 1;
    let mut found_closing = false;
    let mut in_quotes = false;

    for ch in chars.by_ref() {
        if ch == '"' {
            in_quotes = !in_quotes;
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

    if !found_closing {
        return format!("{{|{}", &expr[1..]);
    }

    let Some(evaluated) = evaluate_lambda_expression(&expr, player_data) else {
        warn!("Failed to evaluate lambda expression: {}", expr);
        return format!("{{{}}})", expr);
    };

    resolve_text_content(&evaluated, mortar_strings, player_data)
}

fn resolve_regular_fact(key: &str, player_data: &PlayerDataView) -> String {
    let Some(fact) = player_data.get_fact(key) else {
        warn!("Fact '{}' not found for template substitution", key);
        return format!("<{}>", key);
    };
    match fact {
        bevy_fact_rule_event::FactValue::Int(i) => i.to_string(),
        bevy_fact_rule_event::FactValue::Float(f) => f.to_string(),
        bevy_fact_rule_event::FactValue::String(s) => s.clone(),
        bevy_fact_rule_event::FactValue::Bool(b) => b.to_string(),
        bevy_fact_rule_event::FactValue::StringList(list) => list.join(", "),
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
}

fn resolve_array_fact_access(
    key: &str,
    bracket_pos: usize,
    player_data: &PlayerDataView,
) -> String {
    let array_name = &key[..bracket_pos];
    let index_part = &key[bracket_pos + 1..];

    let Some(close_bracket) = index_part.find(']') else {
        warn!("Malformed array access syntax: {}", key);
        return format!("<{}>", key);
    };

    let index_str = &index_part[..close_bracket];
    let Ok(index) = index_str.parse::<usize>() else {
        warn!("Invalid index '{}' in array access '{}'", index_str, key);
        return format!("<{}>", key);
    };

    if let Some(list) = player_data.get_fact_string_list(array_name) {
        list.get(index).cloned().unwrap_or_else(|| {
            warn!(
                "Index {} out of bounds for StringList '{}'",
                index, array_name
            );
            format!("<{}[{}]>", array_name, index)
        })
    } else if let Some(list) = player_data.get_fact_int_list(array_name) {
        list.get(index).map(|i| i.to_string()).unwrap_or_else(|| {
            warn!("Index {} out of bounds for IntList '{}'", index, array_name);
            format!("<{}[{}]>", array_name, index)
        })
    } else {
        warn!("Array fact '{}' not found for index access", array_name);
        format!("<{}[{}]>", array_name, index)
    }
}

fn resolve_fact_access_template(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    player_data: &PlayerDataView,
) -> String {
    let mut key = String::new();
    let mut found_closing = false;

    for ch in chars.by_ref() {
        if ch == '}' {
            found_closing = true;
            break;
        }
        key.push(ch);
    }

    if !found_closing {
        return format!("{{${}", key);
    }

    if let Some(bracket_pos) = key.find('[') {
        resolve_array_fact_access(&key, bracket_pos, player_data)
    } else {
        resolve_regular_fact(&key, player_data)
    }
}

pub fn resolve_text_content(
    template: &str,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &PlayerDataView,
) -> String {
    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '{' {
            result.push(ch);
            continue;
        }
        let Some(&next_ch) = chars.peek() else {
            result.push(ch);
            continue;
        };

        if next_ch == '{' {
            chars.next();
            result.push_str(&resolve_double_brace_template(
                &mut chars,
                mortar_strings,
                player_data,
            ));
        } else if next_ch == '|' {
            chars.next();
            result.push_str(&resolve_lambda_template(
                &mut chars,
                mortar_strings,
                player_data,
            ));
        } else if next_ch == '$' {
            chars.next();
            result.push_str(&resolve_fact_access_template(&mut chars, player_data));
        } else {
            result.push(ch);
        }
    }

    result
}

/// Resolve `{{data:path}}` template expressions using FRE facts.
///
/// Generic resolution: converts `a.b.c` → fact key `a:b.c` (first dot becomes colon)
/// and reads from the fact database. Custom resolvers registered in
/// `DataPathResolvers` are checked first for computed values.
///
/// 解析 `{{data:path}}` 模板表达式。
/// 通用解析：将 `a.b.c` 转换为事实键 `a:b.c`，从事实数据库读取。
/// 注册在 `DataPathResolvers` 中的自定义解析器优先检查。
pub fn resolve_data_path(
    path: &str,
    player_data: &PlayerDataView,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
) -> String {
    // Check custom resolvers first
    if let Some(result) = player_data.resolve_data_path(path, mortar_strings) {
        return result;
    }

    // Generic: convert first dot to colon for fact key lookup
    let fact_key = if let Some(dot_pos) = path.find('.') {
        format!("{}:{}", &path[..dot_pos], &path[dot_pos + 1..])
    } else {
        path.to_string()
    };

    if let Some(value) = player_data.get_fact(&fact_key) {
        return match value {
            bevy_fact_rule_event::FactValue::String(s) => s.clone(),
            bevy_fact_rule_event::FactValue::Int(i) => i.to_string(),
            bevy_fact_rule_event::FactValue::Float(f) => f.to_string(),
            bevy_fact_rule_event::FactValue::Bool(b) => b.to_string(),
            bevy_fact_rule_event::FactValue::StringList(list) => list.join("\n"),
            bevy_fact_rule_event::FactValue::IntList(list) => list
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
            bevy_fact_rule_event::FactValue::FloatList(list) => list
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
            bevy_fact_rule_event::FactValue::BoolList(list) => list
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        };
    }

    debug!("[resolve_data_path] unknown path: {path} (fact key: {fact_key})");
    format!("<unknown:{path}>")
}
