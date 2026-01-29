use super::super::components::IndexBound;
use super::super::layout::FloatOrExpr;
use crate::app_state::battle::chapter_schema::Val;
use crate::app_state::overworld::OverworldState;
use crate::core::input::Action;
use bevy::prelude::*;

pub fn parse_action(action_str: &str) -> Option<Action> {
    match action_str {
        "Up" => Some(Action::Up),
        "Down" => Some(Action::Down),
        "Left" => Some(Action::Left),
        "Right" => Some(Action::Right),
        "Confirm" => Some(Action::Confirm),
        "Cancel" => Some(Action::Cancel),
        "Menu" => Some(Action::Menu),
        _ => None,
    }
}

pub fn parse_overworld_state(state_str: &str) -> Option<OverworldState> {
    match state_str {
        "Normal" => Some(OverworldState::Normal),
        "Backpack" => Some(OverworldState::Backpack),
        "Cutscene" => Some(OverworldState::Cutscene),
        _ => None,
    }
}

pub fn evaluate_index_bound(
    bound: &IndexBound,
    player_data: &crate::core::data::PlayerData,
) -> usize {
    match bound {
        IndexBound::Static(value) => *value,
        IndexBound::Dynamic(expr) => evaluate_index_expression(expr, player_data),
    }
}

fn evaluate_index_expression(expr: &str, player_data: &crate::core::data::PlayerData) -> usize {
    let expr = expr.trim();

    if expr == "inventory.len()" {
        return player_data.inventory.len();
    }

    if expr == "inventory_capacity" {
        return player_data.inventory_capacity;
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

pub fn evaluate_float_expr(
    expr: &FloatOrExpr,
    player_data: &crate::core::data::PlayerData,
    time: Option<f64>,
) -> f32 {
    match expr {
        Val::Static(v) => *v,
        Val::Expr(expr_str) => {
            use evalexpr::{
                ContextWithMutableFunctions, ContextWithMutableVariables, DefaultNumericTypes,
                HashMapContext,
            };

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

            let _ = context.set_value(
                "@player.hp".to_string(),
                evalexpr::Value::Int(player_data.hp as i64),
            );
            let _ = context.set_value(
                "@player.hp_max".to_string(),
                evalexpr::Value::Int(player_data.hp_max as i64),
            );
            let _ = context.set_value(
                "@player.lv".to_string(),
                evalexpr::Value::Int(player_data.lv as i64),
            );

            if let Some(t) = time {
                // debug!("Setting @time to {}", t);
                let _ = context.set_value("@time".to_string(), evalexpr::Value::Float(t));
            } else {
                let _ = context.set_value("@time".to_string(), evalexpr::Value::Float(0.0));
            }

            match evalexpr::eval_with_context(expr_str, &context) {
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

pub fn resolve_text_content(
    template: &str,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &crate::core::data::PlayerData,
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
            } else if next_ch == '@' {
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

pub fn evaluate_condition(condition: &str, player_data: &crate::core::data::PlayerData) -> bool {
    match condition {
        "player.inventory.is_empty" => player_data.inventory.is_empty(),
        "player.inventory.is_not_empty" => !player_data.inventory.is_empty(),
        _ => {
            if condition.starts_with("player.") {
                let parts: Vec<&str> = condition.split('.').collect();
                if parts.len() >= 3 {
                    match (parts[1], parts[2]) {
                        ("hp", "is_low") => player_data.hp < player_data.hp_max / 4,
                        ("hp", "is_critical") => player_data.hp <= 1,
                        ("gold", "is_zero") => player_data.gold == 0,
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
    player_data: &crate::core::data::PlayerData,
    item_registry: &crate::core::item::ItemRegistry,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
) -> String {
    use crate::core::item::ItemType;

    match path {
        "player.name" => player_data.name.clone(),
        "player.lv" => player_data.lv.to_string(),
        "player.hp" => {
            let result = player_data.hp.to_string();
            info!("[resolve_data_path] player.hp = {}", result);
            result
        }
        "player.hp_max" => {
            let result = player_data.hp_max.to_string();
            info!("[resolve_data_path] player.hp_max = {}", result);
            result
        }
        "player.gold" => player_data.gold.to_string(),
        "player.exp" => player_data.exp.to_string(),
        "player.next_exp" => player_data.next_exp.to_string(),
        "player.attack" => player_data.attack.to_string(),
        "player.defense" => player_data.defense.to_string(),
        "player.inventory" => player_data
            .inventory
            .iter()
            .take(8)
            .map(|item_id| {
                if let Some(item) = item_registry.get(&item_id.0) {
                    let key = format!("{}:{}", item.locate_file, item.locate_name);
                    mortar_strings.resolve(&key).to_string()
                } else {
                    warn!("Item ID '{}' not found in registry!", item_id.0);
                    format!("UNDEFINED ({})", item_id.0)
                }
            })
            .collect::<Vec<String>>()
            .join("\n"),
        "player.weapon" => {
            if let Some(item) = item_registry.get(&player_data.weapon.0) {
                let key = format!("{}:{}", item.locate_file, item.locate_name);
                mortar_strings.resolve(&key).to_string()
            } else {
                player_data.weapon.0.clone()
            }
        }
        "player.weapon_atk" => {
            if let Some(item) = item_registry.get(&player_data.weapon.0)
                && let ItemType::Weapon { damage, .. } = item.item_type
            {
                return damage.to_string();
            }
            "0".to_string()
        }
        "player.total_attack" => {
            let weapon_atk = if let Some(item) = item_registry.get(&player_data.weapon.0) {
                if let ItemType::Weapon { damage, .. } = item.item_type {
                    damage as usize
                } else {
                    0
                }
            } else {
                0
            };
            (player_data.attack + weapon_atk).to_string()
        }
        "player.armor" => {
            if let Some(item) = item_registry.get(&player_data.armor.0) {
                let key = format!("{}:{}", item.locate_file, item.locate_name);
                mortar_strings.resolve(&key).to_string()
            } else {
                player_data.armor.0.clone()
            }
        }
        "player.armor_def" => {
            if let Some(item) = item_registry.get(&player_data.armor.0)
                && let ItemType::Armor { defense } = item.item_type
            {
                return defense.to_string();
            }
            "0".to_string()
        }
        "player.total_defense" => {
            let armor_def = if let Some(item) = item_registry.get(&player_data.armor.0) {
                if let ItemType::Armor { defense } = item.item_type {
                    defense as usize
                } else {
                    0
                }
            } else {
                0
            };
            (player_data.defense + armor_def).to_string()
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
    player_data: &crate::core::data::PlayerData,
    time: Option<f64>,
) -> f32 {
    match val {
        crate::app_state::battle::chapter_schema::Val::Static(v) => *v,
        crate::app_state::battle::chapter_schema::Val::Expr(expr_str) => {
            if expr_str == "@current" {
                return current_value.unwrap_or(0.0);
            }

            use crate::core::view::layout::FloatOrExpr;
            evaluate_float_expr(&FloatOrExpr::Expr(expr_str.clone()), player_data, time)
        }
    }
}

/// Resolve a Val<bool> to an actual bool value.
///
/// 将 Val<bool> 解析为实际的 bool 值。
pub fn resolve_val_bool(
    val: &crate::app_state::battle::chapter_schema::Val<bool>,
    player_data: &crate::core::data::PlayerData,
) -> bool {
    match val {
        crate::app_state::battle::chapter_schema::Val::Static(v) => *v,
        crate::app_state::battle::chapter_schema::Val::Expr(expr_str) => {
            // Evaluate condition expression
            evaluate_condition(expr_str, player_data)
        }
    }
}
