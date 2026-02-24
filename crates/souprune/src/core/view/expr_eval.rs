//! # expr_eval.rs
//!
//! Expression evaluation utilities for the View system.
//! Uses fasteval for safe, fast algebraic expression evaluation.
//!
//! 表达式求值工具，用于 View 系统。
//! 使用 fasteval 进行安全、快速的代数表达式求值。

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::BTreeMap;

/// Regex pattern for converting variable names to fasteval-compatible format.
/// Matches `@varname` and `obj.field` patterns.
///
/// 用于将变量名转换为 fasteval 兼容格式的正则表达式。
/// 匹配 `@varname` 和 `obj.field` 模式。
static VAR_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"@([a-zA-Z_][a-zA-Z0-9_]*)|([a-zA-Z_][a-zA-Z0-9_]*)\.([a-zA-Z_][a-zA-Z0-9_]*)")
        .unwrap()
});

/// Preprocess expression to convert special variable names to fasteval-compatible format.
/// - `@time` → `__at_time`
/// - `player.x` → `player_x`
/// - `player:hp` → `player__hp` (namespace format)
///
/// 预处理表达式，将特殊变量名转换为 fasteval 兼容格式。
pub fn preprocess_varname(expr: &str) -> String {
    // First, replace colons in variable names with double underscores
    // This handles the namespace format like `player:hp`
    let expr = expr.replace(':', "__");

    VAR_PATTERN
        .replace_all(&expr, |caps: &regex::Captures| {
            if let Some(at_var) = caps.get(1) {
                // @varname → __at_varname
                format!("__at_{}", at_var.as_str())
            } else if let (Some(obj), Some(field)) = (caps.get(2), caps.get(3)) {
                // obj.field → obj_field
                format!("{}_{}", obj.as_str(), field.as_str())
            } else {
                caps[0].to_string()
            }
        })
        .to_string()
}

/// Create a callback function that handles custom functions and variable lookup.
///
/// 创建处理自定义函数和变量查找的回调函数。
pub fn create_eval_callback<'a>(
    vars: &'a BTreeMap<String, f64>,
) -> impl FnMut(&str, Vec<f64>) -> Option<f64> + 'a {
    move |name: &str, args: Vec<f64>| -> Option<f64> {
        match name {
            // if(condition, then, else)
            "if" => {
                if args.len() == 3 {
                    Some(if args[0] != 0.0 { args[1] } else { args[2] })
                } else {
                    None
                }
            }
            // snap(val, step) - snap to step
            "snap" => {
                if args.len() == 2 {
                    let val = args[0];
                    let step = args[1];
                    if step == 0.0 {
                        Some(val)
                    } else {
                        Some((val / step).floor() * step)
                    }
                } else {
                    None
                }
            }
            // random() or random(max) or random(min, max)
            "random" => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let nanos = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .subsec_nanos();
                let rand_val = ((nanos as f64) / (u32::MAX as f64)) * 2.0 - 1.0;

                match args.len() {
                    0 => Some(rand_val),
                    1 => Some(rand_val * args[0]),
                    2 => Some(args[0] + (rand_val + 1.0) * 0.5 * (args[1] - args[0])),
                    _ => None,
                }
            }
            // Variable lookup
            _ => vars.get(name).copied(),
        }
    }
}

/// Evaluate a numeric expression with the given variables.
///
/// 使用给定变量求值数值表达式。
pub fn eval_number(expr: &str, vars: &BTreeMap<String, f64>) -> Result<f64, fasteval::Error> {
    let processed = preprocess_varname(expr);
    let mut cb = create_eval_callback(vars);
    fasteval::ez_eval(&processed, &mut cb)
}

/// Evaluate a boolean expression with the given variables.
/// Returns true if the result is non-zero.
///
/// 使用给定变量求值布尔表达式。
/// 如果结果非零则返回 true。
pub fn eval_bool(expr: &str, vars: &BTreeMap<String, f64>) -> Result<bool, fasteval::Error> {
    eval_number(expr, vars).map(|v| v != 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_varname() {
        assert_eq!(preprocess_varname("@time"), "__at_time");
        assert_eq!(preprocess_varname("@current"), "__at_current");
        assert_eq!(preprocess_varname("player.x"), "player_x");
        assert_eq!(preprocess_varname("camera.y"), "camera_y");
        assert_eq!(
            preprocess_varname("@time + player.x * 2"),
            "__at_time + player_x * 2"
        );
    }

    #[test]
    fn test_eval_number_basic() {
        let vars = BTreeMap::new();
        assert_eq!(eval_number("1 + 2 * 3", &vars).unwrap(), 7.0);
        assert_eq!(eval_number("sin(0)", &vars).unwrap(), 0.0);
        assert!((eval_number("cos(0)", &vars).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_eval_number_with_vars() {
        let mut vars = BTreeMap::new();
        vars.insert("x".to_string(), 10.0);
        vars.insert("__at_time".to_string(), 5.0);
        assert_eq!(eval_number("x + 5", &vars).unwrap(), 15.0);
        assert_eq!(eval_number("@time * 2", &vars).unwrap(), 10.0);
    }

    #[test]
    fn test_eval_if_function() {
        let vars = BTreeMap::new();
        assert_eq!(eval_number("if(1, 10, 20)", &vars).unwrap(), 10.0);
        assert_eq!(eval_number("if(0, 10, 20)", &vars).unwrap(), 20.0);
    }

    #[test]
    fn test_eval_snap_function() {
        let vars = BTreeMap::new();
        assert_eq!(eval_number("snap(1.7, 0.5)", &vars).unwrap(), 1.5);
        assert_eq!(eval_number("snap(2.3, 1)", &vars).unwrap(), 2.0);
    }

    #[test]
    fn test_eval_bool() {
        let vars = BTreeMap::new();
        assert!(eval_bool("1 > 0", &vars).unwrap());
        assert!(!eval_bool("1 < 0", &vars).unwrap());
        assert!(eval_bool("1 == 1", &vars).unwrap());
    }
}
