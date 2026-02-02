use super::super::components::IndexBound;
use super::super::layout::FloatOrExpr;
use crate::app_state::battle::chapter_schema::Val;
use crate::app_state::overworld::OverworldSubState;
use crate::core::item::ItemId;
use bevy::prelude::*;
use bevy_fact_rule_event::{FactDatabase, FactValue, LayeredFactDatabase};

/// Helper struct to read player data from LayeredFactDatabase.
/// This provides a view into player facts for the View system.
///
/// 从 LayeredFactDatabase 读取玩家数据的辅助结构体。
/// 为 View 系统提供玩家事实的视图。
pub struct PlayerDataView<'a> {
    db: &'a LayeredFactDatabase,
    /// Optional local facts from ViewRoot (View-specific facts)
    /// 来自 ViewRoot 的可选局部事实（View 特定的事实）
    local_facts: Option<&'a FactDatabase>,
}

impl<'a> PlayerDataView<'a> {
    pub fn new(db: &'a LayeredFactDatabase) -> Self {
        Self {
            db,
            local_facts: None,
        }
    }

    /// Create a PlayerDataView with local facts from a ViewRoot.
    ///
    /// 创建一个带有来自 ViewRoot 局部事实的 PlayerDataView。
    pub fn with_local_facts(db: &'a LayeredFactDatabase, local_facts: &'a FactDatabase) -> Self {
        Self {
            db,
            local_facts: Some(local_facts),
        }
    }

    /// Get a fact value with priority: local_facts -> scene -> global.
    /// Supports `fact('key')` and `$key` syntax.
    ///
    /// 获取事实值，优先级为：local_facts -> scene -> global。
    /// 支持 `fact('key')` 和 `$key` 语法。
    pub fn get_fact(&self, key: &str) -> Option<&FactValue> {
        // First check local facts
        if let Some(local) = self.local_facts {
            if let Some(value) = local.get_by_str(key) {
                return Some(value);
            }
        }
        // Then check layered database (scene -> global)
        self.db.get_by_str(key)
    }

    /// Get a fact value as f64, with optional default.
    ///
    /// 获取事实值为 f64，带可选默认值。
    pub fn get_fact_float(&self, key: &str, default: Option<f64>) -> f64 {
        if let Some(value) = self.get_fact(key) {
            match value {
                FactValue::Float(f) => *f,
                FactValue::Int(i) => *i as f64,
                FactValue::Bool(b) => {
                    if *b {
                        1.0
                    } else {
                        0.0
                    }
                }
                FactValue::String(_) => default.unwrap_or(0.0),
            }
        } else {
            default.unwrap_or(0.0)
        }
    }

    pub fn name(&self) -> String {
        self.db
            .get_string("player_name")
            .unwrap_or("???")
            .to_string()
    }

    pub fn lv(&self) -> usize {
        self.db.get_int("player_lv").unwrap_or(1) as usize
    }

    pub fn exp(&self) -> usize {
        self.db.get_int("player_exp").unwrap_or(0) as usize
    }

    pub fn next_exp(&self) -> usize {
        self.db.get_int("player_next_exp").unwrap_or(10) as usize
    }

    pub fn hp(&self) -> usize {
        self.db.get_int("player_hp").unwrap_or(20) as usize
    }

    pub fn hp_max(&self) -> usize {
        self.db.get_int("player_hp_max").unwrap_or(20) as usize
    }

    pub fn attack(&self) -> usize {
        self.db.get_int("player_atk").unwrap_or(0) as usize
    }

    pub fn defense(&self) -> usize {
        self.db.get_int("player_def").unwrap_or(0) as usize
    }

    pub fn gold(&self) -> usize {
        self.db.get_int("player_gold").unwrap_or(0) as usize
    }

    pub fn weapon(&self) -> ItemId {
        ItemId(
            self.db
                .get_string("player_weapon")
                .unwrap_or("stick")
                .to_string(),
        )
    }

    pub fn armor(&self) -> ItemId {
        ItemId(
            self.db
                .get_string("player_armor")
                .unwrap_or("bandage")
                .to_string(),
        )
    }

    pub fn inventory(&self) -> Vec<ItemId> {
        self.db
            .get_string("player_inventory")
            .map(|s| {
                s.split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| ItemId(s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn inventory_capacity(&self) -> usize {
        self.db.get_int("player_inventory_capacity").unwrap_or(8) as usize
    }
}

pub fn parse_overworld_state(state_str: &str) -> Option<OverworldSubState> {
    match state_str {
        "" | "None" => None,
        name => Some(OverworldSubState::new(name)),
    }
}

pub fn evaluate_index_bound(bound: &IndexBound, player_data: &PlayerDataView) -> usize {
    match bound {
        IndexBound::Static(value) => *value,
        IndexBound::Dynamic(expr) => evaluate_index_expression(expr, player_data),
    }
}

fn evaluate_index_expression(expr: &str, player_data: &PlayerDataView) -> usize {
    let expr = expr.trim();

    if expr == "inventory.len()" {
        return player_data.inventory().len();
    }

    if expr == "inventory_capacity" {
        return player_data.inventory_capacity();
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

/// Preprocess expression string to handle `$name` and `fact('name')` syntax.
/// - `$name` is converted to the actual fact value
/// - `fact('name')` is converted to the actual fact value
/// - `fact_or('name', default)` is converted to the actual fact value or default
///
/// 预处理表达式字符串以处理 `$name` 和 `fact('name')` 语法。
/// - `$name` 会被转换为实际的 fact 值
/// - `fact('name')` 会被转换为实际的 fact 值
/// - `fact_or('name', default)` 会被转换为实际的 fact 值或默认值
fn preprocess_fact_expressions(expr: &str, player_data: &PlayerDataView) -> String {
    let mut result = expr.to_string();

    // Handle $name syntax: replace $word with the fact value
    // 处理 $name 语法：将 $word 替换为 fact 值
    let dollar_regex = regex::Regex::new(r"\$([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    result = dollar_regex
        .replace_all(&result, |caps: &regex::Captures| {
            let key = &caps[1];
            format!("{}", player_data.get_fact_float(key, Some(0.0)))
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
            let default_val = default_str.parse::<f64>().unwrap_or(0.0);
            format!("{}", player_data.get_fact_float(key, Some(default_val)))
        })
        .to_string();

    // Handle fact('name') syntax
    // 处理 fact('name') 语法
    let fact_regex = regex::Regex::new(r#"fact\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap();
    result = fact_regex
        .replace_all(&result, |caps: &regex::Captures| {
            let key = &caps[1];
            format!("{}", player_data.get_fact_float(key, Some(0.0)))
        })
        .to_string();

    result
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

            let _ = context.set_value(
                "@player.hp".to_string(),
                evalexpr::Value::Int(player_data.hp() as i64),
            );
            let _ = context.set_value(
                "@player.hp_max".to_string(),
                evalexpr::Value::Int(player_data.hp_max() as i64),
            );
            let _ = context.set_value(
                "@player.lv".to_string(),
                evalexpr::Value::Int(player_data.lv() as i64),
            );

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

/// Evaluate a float expression string with support for @current variable.
/// This is used by the tween system where @current represents the current value.
///
/// 计算支持 @current 变量的浮点表达式字符串。
/// 这由 tween 系统使用，其中 @current 代表当前值。
pub fn evaluate_float_expr_with_current(
    expr_str: &str,
    current_value: Option<f32>,
    player_data: &PlayerDataView,
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

    // Set player data variables
    let _ = context.set_value(
        "@player.hp".to_string(),
        evalexpr::Value::Int(player_data.hp() as i64),
    );
    let _ = context.set_value(
        "@player.hp_max".to_string(),
        evalexpr::Value::Int(player_data.hp_max() as i64),
    );
    let _ = context.set_value(
        "@player.lv".to_string(),
        evalexpr::Value::Int(player_data.lv() as i64),
    );

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

pub fn evaluate_condition(condition: &str, player_data: &PlayerDataView) -> bool {
    match condition {
        "player.inventory.is_empty" => player_data.inventory().is_empty(),
        "player.inventory.is_not_empty" => !player_data.inventory().is_empty(),
        _ => {
            if condition.starts_with("player.") {
                let parts: Vec<&str> = condition.split('.').collect();
                if parts.len() >= 3 {
                    match (parts[1], parts[2]) {
                        ("hp", "is_low") => player_data.hp() < player_data.hp_max() / 4,
                        ("hp", "is_critical") => player_data.hp() <= 1,
                        ("gold", "is_zero") => player_data.gold() == 0,
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

    match path {
        "player.name" => player_data.name(),
        "player.lv" => player_data.lv().to_string(),
        "player.hp" => {
            let result = player_data.hp().to_string();
            info!("[resolve_data_path] player.hp = {}", result);
            result
        }
        "player.hp_max" => {
            let result = player_data.hp_max().to_string();
            info!("[resolve_data_path] player.hp_max = {}", result);
            result
        }
        "player.gold" => player_data.gold().to_string(),
        "player.exp" => player_data.exp().to_string(),
        "player.next_exp" => player_data.next_exp().to_string(),
        "player.attack" => player_data.attack().to_string(),
        "player.defense" => player_data.defense().to_string(),
        "player.inventory" => player_data
            .inventory()
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
            let weapon = player_data.weapon();
            if let Some(item) = item_registry.get(&weapon.0) {
                let key = format!("{}:{}", item.locate_file, item.locate_name);
                mortar_strings.resolve(&key).to_string()
            } else {
                weapon.0
            }
        }
        "player.weapon_atk" => {
            let weapon = player_data.weapon();
            if let Some(item) = item_registry.get(&weapon.0)
                && let ItemType::Weapon { damage, .. } = item.item_type
            {
                return damage.to_string();
            }
            "0".to_string()
        }
        "player.total_attack" => {
            let weapon = player_data.weapon();
            let weapon_atk = if let Some(item) = item_registry.get(&weapon.0) {
                if let ItemType::Weapon { damage, .. } = item.item_type {
                    damage as usize
                } else {
                    0
                }
            } else {
                0
            };
            (player_data.attack() + weapon_atk).to_string()
        }
        "player.armor" => {
            let armor = player_data.armor();
            if let Some(item) = item_registry.get(&armor.0) {
                let key = format!("{}:{}", item.locate_file, item.locate_name);
                mortar_strings.resolve(&key).to_string()
            } else {
                armor.0
            }
        }
        "player.armor_def" => {
            let armor = player_data.armor();
            if let Some(item) = item_registry.get(&armor.0)
                && let ItemType::Armor { defense } = item.item_type
            {
                return defense.to_string();
            }
            "0".to_string()
        }
        "player.total_defense" => {
            let armor = player_data.armor();
            let armor_def = if let Some(item) = item_registry.get(&armor.0) {
                if let ItemType::Armor { defense } = item.item_type {
                    defense as usize
                } else {
                    0
                }
            } else {
                0
            };
            (player_data.defense() + armor_def).to_string()
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
                    if player_data.inventory().is_empty() {
                        return false;
                    }
                } else if *part == "player.inventory.is_empty"
                    && !player_data.inventory().is_empty()
                {
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
