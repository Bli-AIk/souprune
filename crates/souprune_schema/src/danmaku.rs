//! # danmaku.rs
//!
//! DanmakuPerformance schema types for `.performance.ron` files.
//! Mirrors `souprune::core::danmaku::danmaku_schema` without Bevy dependency.
//!
//! `.performance.ron` 文件的弹幕演出 Schema 类型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Core Types
// ============================================================================

/// Danmaku Performance asset — top-level `.performance.ron` schema.
///
/// 弹幕演出资源 — `.performance.ron` 的顶层 Schema。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DanmakuPerformance {
    #[serde(default)]
    pub prototypes: HashMap<String, BulletPrototype>,
    #[serde(default)]
    pub behaviors: HashMap<String, BulletBehavior>,
    pub timeline: Vec<TimelineEvent>,
    /// How long this performance lasts. Supports `@current` (accumulated timeline time).
    /// - Literal: `duration: 5.0`
    /// - Expression: `duration: "@current + 2.0"`
    /// - Omitted / 0 / negative: fire-and-forget (won't block sequencer).
    ///
    /// 演出持续时间。支持 `@current`（timeline 累计时间）。
    /// - 字面量：`duration: 5.0`
    /// - 表达式：`duration: "@current + 2.0"`
    /// - 省略 / 0 / 负数：不阻塞序列器。
    #[serde(default)]
    pub duration: Option<DurationExpr>,
}

/// Duration expression — either a literal float or a `@current`-relative expression.
///
/// 持续时间表达式 — 字面量浮点数或基于 `@current` 的相对表达式。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum DurationExpr {
    Literal(f32),
    Expr(String),
}

impl DanmakuPerformance {
    /// Resolve `duration` to an `f32`, substituting `@current` with accumulated timeline time.
    /// Returns `None` if duration is omitted, or `Some(0.0)` / negative → caller treats as
    /// fire-and-forget.
    ///
    /// 将 `duration` 解析为 `f32`，用 timeline 累计时间替换 `@current`。
    pub fn resolved_duration(&self) -> Option<f32> {
        match &self.duration {
            None => None,
            Some(DurationExpr::Literal(v)) => Some(*v),
            Some(DurationExpr::Expr(expr)) => {
                let current = self.accumulated_timeline_time();
                let replaced = expr.replace("@current", &current.to_string());
                evaluate_simple_expr(&replaced)
            }
        }
    }

    /// Accumulated absolute time at the end of the timeline.
    ///
    /// Timeline 的最终累计绝对时间。
    fn accumulated_timeline_time(&self) -> f32 {
        let mut accumulated = 0.0f32;
        for event in &self.timeline {
            accumulated = match event.time_mode {
                TimeMode::Absolute => event.t,
                TimeMode::Delta => accumulated + event.t,
            };
        }
        accumulated
    }
}

/// Evaluate a simple arithmetic expression (supports `+`, `-`, `*`, `/` and parentheses).
/// Returns `None` on parse failure.
fn evaluate_simple_expr(expr: &str) -> Option<f32> {
    let expr = expr.trim();
    if expr.is_empty() {
        return None;
    }
    // Fast path: try parsing as a plain number first
    if let Ok(v) = expr.parse::<f32>() {
        return Some(v);
    }
    // Minimal recursive-descent parser for +, -, *, /
    let mut parser = SimpleExprParser::new(expr);
    parser.parse_expr()
}

struct SimpleExprParser<'a> {
    chars: &'a [u8],
    pos: usize,
}

impl<'a> SimpleExprParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.as_bytes(),
            pos: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn parse_expr(&mut self) -> Option<f32> {
        let mut result = self.parse_term()?;
        loop {
            self.skip_whitespace();
            if self.pos >= self.chars.len() {
                break;
            }
            match self.chars[self.pos] {
                b'+' => {
                    self.pos += 1;
                    result += self.parse_term()?;
                }
                b'-' => {
                    self.pos += 1;
                    result -= self.parse_term()?;
                }
                _ => break,
            }
        }
        Some(result)
    }

    fn parse_term(&mut self) -> Option<f32> {
        let mut result = self.parse_atom()?;
        loop {
            self.skip_whitespace();
            if self.pos >= self.chars.len() {
                break;
            }
            match self.chars[self.pos] {
                b'*' => {
                    self.pos += 1;
                    result *= self.parse_atom()?;
                }
                b'/' => {
                    self.pos += 1;
                    result = self.parse_division(result)?;
                }
                _ => break,
            }
        }
        Some(result)
    }

    fn parse_division(&mut self, dividend: f32) -> Option<f32> {
        let divisor = self.parse_atom()?;
        if divisor == 0.0 {
            return None;
        }
        Some(dividend / divisor)
    }

    fn parse_atom(&mut self) -> Option<f32> {
        self.skip_whitespace();
        if self.pos >= self.chars.len() {
            return None;
        }

        // Parenthesized expression
        if self.chars[self.pos] == b'(' {
            self.pos += 1;
            let result = self.parse_expr()?;
            self.skip_whitespace();
            if self.pos < self.chars.len() && self.chars[self.pos] == b')' {
                self.pos += 1;
            }
            return Some(result);
        }

        // Negative number / unary minus
        let negative = if self.chars[self.pos] == b'-' {
            self.pos += 1;
            true
        } else {
            false
        };

        self.skip_whitespace();
        let start = self.pos;
        while self.pos < self.chars.len()
            && (self.chars[self.pos].is_ascii_digit() || self.chars[self.pos] == b'.')
        {
            self.pos += 1;
        }
        if self.pos == start {
            return None;
        }
        let num_str = std::str::from_utf8(&self.chars[start..self.pos]).ok()?;
        let val: f32 = num_str.parse().ok()?;
        Some(if negative { -val } else { val })
    }
}

// ============================================================================
// Bullet Prototype
// ============================================================================

/// Hit behavior preset.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub enum HitBehaviorPreset {
    #[default]
    Default,
    Persistent,
    DamageWhenMoving,
    DamageWhenStationary,
    Custom {
        #[serde(default = "default_true")]
        despawn_on_hit: bool,
        #[serde(default)]
        damage_on_player_moving: bool,
        #[serde(default)]
        damage_on_player_stationary: bool,
        #[serde(default)]
        invincibility_duration: f32,
    },
}

/// Color tint configuration for bullets.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ColorTint {
    #[serde(default)]
    pub hex: String,
    #[serde(default)]
    pub rgba: Option<(f32, f32, f32, f32)>,
}

/// Bullet prototype — appearance and collision definition.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct BulletPrototype {
    pub visual: String,
    pub collider: ColliderShape,
    pub damage: f32,
    pub lifetime: f32,
    pub z_index: f32,
    pub scale: f32,
    #[serde(default)]
    pub rotation: f32,
    pub hit_behavior: HitBehaviorPreset,
    pub color_tint: ColorTint,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
    #[serde(default)]
    pub frame_duration: Option<f32>,
}

impl Default for BulletPrototype {
    fn default() -> Self {
        Self {
            visual: String::new(),
            collider: ColliderShape::default(),
            damage: 1.0,
            lifetime: 5.0,
            z_index: 15.0,
            scale: 1.0,
            rotation: 0.0,
            hit_behavior: HitBehaviorPreset::Default,
            color_tint: ColorTint::default(),
            flip_x: false,
            flip_y: false,
            frame_duration: None,
        }
    }
}

/// Collider shape for hit detection.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ColliderShape {
    CircleCollider(f32),
    BoxCollider(f32, f32),
}

impl Default for ColliderShape {
    fn default() -> Self {
        ColliderShape::CircleCollider(4.0)
    }
}

// ============================================================================
// Bullet Behavior
// ============================================================================

/// Bullet behavior definition.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum BulletBehavior {
    Linear(LinearConfig),
    Tween(TweenConfig),
    Stationary(),
    Orbital(OrbitalConfig),
    Sine(SineConfig),
    Aimed(AimedConfig),
    Custom {
        id: String,
        #[serde(default)]
        props: HashMap<String, f32>,
    },
}

/// Linear motion configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct LinearConfig {
    pub dir: (f32, f32),
    pub speed: f32,
}

impl Default for LinearConfig {
    fn default() -> Self {
        Self {
            dir: (0.0, -1.0),
            speed: 100.0,
        }
    }
}

/// Orbital motion configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct OrbitalConfig {
    pub angular_velocity: f32,
    pub radial_velocity: f32,
}

impl Default for OrbitalConfig {
    fn default() -> Self {
        Self {
            angular_velocity: 1.0,
            radial_velocity: 0.0,
        }
    }
}

/// Sine wave configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SineConfig {
    pub axis: (f32, f32),
    pub amplitude: f32,
    pub frequency: f32,
    pub phase: f32,
}

impl Default for SineConfig {
    fn default() -> Self {
        Self {
            axis: (1.0, 0.0),
            amplitude: 20.0,
            frequency: 2.0,
            phase: 0.0,
        }
    }
}

/// Aimed bullet configuration (自机狙).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AimedConfig {
    pub speed: f32,
    pub angle_offset: f32,
}

impl Default for AimedConfig {
    fn default() -> Self {
        Self {
            speed: 120.0,
            angle_offset: 0.0,
        }
    }
}

/// Mode for tween value interpretation.
///
/// 补间值的解释模式。
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub enum TweenMode {
    /// Value is added as an offset to the spawn position (default behavior).
    ///
    /// 值作为偏移量叠加到生成位置。
    #[default]
    Relative,
    /// Value directly sets the coordinate / property.
    ///
    /// 值直接设置坐标 / 属性。
    Absolute,
}

/// Tween animation configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TweenConfig {
    pub target: DanmakuTweenTarget,
    pub duration: f32,
    #[serde(default)]
    pub ease: Easing,
    #[serde(default)]
    pub from: f32,
    pub to: f32,
    #[serde(default)]
    pub delay: f32,
    #[serde(default)]
    pub mode: TweenMode,
}

/// Tween target properties (danmaku-specific).
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub enum DanmakuTweenTarget {
    #[default]
    Opacity,
    Scale,
    ScaleX,
    ScaleY,
    PositionX,
    PositionY,
    Rotation,
}

/// Easing functions for interpolation.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub enum Easing {
    #[default]
    Linear,
    QuadIn,
    QuadOut,
    InOutQuad,
    CubicIn,
    CubicOut,
    InOutCubic,
    SineIn,
    SineOut,
    InOutSine,
}

// ============================================================================
// Timeline Event
// ============================================================================

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
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimelineEvent {
    pub t: f32,
    #[serde(default)]
    pub time_mode: TimeMode,
    pub spawn: String,
    #[serde(default)]
    pub pattern: SpawnPattern,
    #[serde(default)]
    pub offset: (f32, f32),
    #[serde(default)]
    pub apply: Vec<String>,
    #[serde(default)]
    pub behaviors: Vec<BulletBehavior>,
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
        /// Number of bullets to spawn on the ring.
        ///
        /// 环上生成的弹幕数量。
        count: usize,
        /// Radius of the ring in world units. `0.0` collapses all bullets to the center.
        ///
        /// 环的半径（世界坐标单位）。`0.0` 时所有弹幕坍缩到中心。
        #[serde(default)]
        radius: f32,
        /// Starting angle in radians. `0.0` points right (+X), increases counter-clockwise.
        ///
        /// 起始角度（弧度）。`0.0` 朝右（+X 方向），逆时针递增。
        #[serde(default)]
        start_angle: f32,
        /// Positional jitter factor (0.0–1.0). `0.0` = exact positions, `1.0` = maximum scatter.
        /// Jitter magnitude is proportional to the arc distance between adjacent bullets.
        ///
        /// 位置抖动因子（0.0–1.0）。`0.0` = 精确位置，`1.0` = 最大散布。
        /// 抖动幅度与相邻弹幕之间的弧长成正比。
        #[serde(default)]
        randomness: f32,
    },

    /// Distributes bullets along a straight line.
    ///
    /// 将弹幕沿直线分布。
    LineGenerator {
        /// Number of bullets on the line.
        ///
        /// 线上的弹幕数量。
        count: usize,
        /// Distance between adjacent bullets in world units.
        ///
        /// 相邻弹幕之间的距离（世界坐标单位）。
        #[serde(default = "default_line_spacing")]
        spacing: f32,
        /// Direction vector of the line. Does not need to be normalized.
        ///
        /// 线的方向向量。无需归一化。
        #[serde(default = "default_linear_direction")]
        direction: (f32, f32),
        /// Positional jitter factor (0.0–1.0). `0.0` = exact positions, `1.0` = maximum scatter.
        /// Jitter magnitude is proportional to the spacing between bullets.
        ///
        /// 位置抖动因子（0.0–1.0）。`0.0` = 精确位置，`1.0` = 最大散布。
        /// 抖动幅度与弹幕间距成正比。
        #[serde(default)]
        randomness: f32,
    },

    /// Distributes bullets along a screen edge at fixed intervals.
    ///
    /// 将弹幕沿屏幕边缘以固定间隔分布。
    EdgeGenerator {
        /// Number of bullets along the edge.
        ///
        /// 沿边缘生成的弹幕数量。
        count: usize,
        /// Which screen edge to spawn from.
        ///
        /// 从哪条屏幕边缘生成。
        #[serde(default)]
        side: EdgeSide,
        /// Distance between adjacent bullets along the edge.
        ///
        /// 沿边缘相邻弹幕之间的距离。
        #[serde(default = "default_edge_spacing")]
        spacing: f32,
        /// Distance from the center of the screen to the spawn line.
        ///
        /// 从屏幕中心到生成线的距离。
        #[serde(default = "default_edge_margin")]
        margin: f32,
        /// Positional jitter factor (0.0–1.0). `0.0` = exact positions, `1.0` = maximum scatter.
        /// Jitter magnitude is proportional to the spacing between bullets.
        ///
        /// 位置抖动因子（0.0–1.0）。`0.0` = 精确位置，`1.0` = 最大散布。
        /// 抖动幅度与弹幕间距成正比。
        #[serde(default)]
        randomness: f32,
    },

    /// Spawns bullets relative to a named ViewBox's edge.
    /// `outside_margin` is the distance *outside* the box edge (not from center).
    /// At runtime, actual margin = box_half_size + outside_margin.
    ///
    /// 相对于命名 ViewBox 边缘生成弹幕。
    /// `outside_margin` 是距离框**外边缘**的距离（不是距中心的距离）。
    /// 运行时实际 margin = 框半尺寸 + outside_margin。
    BoxEdgeGenerator {
        /// Name of the ViewBox entity to reference for edge position.
        ///
        /// 用于确定边缘位置的 ViewBox 实体名称。
        box_name: String,
        /// Number of bullets along the edge.
        ///
        /// 沿边缘生成的弹幕数量。
        count: usize,
        /// Which edge of the box to spawn from.
        ///
        /// 从框的哪条边缘生成。
        #[serde(default)]
        side: EdgeSide,
        /// Distance between adjacent bullets along the edge.
        ///
        /// 沿边缘相邻弹幕之间的距离。
        #[serde(default = "default_edge_spacing")]
        spacing: f32,
        /// Distance outside the box edge where bullets spawn.
        ///
        /// 弹幕生成位置距离框外边缘的距离。
        #[serde(default)]
        outside_margin: f32,
        /// Positional jitter factor (0.0–1.0). `0.0` = exact positions, `1.0` = maximum scatter.
        /// Jitter magnitude is proportional to the spacing between bullets.
        ///
        /// 位置抖动因子（0.0–1.0）。`0.0` = 精确位置，`1.0` = 最大散布。
        /// 抖动幅度与弹幕间距成正比。
        #[serde(default)]
        randomness: f32,
    },

    /// User-defined spawn pattern provided by a WASM module.
    ///
    /// 由 WASM 模块提供的自定义生成模式。
    CustomGenerator {
        /// Identifier for the WASM pattern generator (e.g., `"my_mod.spiral"`).
        ///
        /// WASM 模式生成器的标识符（如 `"my_mod.spiral"`）。
        id: String,
        /// Key-value parameters passed to the WASM generator.
        ///
        /// 传递给 WASM 生成器的键值参数。
        #[serde(default)]
        params: HashMap<String, f32>,
    },
}

impl SpawnPattern {
    /// Returns the randomness factor (0.0–1.0) for this pattern.
    /// Non-generator variants and `CustomGenerator` return `0.0`.
    ///
    /// 返回此模式的随机因子（0.0–1.0）。
    /// 非生成器变体和 `CustomGenerator` 返回 `0.0`。
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
    /// Used as the reference scale for randomness jitter.
    ///
    /// 返回相邻生成点之间的代表性间距。
    /// 用作随机抖动的参考尺度。
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
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub enum EdgeSide {
    #[default]
    Left,
    Right,
    Top,
    Bottom,
}

// ============================================================================
// Default helper functions
// ============================================================================

fn default_true() -> bool {
    true
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
