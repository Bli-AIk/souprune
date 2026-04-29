//! Native authoring helpers for View numeric expressions.
//!
//! View 数值表达式的原生编写辅助。
//!
//! This module builds the legacy string expressions still consumed by the
//! runtime, while giving content authors typed Rust functions and operators.
//!
//! 本模块生成运行时仍然消费的旧字符串表达式，
//! 同时为内容作者提供有类型的 Rust 函数与运算符。

use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use souprune_schema::fre::LocalFactValue;
use souprune_schema::sequence::FactValueMatch;
use souprune_schema::val::Val;
use souprune_schema::view::MaterialParamValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Compare,
    Add,
    Multiply,
    Unary,
    Atom,
}

/// A numeric expression that can be exported to the existing schema format.
///
/// 可以导出为现有 Schema 格式的数值表达式。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
    source: String,
    precedence: Precedence,
}

impl Expression {
    fn new(source: impl Into<String>, precedence: Precedence) -> Self {
        Self {
            source: source.into(),
            precedence,
        }
    }

    fn atom(source: impl Into<String>) -> Self {
        Self::new(source, Precedence::Atom)
    }

    fn render_for_left(&self, parent: Precedence) -> String {
        if self.precedence < parent {
            format!("({})", self.source)
        } else {
            self.source.clone()
        }
    }

    fn render_for_right(&self, parent: Precedence, parent_operator: &str) -> String {
        let needs_equal_parentheses = matches!(parent_operator, "-" | "/" | "%");
        if self.precedence < parent || (self.precedence == parent && needs_equal_parentheses) {
            format!("({})", self.source)
        } else {
            self.source.clone()
        }
    }

    /// Return the generated legacy expression source.
    ///
    /// 返回生成的旧表达式源码。
    pub fn as_str(&self) -> &str {
        &self.source
    }

    /// Convert this expression into a schema float value.
    ///
    /// 将当前表达式转换为 Schema 浮点值。
    pub fn into_schema(self) -> Val<f32> {
        Val::expr(self.source)
    }

    /// Convert this expression into its generated legacy source string.
    ///
    /// 将当前表达式转换为生成的旧源码字符串。
    pub fn into_string(self) -> String {
        self.source
    }

    /// Convert this expression into a material parameter value.
    ///
    /// 将当前表达式转换为材质参数值。
    pub fn into_material_param(self) -> MaterialParamValue {
        MaterialParamValue::Expr(self.source)
    }

    /// Compare this expression with another expression for equality.
    ///
    /// 将当前表达式与另一个表达式做相等比较。
    pub fn equal_to(self, rhs: impl Into<Expression>) -> Condition {
        compare(self, "==", rhs.into())
    }

    /// Compare this expression with another expression for inequality.
    ///
    /// 将当前表达式与另一个表达式做不等比较。
    pub fn not_equal_to(self, rhs: impl Into<Expression>) -> Condition {
        compare(self, "!=", rhs.into())
    }

    /// Compare whether this expression is less than another expression.
    ///
    /// 比较当前表达式是否小于另一个表达式。
    pub fn less_than(self, rhs: impl Into<Expression>) -> Condition {
        compare(self, "<", rhs.into())
    }

    /// Compare whether this expression is less than or equal to another expression.
    ///
    /// 比较当前表达式是否小于或等于另一个表达式。
    pub fn less_than_or_equal_to(self, rhs: impl Into<Expression>) -> Condition {
        compare(self, "<=", rhs.into())
    }

    /// Compare whether this expression is greater than another expression.
    ///
    /// 比较当前表达式是否大于另一个表达式。
    pub fn greater_than(self, rhs: impl Into<Expression>) -> Condition {
        compare(self, ">", rhs.into())
    }

    /// Compare whether this expression is greater than or equal to another expression.
    ///
    /// 比较当前表达式是否大于或等于另一个表达式。
    pub fn greater_than_or_equal_to(self, rhs: impl Into<Expression>) -> Condition {
        compare(self, ">=", rhs.into())
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.source)
    }
}

impl From<Expression> for String {
    fn from(value: Expression) -> Self {
        value.source
    }
}

impl From<Expression> for Val<f32> {
    fn from(value: Expression) -> Self {
        value.into_schema()
    }
}

impl From<Expression> for MaterialParamValue {
    fn from(value: Expression) -> Self {
        value.into_material_param()
    }
}

impl From<Expression> for LocalFactValue {
    fn from(value: Expression) -> Self {
        Self::Expr(value.source)
    }
}

impl From<Expression> for FactValueMatch {
    fn from(value: Expression) -> Self {
        Self::Expr(value.source)
    }
}

impl From<Expression> for Option<Val<f32>> {
    fn from(value: Expression) -> Self {
        Some(value.into_schema())
    }
}

impl From<&Expression> for Expression {
    fn from(value: &Expression) -> Self {
        value.clone()
    }
}

impl From<f32> for Expression {
    fn from(value: f32) -> Self {
        Self::atom(format_float32(value))
    }
}

impl From<f64> for Expression {
    fn from(value: f64) -> Self {
        Self::atom(format_float64(value))
    }
}

macro_rules! impl_integer_expression_from {
    ($($ty:ty),* $(,)?) => {
        $(
            impl From<$ty> for Expression {
                fn from(value: $ty) -> Self {
                    Self::atom(value.to_string())
                }
            }
        )*
    };
}

impl_integer_expression_from!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

macro_rules! impl_expression_binary_operator {
    ($trait:ident, $method:ident, $operator:literal, $precedence:expr) => {
        impl<T> $trait<T> for Expression
        where
            T: Into<Expression>,
        {
            type Output = Expression;

            fn $method(self, rhs: T) -> Self::Output {
                binary(self, $operator, rhs.into(), $precedence)
            }
        }

        impl<T> $trait<T> for &Expression
        where
            T: Into<Expression>,
        {
            type Output = Expression;

            fn $method(self, rhs: T) -> Self::Output {
                binary(self.clone(), $operator, rhs.into(), $precedence)
            }
        }
    };
}

impl_expression_binary_operator!(Add, add, "+", Precedence::Add);
impl_expression_binary_operator!(Sub, sub, "-", Precedence::Add);
impl_expression_binary_operator!(Mul, mul, "*", Precedence::Multiply);
impl_expression_binary_operator!(Div, div, "/", Precedence::Multiply);
impl_expression_binary_operator!(Rem, rem, "%", Precedence::Multiply);

macro_rules! impl_left_numeric_binary_operator {
    ($ty:ty, $trait:ident, $method:ident, $operator:literal, $precedence:expr) => {
        impl $trait<Expression> for $ty {
            type Output = Expression;

            fn $method(self, rhs: Expression) -> Self::Output {
                binary(Expression::from(self), $operator, rhs, $precedence)
            }
        }
    };
}

macro_rules! impl_left_numeric_binary_operators {
    ($($ty:ty),* $(,)?) => {
        $(
            impl_left_numeric_binary_operator!($ty, Add, add, "+", Precedence::Add);
            impl_left_numeric_binary_operator!($ty, Sub, sub, "-", Precedence::Add);
            impl_left_numeric_binary_operator!($ty, Mul, mul, "*", Precedence::Multiply);
            impl_left_numeric_binary_operator!($ty, Div, div, "/", Precedence::Multiply);
            impl_left_numeric_binary_operator!($ty, Rem, rem, "%", Precedence::Multiply);
        )*
    };
}

impl_left_numeric_binary_operators!(f64, i32);

impl Neg for Expression {
    type Output = Expression;

    fn neg(self) -> Self::Output {
        let source = if self.precedence < Precedence::Unary {
            format!("-({})", self.source)
        } else {
            format!("-{}", self.source)
        };
        Expression::new(source, Precedence::Unary)
    }
}

impl Neg for &Expression {
    type Output = Expression;

    fn neg(self) -> Self::Output {
        -self.clone()
    }
}

/// A boolean expression fragment for numeric condition arguments.
///
/// 用于数值条件参数的布尔表达式片段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Condition {
    source: String,
}

impl Condition {
    fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }

    /// Return the generated legacy condition source.
    ///
    /// 返回生成的旧条件源码。
    pub fn as_str(&self) -> &str {
        &self.source
    }

    /// Convert this condition into its generated legacy source string.
    ///
    /// 将当前条件转换为生成的旧源码字符串。
    pub fn into_string(self) -> String {
        self.source
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.source)
    }
}

impl From<Condition> for String {
    fn from(value: Condition) -> Self {
        value.source
    }
}

impl From<&Condition> for Condition {
    fn from(value: &Condition) -> Self {
        value.clone()
    }
}

fn binary(lhs: Expression, operator: &str, rhs: Expression, precedence: Precedence) -> Expression {
    let lhs_source = lhs.render_for_left(precedence);
    let rhs_source = rhs.render_for_right(precedence, operator);
    Expression::new(format!("{lhs_source} {operator} {rhs_source}"), precedence)
}

fn compare(lhs: Expression, operator: &str, rhs: Expression) -> Condition {
    let lhs_source = lhs.render_for_left(Precedence::Compare);
    let rhs_source = rhs.render_for_right(Precedence::Compare, operator);
    Condition::new(format!("{lhs_source} {operator} {rhs_source}"))
}

fn function(name: &str, args: Vec<Expression>) -> Expression {
    let source = args
        .into_iter()
        .map(|arg| arg.source)
        .collect::<Vec<_>>()
        .join(", ");
    Expression::atom(format!("{name}({source})"))
}

fn format_float32(value: f32) -> String {
    if value.is_finite() && value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}

fn format_float64(value: f64) -> String {
    if value.is_finite() && value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}

/// Create a literal numeric expression.
///
/// 创建字面量数值表达式。
pub fn literal(value: impl Into<Expression>) -> Expression {
    value.into()
}

/// Create a frame interval expression such as `1.0/30.0`.
///
/// 创建形如 `1.0/30.0` 的帧间隔表达式。
pub fn frame_step(frames_per_second: impl Into<Expression>) -> Expression {
    let frames_per_second = frames_per_second.into();
    Expression::atom(format!("1.0/{}", frames_per_second.source))
}

/// Reference the runtime `@time` variable.
///
/// 引用运行时 `@time` 变量。
pub fn time() -> Expression {
    Expression::atom("@time")
}

/// Reference the tween `@current` variable.
///
/// 引用 tween 的 `@current` 变量。
pub fn current() -> Expression {
    Expression::atom("@current")
}

/// Reference the repeat index variable `@i`.
///
/// 引用重复索引变量 `@i`。
pub fn repeat_index() -> Expression {
    Expression::atom("@i")
}

/// Reference the repeat index variable `@index`.
///
/// 引用重复索引变量 `@index`。
pub fn repeat_index_alias() -> Expression {
    Expression::atom("@index")
}

/// Reference a fact value such as `$player:hp_max`.
///
/// 引用形如 `$player:hp_max` 的 fact 值。
pub fn fact(name: impl Into<String>) -> Expression {
    Expression::atom(format!("${}", name.into()))
}

/// Reference a fact array element such as `$enemy_hps[@i]`.
///
/// 引用形如 `$enemy_hps[@i]` 的 fact 数组元素。
pub fn fact_at(name: impl Into<String>, index: impl Into<Expression>) -> Expression {
    Expression::atom(format!("${}[{}]", name.into(), index.into().source))
}

/// Reference a dynamically named fact field such as `$${current_enemy_id}.act_count`.
///
/// 引用形如 `$${current_enemy_id}.act_count` 的动态命名 fact 字段。
pub fn dynamic_fact(anchor: impl Into<String>, field: impl Into<String>) -> Expression {
    Expression::atom(format!("$${{{}}}.{}", anchor.into(), field.into()))
}

/// Reference a dynamically named fact array element such as `$${current_enemy_id}.items[$i]`.
///
/// 引用形如 `$${current_enemy_id}.items[$i]` 的动态命名 fact 数组元素。
pub fn dynamic_fact_at(
    anchor: impl Into<String>,
    field: impl Into<String>,
    index: impl Into<Expression>,
) -> Expression {
    Expression::atom(format!(
        "$${{{}}}.{}[{}]",
        anchor.into(),
        field.into(),
        index.into().source
    ))
}

/// Preserve an explicit parenthesized group in generated output.
///
/// 在生成结果中保留显式括号分组。
pub fn group(value: impl Into<Expression>) -> Expression {
    Expression::atom(format!("({})", value.into().source))
}

/// Build `sin(value)`.
///
/// 构建 `sin(value)`。
pub fn sin(value: impl Into<Expression>) -> Expression {
    function("sin", vec![value.into()])
}

/// Build `cos(value)`.
///
/// 构建 `cos(value)`。
pub fn cos(value: impl Into<Expression>) -> Expression {
    function("cos", vec![value.into()])
}

/// Build `floor(value)`.
///
/// 构建 `floor(value)`。
pub fn floor(value: impl Into<Expression>) -> Expression {
    function("floor", vec![value.into()])
}

/// Build `snap(value, step)`.
///
/// 构建 `snap(value, step)`。
pub fn snap(value: impl Into<Expression>, step: impl Into<Expression>) -> Expression {
    function("snap", vec![value.into(), step.into()])
}

/// Build `max_strlen($list)`.
///
/// 构建 `max_strlen($list)`。
pub fn max_strlen(list: impl Into<Expression>) -> Expression {
    function("max_strlen", vec![list.into()])
}

/// Build `random()`.
///
/// 构建 `random()`。
pub fn random() -> Expression {
    function("random", Vec::new())
}

/// Build `random(max)`.
///
/// 构建 `random(max)`。
pub fn random_to(max: impl Into<Expression>) -> Expression {
    function("random", vec![max.into()])
}

/// Build `random(min, max)`.
///
/// 构建 `random(min, max)`。
pub fn random_range(min: impl Into<Expression>, max: impl Into<Expression>) -> Expression {
    function("random", vec![min.into(), max.into()])
}

/// Build `if(condition, then, else)`.
///
/// 构建 `if(condition, then, else)`。
pub fn if_else(
    condition: impl Into<Condition>,
    then_branch: impl Into<Expression>,
    else_branch: impl Into<Expression>,
) -> Expression {
    let condition = condition.into();
    function(
        "if",
        vec![
            Expression::atom(condition.source),
            then_branch.into(),
            else_branch.into(),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_expression_matches_mad_dummy_rotation_string() {
        let animation = -sin(snap(time(), frame_step(30.0)) * 4.0) * 3.0;

        assert_eq!(
            animation.to_string(),
            "-sin(snap(@time, 1.0/30.0) * 4.0) * 3.0"
        );
    }

    #[test]
    fn animation_expression_matches_sans_idle_strings() {
        let snapped_time = snap(time() * 0.5, frame_step(30.0));
        let x = cos(&snapped_time * 10.0);
        let y = literal(23.0) + group(sin(snapped_time * 20.0) / 1.5);

        assert_eq!(x.to_string(), "cos(snap(@time * 0.5, 1.0/30.0) * 10.0)");
        assert_eq!(
            y.to_string(),
            "23.0 + (sin(snap(@time * 0.5, 1.0/30.0) * 20.0) / 1.5)"
        );
    }

    #[test]
    fn fact_arithmetic_expression_matches_hud_position_string() {
        let x = literal(-5.5) + (fact("player:hp_max") - 20) * 94.5 / 79;

        assert_eq!(x.to_string(), "-5.5 + ($player:hp_max - 20) * 94.5 / 79");
    }

    #[test]
    fn repeat_expression_matches_enemy_hp_bar_strings() {
        let x = literal(15) * max_strlen(fact("enemy_names")) - 125;
        let y = literal(31.25) - (repeat_index() - fact("enemy_view_offset")) * 32.0;
        let ratio = fact_at("enemy_hps", repeat_index()) / fact_at("enemy_hp_maxs", repeat_index());

        assert_eq!(x.to_string(), "15 * max_strlen($enemy_names) - 125");
        assert_eq!(y.to_string(), "31.25 - (@i - $enemy_view_offset) * 32.0");
        assert_eq!(ratio.to_string(), "$enemy_hps[@i] / $enemy_hp_maxs[@i]");
    }

    #[test]
    fn conditional_expression_matches_cursor_position_strings() {
        let act_x = if_else((fact("act_selection") % 2).equal_to(0), -248.0, 11.5);
        let item_y = literal(-45.5) - floor(fact("item_selection") % 4 / 2) * 32.0;

        assert_eq!(
            act_x.to_string(),
            "if($act_selection % 2 == 0, -248.0, 11.5)"
        );
        assert_eq!(
            item_y.to_string(),
            "-45.5 - floor($item_selection % 4 / 2) * 32.0"
        );
    }

    #[test]
    fn expression_converts_to_existing_schema_values() {
        let schema_value = current().into_schema();
        let material_value = (fact("player:hp") / fact("player:hp_max")).into_material_param();
        let fre_value = dynamic_fact_at("current_enemy_id", "action_params", fact("act_selection"))
            .into_string();

        assert!(matches!(schema_value, Val::Expr(source) if source == "@current"));
        assert!(matches!(
            material_value,
            MaterialParamValue::Expr(source) if source == "$player:hp / $player:hp_max"
        ));
        assert_eq!(
            fre_value,
            "$${current_enemy_id}.action_params[$act_selection]"
        );
    }

    #[test]
    fn expression_uses_into_for_authoring_boundary_values() {
        let schema_value: Val<f32> = current().into();
        let optional_schema_value: Option<Val<f32>> = (time() * 2.0).into();
        let material_value: MaterialParamValue = (fact("player:hp") / fact("player:hp_max")).into();
        let local_fact_value: LocalFactValue = dynamic_fact("current_enemy_id", "hp").into();
        let fact_match: FactValueMatch = fact("turn_group").into();

        assert!(matches!(schema_value, Val::Expr(source) if source == "@current"));
        assert!(matches!(
            optional_schema_value,
            Some(Val::Expr(source)) if source == "@time * 2.0"
        ));
        assert!(matches!(
            material_value,
            MaterialParamValue::Expr(source) if source == "$player:hp / $player:hp_max"
        ));
        assert!(matches!(
            local_fact_value,
            LocalFactValue::Expr(source) if source == "$${current_enemy_id}.hp"
        ));
        assert!(matches!(
            fact_match,
            FactValueMatch::Expr(source) if source == "$turn_group"
        ));
    }
}
