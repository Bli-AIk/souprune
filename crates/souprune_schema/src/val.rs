//! # val.rs
//!
//! Core value types used across all SoupRune schemas.
//! Provides static-or-expression values for data-driven configuration.
//!
//! 所有 SoupRune Schema 通用的核心值类型。
//! 提供静态值或表达式值，用于数据驱动配置。

use serde::{Deserialize, Serialize};

/// Generic non-string value: static or computed from an expression at runtime.
///
/// Do not use `Val<String>` in schema fields. String dynamics must use explicit
/// domain types such as localization keys, text templates, Mortar, or FRE.
///
/// 泛型非字符串值：静态值或运行时从表达式计算。
///
/// 不要在 Schema 字段中使用 `Val<String>`。字符串动态能力必须使用显式领域类型，
/// 例如本地化键、文本模板、Mortar 或 FRE。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Val<T> {
    Static(T),
    Expr(String),
}

impl<T> Val<T> {
    /// Create a literal value.
    ///
    /// 创建字面量值。
    pub fn literal(value: T) -> Self {
        Self::Static(value)
    }

    /// Create a runtime expression value.
    ///
    /// 创建运行时表达式值。
    pub fn expr(value: impl Into<String>) -> Self {
        Self::Expr(value.into())
    }

    pub fn is_expr(&self) -> bool {
        matches!(self, Val::Expr(_))
    }

    pub fn is_dynamic(&self) -> bool {
        self.is_expr()
    }

    pub fn as_static(&self) -> Option<&T> {
        match self {
            Val::Static(v) => Some(v),
            Val::Expr(_) => None,
        }
    }
}

impl<T> From<T> for Val<T> {
    fn from(value: T) -> Self {
        Self::Static(value)
    }
}

impl<T: Default> Default for Val<T> {
    fn default() -> Self {
        Val::Static(T::default())
    }
}

/// Static floating-point value for authoring dynamic schemas.
///
/// 用于编写动态 Schema 的静态浮点值。
pub fn static_float(value: f32) -> Val<f32> {
    Val::literal(value)
}

/// Runtime floating-point expression for authoring dynamic schemas.
///
/// 用于编写动态 Schema 的运行时浮点表达式。
pub fn expression(value: impl Into<String>) -> Val<f32> {
    Val::expr(value)
}

/// 3D vector: supports both positional `(1.0, 2.0, 3.0)` and named `(x: 1.0, y: 2.0, z: 3.0)`.
///
/// 三维向量：支持位置式 `(1.0, 2.0, 3.0)` 和命名式 `(x: 1.0, y: 2.0, z: 3.0)`。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Vec3Tuple {
    Named {
        x: Val<f32>,
        y: Val<f32>,
        z: Val<f32>,
    },
    Positional(Val<f32>, Val<f32>, Val<f32>),
}

impl Vec3Tuple {
    /// Create a positional 3D vector.
    ///
    /// 创建位置式三维向量。
    pub fn positional(
        x: impl Into<Val<f32>>,
        y: impl Into<Val<f32>>,
        z: impl Into<Val<f32>>,
    ) -> Self {
        Self::Positional(x.into(), y.into(), z.into())
    }

    /// Create a named 3D vector.
    ///
    /// 创建命名式三维向量。
    pub fn named(x: impl Into<Val<f32>>, y: impl Into<Val<f32>>, z: impl Into<Val<f32>>) -> Self {
        Self::Named {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }
}

/// 2D vector: supports both positional and named syntax.
///
/// 二维向量：支持位置式和命名式语法。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Vec2Tuple {
    Named { x: Val<f32>, y: Val<f32> },
    Positional(Val<f32>, Val<f32>),
}

impl Vec2Tuple {
    /// Create a positional 2D vector.
    ///
    /// 创建位置式二维向量。
    pub fn positional(x: impl Into<Val<f32>>, y: impl Into<Val<f32>>) -> Self {
        Self::Positional(x.into(), y.into())
    }

    /// Create a named 2D vector.
    ///
    /// 创建命名式二维向量。
    pub fn named(x: impl Into<Val<f32>>, y: impl Into<Val<f32>>) -> Self {
        Self::Named {
            x: x.into(),
            y: y.into(),
        }
    }
}

/// RGBA color: supports both positional and named `(r:, g:, b:, a:)` syntax.
///
/// RGBA 颜色：支持位置式和命名式 `(r:, g:, b:, a:)` 语法。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ColorTuple {
    Named {
        r: Val<f32>,
        g: Val<f32>,
        b: Val<f32>,
        a: Val<f32>,
    },
    Positional(Val<f32>, Val<f32>, Val<f32>, Val<f32>),
}

impl ColorTuple {
    /// Create a positional RGBA color tuple.
    ///
    /// 创建位置式 RGBA 颜色元组。
    pub fn rgba(
        red: impl Into<Val<f32>>,
        green: impl Into<Val<f32>>,
        blue: impl Into<Val<f32>>,
        alpha: impl Into<Val<f32>>,
    ) -> Self {
        Self::Positional(red.into(), green.into(), blue.into(), alpha.into())
    }

    /// Create a named RGBA color tuple.
    ///
    /// 创建命名式 RGBA 颜色元组。
    pub fn named(
        red: impl Into<Val<f32>>,
        green: impl Into<Val<f32>>,
        blue: impl Into<Val<f32>>,
        alpha: impl Into<Val<f32>>,
    ) -> Self {
        Self::Named {
            r: red.into(),
            g: green.into(),
            b: blue.into(),
            a: alpha.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    #[test]
    fn schema_sources_do_not_define_val_string_fields() {
        let schema_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut offenders = Vec::new();
        collect_val_string_offenders(&schema_root, &mut offenders);

        assert!(
            offenders.is_empty(),
            "schema fields must not use Val<String>: {offenders:?}"
        );
    }

    fn collect_val_string_offenders(current: &Path, offenders: &mut Vec<String>) {
        for entry in fs::read_dir(current).expect("schema source directory should be readable") {
            let entry = entry.expect("schema source entry should be readable");
            let path = entry.path();
            if path.is_dir() {
                collect_val_string_offenders(&path, offenders);
                continue;
            }
            if path.file_name().is_some_and(|name| name == "val.rs") {
                continue;
            }
            if path.extension().is_none_or(|ext| ext != "rs") {
                continue;
            }

            let source = fs::read_to_string(&path).expect("schema source should be readable");
            let compact_source = source
                .chars()
                .filter(|character| !character.is_whitespace())
                .collect::<String>();
            if compact_source.contains("Val<String>") {
                offenders.push(path.display().to_string());
            }
        }
    }
}
