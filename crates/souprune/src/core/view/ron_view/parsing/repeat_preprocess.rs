//! # repeat_preprocess.rs
//!
//! # repeat_preprocess.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Preprocesses repeat-aware expressions before a repeated sprite is spawned. It
//! resolves placeholders such as `@i` and array lookups that depend on the current repeat
//! iteration so the downstream spawn code can operate on a concrete definition.
//!
//! 会在 repeat 精灵真正生成前，先预处理其中依赖 repeat 上下文的表达式。像 `@i`
//! 以及依赖当前索引的数组访问都会在这里被展开，好让后续生成逻辑面对的是更具体的定义。

use super::super::super::layout::SpriteDef;
use super::super::super::layout::view_schema::MaterialParamValue;
use super::super::evaluation::{DYNAMIC_INDEX_RE, REPEAT_VAR_RE};
use super::RepeatContext;
use crate::core::sequencer::chapter_schema::Value;
use bevy::prelude::trace;
use std::collections::HashMap;

/// Preprocess a SpriteDef to resolve repeat context variables (@i, $array[@i]).
pub fn preprocess_sprite_def_for_repeat(
    sprite_def: &SpriteDef,
    repeat_ctx: &RepeatContext,
) -> SpriteDef {
    let mut result = sprite_def.clone();

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

fn preprocess_material_param_for_repeat(
    value: &MaterialParamValue,
    repeat_ctx: &RepeatContext,
) -> MaterialParamValue {
    match value {
        MaterialParamValue::Static(v) => MaterialParamValue::Static(*v),
        MaterialParamValue::Expr(expr_str) => {
            if !expr_str.contains('@') && !expr_str.contains("[@") {
                return MaterialParamValue::Expr(expr_str.clone());
            }

            let val = Value::Expr(expr_str.clone());
            let processed = preprocess_val_for_repeat(&val, repeat_ctx);
            match processed {
                Value::Static(v) => {
                    trace!(
                        "[MaterialParam Preprocess] '{}' -> Static({}), repeat index: {}",
                        expr_str, v, repeat_ctx.index
                    );
                    MaterialParamValue::Static(v)
                }
                Value::Expr(s) => {
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

fn preprocess_val_for_repeat(val: &Value<f32>, repeat_ctx: &RepeatContext) -> Value<f32> {
    match val {
        Value::Static(v) => Value::Static(*v),
        Value::Expr(expr_str) => {
            if !expr_str.contains('@') && !expr_str.contains("[@") {
                return Value::Expr(expr_str.clone());
            }

            let dynamic_index_regex = &*DYNAMIC_INDEX_RE;
            let mut result = dynamic_index_regex
                .replace_all(expr_str, |caps: &regex::Captures| {
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
                    format!("${}[{}]", array_name, index)
                })
                .to_string();

            let repeat_var_regex = &*REPEAT_VAR_RE;
            result = repeat_var_regex
                .replace_all(&result, |caps: &regex::Captures| {
                    let var_name = &caps[1];
                    if var_name == "i" || var_name == "index" {
                        repeat_ctx.index.to_string()
                    } else if var_name == "time" {
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

            Value::Expr(result)
        }
    }
}
