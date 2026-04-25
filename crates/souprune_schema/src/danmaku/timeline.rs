//! # Danmaku Timeline Schema
//!
//! Timeline event and spawn pattern schema types.
//!
//! # 弹幕时间线 Schema
//!
//! 时间线事件与生成模式 Schema 类型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::BulletBehavior;

/// How the `t` field of a timeline event is interpreted.
///
/// 时间线事件的 `t` 字段解释方式。
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub enum TimeMode {
    /// `t` is a delta from the previous event's trigger time (default).
    ///
    /// `t` 是距上一事件触发时间的增量。
    #[default]
    Delta,
    /// `t` is an absolute time since the performance started.
    ///
    /// `t` 是从演出开始算起的绝对时间。
    Absolute,
}

/// Timeline event — describes what happens at a specific time.
///
/// 时间线事件 — 描述在特定时间发生的情况。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimelineEvent {
    /// Time coordinate (interpreted based on `time_mode`).
    ///
    /// 时间坐标（根据 `time_mode` 解释）。
    pub t: f32,
    /// Interpretation mode for the `t` field.
    ///
    /// `t` 字段的解释模式。
    #[serde(default)]
    pub time_mode: TimeMode,
    /// Identifier of the bullet prototype to spawn.
    ///
    /// 要生成的弹幕原型的标识符。
    pub spawn: String,
    /// Spatial distribution pattern for spawned bullets.
    ///
    /// 生成弹幕的空间分布模式。
    #[serde(default)]
    pub pattern: SpawnPattern,
    /// Positional offset applied to the pattern center.
    ///
    /// 应用于模式中心的坐标偏移。
    #[serde(default)]
    pub offset: (f32, f32),
    /// List of named behaviors to apply to the spawned bullets.
    ///
    /// 要应用到生成的弹幕上的命名行为列表。
    #[serde(default)]
    pub apply: Vec<String>,
    /// Inline behavior definitions to apply to the spawned bullets.
    ///
    /// 要应用到生成的弹幕上的内联行为定义。
    #[serde(default)]
    pub behaviors: Vec<BulletBehavior>,
}

impl Default for TimelineEvent {
    fn default() -> Self {
        Self {
            t: 0.0,
            time_mode: TimeMode::default(),
            spawn: String::new(),
            pattern: SpawnPattern::default(),
            offset: (0.0, 0.0),
            apply: Vec::new(),
            behaviors: Vec::new(),
        }
    }
}

impl TimelineEvent {
    /// Create a delta-time event with named behavior applications.
    ///
    /// 创建带命名行为应用的增量时间事件。
    pub fn delta(
        t: f32,
        spawn: impl Into<String>,
        pattern: SpawnPattern,
        apply: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self::delta_with(t, spawn, pattern, (0.0, 0.0), apply, Vec::new())
    }

    /// Create a fully specified delta-time event.
    ///
    /// 创建完整指定的增量时间事件。
    pub fn delta_with(
        t: f32,
        spawn: impl Into<String>,
        pattern: SpawnPattern,
        offset: (f32, f32),
        apply: impl IntoIterator<Item = impl Into<String>>,
        behaviors: impl IntoIterator<Item = BulletBehavior>,
    ) -> Self {
        Self {
            t,
            time_mode: TimeMode::Delta,
            spawn: spawn.into(),
            pattern,
            offset,
            apply: apply.into_iter().map(Into::into).collect(),
            behaviors: behaviors.into_iter().collect(),
        }
    }

    /// Create an absolute-time event with named behavior applications.
    ///
    /// 创建带命名行为应用的绝对时间事件。
    pub fn absolute(
        t: f32,
        spawn: impl Into<String>,
        pattern: SpawnPattern,
        apply: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self::absolute_with(t, spawn, pattern, (0.0, 0.0), apply, Vec::new())
    }

    /// Create a fully specified absolute-time event.
    ///
    /// 创建完整指定的绝对时间事件。
    pub fn absolute_with(
        t: f32,
        spawn: impl Into<String>,
        pattern: SpawnPattern,
        offset: (f32, f32),
        apply: impl IntoIterator<Item = impl Into<String>>,
        behaviors: impl IntoIterator<Item = BulletBehavior>,
    ) -> Self {
        Self {
            t,
            time_mode: TimeMode::Absolute,
            spawn: spawn.into(),
            pattern,
            offset,
            apply: apply.into_iter().map(Into::into).collect(),
            behaviors: behaviors.into_iter().collect(),
        }
    }
}

/// Spawn pattern for timeline events — controls how bullets are distributed when spawned.
///
/// 时间线事件的生成模式 — 控制弹幕生成时的空间分布方式。
///
/// Each variant defines a spatial distribution strategy. Generator variants include a
/// `randomness` parameter (0.0–1.0) that adds positional jitter to the computed spawn
/// points without changing the overall pattern shape.
///
/// 每个变体定义一种空间分布策略。生成器变体包含 `randomness` 参数（0.0–1.0），
/// 在不改变整体图案形状的前提下，为计算出的生成点添加位置抖动。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub enum SpawnPattern {
    /// Spawns a single bullet at the event's position. No pattern generation.
    ///
    /// 在事件位置生成单颗弹幕。不进行图案生成。
    #[default]
    Single,
    /// Distributes bullets evenly around a circle (ring pattern).
    ///
    /// 将弹幕均匀分布在圆环上（环形图案）。
    RingGenerator {
        count: usize,
        #[serde(default)]
        radius: f32,
        #[serde(default)]
        start_angle: f32,
        #[serde(default)]
        randomness: f32,
    },
    /// Distributes bullets along a straight line.
    ///
    /// 将弹幕沿直线分布。
    LineGenerator {
        count: usize,
        #[serde(default = "default_line_spacing")]
        spacing: f32,
        #[serde(default = "default_linear_direction")]
        direction: (f32, f32),
        #[serde(default)]
        randomness: f32,
    },
    /// Distributes bullets along a screen edge at fixed intervals.
    ///
    /// 将弹幕沿屏幕边缘以固定间隔分布。
    EdgeGenerator {
        count: usize,
        #[serde(default)]
        side: EdgeSide,
        #[serde(default = "default_edge_spacing")]
        spacing: f32,
        #[serde(default = "default_edge_margin")]
        margin: f32,
        #[serde(default)]
        randomness: f32,
    },
    /// Spawns bullets relative to a named ViewBox's edge.
    ///
    /// 相对于命名 ViewBox 边缘生成弹幕。
    BoxEdgeGenerator {
        box_name: String,
        count: usize,
        #[serde(default)]
        side: EdgeSide,
        #[serde(default = "default_edge_spacing")]
        spacing: f32,
        #[serde(default)]
        outside_margin: f32,
        #[serde(default)]
        randomness: f32,
    },
    /// User-defined spawn pattern provided by a WASM module.
    ///
    /// 由 WASM 模块提供的自定义生成模式。
    CustomGenerator {
        id: String,
        #[serde(default, serialize_with = "crate::ordered_map::serialize_ordered_map")]
        params: HashMap<String, f32>,
    },
}

impl SpawnPattern {
    /// Create a ring generator pattern.
    ///
    /// 创建环形生成模式。
    pub fn ring(count: usize, radius: f32) -> Self {
        Self::RingGenerator {
            count,
            radius,
            start_angle: 0.0,
            randomness: 0.0,
        }
    }

    /// Create a ring generator pattern with a custom start angle.
    ///
    /// 创建带自定义起始角度的环形生成模式。
    pub fn ring_with_start_angle(count: usize, radius: f32, start_angle: f32) -> Self {
        Self::RingGenerator {
            count,
            radius,
            start_angle,
            randomness: 0.0,
        }
    }

    /// Create a line generator pattern.
    ///
    /// 创建直线生成模式。
    pub fn line(count: usize, spacing: f32, direction: (f32, f32)) -> Self {
        Self::LineGenerator {
            count,
            spacing,
            direction,
            randomness: 0.0,
        }
    }

    /// Create a screen-edge generator pattern.
    ///
    /// 创建屏幕边缘生成模式。
    pub fn edge(side: EdgeSide, count: usize, spacing: f32, margin: f32) -> Self {
        Self::EdgeGenerator {
            count,
            side,
            spacing,
            margin,
            randomness: 0.0,
        }
    }

    /// Create a ViewBox-edge generator pattern.
    ///
    /// 创建 ViewBox 边缘生成模式。
    pub fn box_edge(
        box_name: impl Into<String>,
        side: EdgeSide,
        count: usize,
        spacing: f32,
        outside_margin: f32,
    ) -> Self {
        Self::BoxEdgeGenerator {
            box_name: box_name.into(),
            count,
            side,
            spacing,
            outside_margin,
            randomness: 0.0,
        }
    }

    /// Create a custom WASM-provided generator pattern.
    ///
    /// 创建由 WASM 提供的自定义生成模式。
    pub fn custom(
        id: impl Into<String>,
        params: impl IntoIterator<Item = (impl Into<String>, f32)>,
    ) -> Self {
        Self::CustomGenerator {
            id: id.into(),
            params: params
                .into_iter()
                .map(|(key, value)| (key.into(), value))
                .collect(),
        }
    }

    /// Returns the randomness factor (0.0–1.0) for this pattern.
    ///
    /// 返回此模式的随机因子（0.0–1.0）。
    pub fn randomness(&self) -> f32 {
        match self {
            Self::Single | Self::CustomGenerator { .. } => 0.0,
            Self::RingGenerator { randomness, .. }
            | Self::LineGenerator { randomness, .. }
            | Self::EdgeGenerator { randomness, .. }
            | Self::BoxEdgeGenerator { randomness, .. } => *randomness,
        }
    }

    /// Returns a representative spacing distance between adjacent spawn points.
    ///
    /// 返回相邻生成点之间的代表性间距。
    pub fn characteristic_spacing(&self) -> f32 {
        match self {
            Self::Single | Self::CustomGenerator { .. } => 0.0,
            Self::RingGenerator { count, radius, .. } => {
                if *count <= 1 {
                    *radius
                } else {
                    std::f32::consts::TAU * radius / *count as f32
                }
            }
            Self::LineGenerator { spacing, .. }
            | Self::EdgeGenerator { spacing, .. }
            | Self::BoxEdgeGenerator { spacing, .. } => *spacing,
        }
    }
}

/// Which screen edge to spawn from.
///
/// 从哪条屏幕边缘生成。
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub enum EdgeSide {
    #[default]
    Left,
    Right,
    Top,
    Bottom,
}

fn default_line_spacing() -> f32 {
    20.0
}

fn default_linear_direction() -> (f32, f32) {
    (0.0, -1.0)
}

fn default_edge_spacing() -> f32 {
    30.0
}

fn default_edge_margin() -> f32 {
    200.0
}
