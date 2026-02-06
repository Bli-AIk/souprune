//! # bindings.rs
//!
//! # 属性绑定模块
//!
//! Defines PropertyBinding<T> for static values vs dynamic expressions.
//!
//! 定义 PropertyBinding<T>，用于区分静态值和动态表达式。

use std::fmt::Debug;

/// Property can be a static value or a dynamic expression binding.
/// This is the core abstraction that enables declarative UI programming.
///
/// 属性可以是静态值或动态表达式绑定。
/// 这是实现声明式 UI 编程的核心抽象。
///
/// # Examples
///
/// ```ron
/// // Static value
/// transform: (translation: (100.0, 200.0, 1.0), ...)
///
/// // Dynamic expression
/// transform: (translation: ("$x * 10", "$y", 1.0), ...)
///
/// // Mixed
/// visible_when: "$depth == 1 && $active"
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyBinding<T: Clone + Debug + PartialEq> {
    /// Static value that never changes.
    /// 永不变化的静态值。
    Static(T),

    /// Expression binding that is re-evaluated on each reconciliation.
    /// 在每次协调时重新计算的表达式绑定。
    Expr {
        /// The expression string (e.g., "$enemy_hp[@i]", "$active && $depth == 1")
        /// 表达式字符串
        expression: String,
    },
}

impl<T: Clone + Debug + PartialEq> PropertyBinding<T> {
    /// Create a static binding.
    /// 创建静态绑定。
    pub fn new_static(value: T) -> Self {
        Self::Static(value)
    }

    /// Create an expression binding.
    /// 创建表达式绑定。
    pub fn new_expr(expression: impl Into<String>) -> Self {
        Self::Expr {
            expression: expression.into(),
        }
    }

    /// Check if this binding is static.
    /// 检查此绑定是否为静态。
    pub fn is_static(&self) -> bool {
        matches!(self, Self::Static(_))
    }

    /// Check if this binding is an expression.
    /// 检查此绑定是否为表达式。
    pub fn is_expr(&self) -> bool {
        matches!(self, Self::Expr { .. })
    }

    /// Get the expression string if this is an expression binding.
    /// 如果这是表达式绑定，则获取表达式字符串。
    pub fn expression(&self) -> Option<&str> {
        match self {
            Self::Expr { expression } => Some(expression),
            Self::Static(_) => None,
        }
    }

    /// Get the static value if this is a static binding.
    /// 如果这是静态绑定，则获取静态值。
    pub fn static_value(&self) -> Option<&T> {
        match self {
            Self::Static(v) => Some(v),
            Self::Expr { .. } => None,
        }
    }
}

impl<T: Clone + Debug + PartialEq + Default> Default for PropertyBinding<T> {
    fn default() -> Self {
        Self::Static(T::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_binding() {
        let binding: PropertyBinding<i32> = PropertyBinding::new_static(42);
        assert!(binding.is_static());
        assert!(!binding.is_expr());
        assert_eq!(binding.static_value(), Some(&42));
        assert_eq!(binding.expression(), None);
    }

    #[test]
    fn test_expr_binding() {
        let binding: PropertyBinding<i32> = PropertyBinding::new_expr("$hp * 2");
        assert!(!binding.is_static());
        assert!(binding.is_expr());
        assert_eq!(binding.static_value(), None);
        assert_eq!(binding.expression(), Some("$hp * 2"));
    }
}
