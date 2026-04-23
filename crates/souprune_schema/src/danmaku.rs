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
    /// How long this performance lasts.
    /// - Literal: `duration: 5.0`
    /// - Omitted / 0 / negative: fire-and-forget (won't block sequencer).
    ///
    /// 演出持续时间。
    /// - 字面量：`duration: 5.0`
    /// - 省略 / 0 / 负数：不阻塞序列器。
    #[serde(default)]
    pub duration: Option<f32>,
}

impl DanmakuPerformance {
    /// Resolve `duration` to a blocking duration value.
    /// Returns `None` if duration is omitted. Callers treat zero or negative values as
    /// fire-and-forget.
    ///
    /// 将 `duration` 解析为阻塞时长。
    /// 如果省略则返回 `None`。调用方将零或负数视为不阻塞。
    pub fn resolved_duration(&self) -> Option<f32> {
        self.duration
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

impl ColorTint {
    /// Create a hex-only tint definition.
    ///
    /// 创建仅使用十六进制字符串的色调定义。
    pub fn hex(hex: impl Into<String>) -> Self {
        Self {
            hex: hex.into(),
            rgba: None,
        }
    }

    /// Create an RGBA tint definition.
    ///
    /// 创建 RGBA 色调定义。
    pub fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            hex: String::new(),
            rgba: Some((red, green, blue, alpha)),
        }
    }
}

/// Bullet prototype — appearance and collision definition.
///
/// 弹幕原型 — 外观与碰撞定义。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct BulletPrototype {
    /// Visual resource path (e.g., "bullet/small").
    ///
    /// 视觉资源路径（如 "bullet/small"）。
    pub visual: String,
    /// Collision shape and size.
    ///
    /// 碰撞形状与尺寸。
    pub collider: ColliderShape,
    /// Damage dealt to the player on hit.
    ///
    /// 命中时对玩家造成的伤害。
    pub damage: f32,
    /// Maximum time the bullet exists in world units (seconds).
    ///
    /// 弹幕在世界中存在的最大时间（秒）。
    pub lifetime: f32,
    /// Visual layering priority.
    ///
    /// 视觉图层优先级。
    pub z_index: f32,
    /// Uniform scale factor.
    ///
    /// 统一缩放比例。
    pub scale: f32,
    /// Initial rotation in radians.
    ///
    /// 初始旋转角度（弧度）。
    #[serde(default)]
    pub rotation: f32,
    /// Preset behavior when hitting the player.
    ///
    /// 命中玩家时的预设行为。
    pub hit_behavior: HitBehaviorPreset,
    /// Color tint applied to the visual.
    ///
    /// 应用于视觉资源的色调。
    pub color_tint: ColorTint,
    /// Whether to flip the visual horizontally.
    ///
    /// 是否水平翻转视觉资源。
    #[serde(default)]
    pub flip_x: bool,
    /// Whether to flip the visual vertically.
    ///
    /// 是否垂直翻转视觉资源。
    #[serde(default)]
    pub flip_y: bool,
    /// Duration of each animation frame (if applicable).
    ///
    /// 每个动画帧的持续时间（如果适用）。
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

impl BulletPrototype {
    /// Create a bullet prototype with the given visual and schema defaults.
    ///
    /// 使用指定视觉资源和 Schema 默认值创建弹幕原型。
    pub fn new(visual: impl Into<String>) -> Self {
        Self {
            visual: visual.into(),
            ..Default::default()
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

impl ColliderShape {
    /// Create a circular collider.
    ///
    /// 创建圆形碰撞体。
    pub fn circle(radius: f32) -> Self {
        Self::CircleCollider(radius)
    }

    /// Create a rectangular collider.
    ///
    /// 创建矩形碰撞体。
    pub fn rectangle(width: f32, height: f32) -> Self {
        Self::BoxCollider(width, height)
    }
}

// ============================================================================
// Bullet Behavior
// ============================================================================

/// Bullet behavior definition.
///
/// 弹幕行为定义。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum BulletBehavior {
    /// Constant velocity movement in a fixed direction.
    ///
    /// 固定方向的等速运动。
    Linear(LinearConfig),
    /// Interpolated property animation (e.g., opacity, scale, position).
    ///
    /// 属性插值动画（如不透明度、缩放、位置）。
    Tween(TweenConfig),
    /// No movement. Useful for static traps or anchors.
    ///
    /// 无运动。适用于静态陷阱或锚点。
    Stationary(),
    /// Circular movement around the spawn point.
    ///
    /// 围绕生成点的圆周运动。
    Orbital(OrbitalConfig),
    /// Sine-wave oscillation along an axis.
    ///
    /// 沿轴线的正弦波振荡。
    Sine(SineConfig),
    /// Movement towards the player's current position at spawn time.
    ///
    /// 朝向弹幕生成时玩家所在位置的运动（自机狙）。
    Aimed(AimedConfig),
    /// User-defined behavior implemented in code.
    ///
    /// 在代码中实现的自定义行为。
    Custom {
        /// Unique identifier for the custom behavior.
        ///
        /// 自定义行为的唯一标识符。
        id: String,
        /// Numeric property bindings passed to the behavior logic.
        ///
        /// 传递给行为逻辑的数值属性绑定。
        #[serde(default)]
        props: HashMap<String, f32>,
    },
}

impl BulletBehavior {
    /// Create an aimed behavior.
    ///
    /// 创建自机狙行为。
    pub fn aimed(speed: f32) -> Self {
        Self::Aimed(AimedConfig {
            speed,
            ..Default::default()
        })
    }

    /// Create a linear movement behavior.
    ///
    /// 创建线性移动行为。
    pub fn linear(dir: (f32, f32), speed: f32) -> Self {
        Self::Linear(LinearConfig { dir, speed })
    }

    /// Create a stationary behavior.
    ///
    /// 创建静止行为。
    pub fn stationary() -> Self {
        Self::Stationary()
    }

    /// Create an orbital movement behavior.
    ///
    /// 创建轨道移动行为。
    pub fn orbital(angular_velocity: f32, radial_velocity: f32) -> Self {
        Self::Orbital(OrbitalConfig {
            angular_velocity,
            radial_velocity,
        })
    }

    /// Create a sine-wave movement behavior.
    ///
    /// 创建正弦波移动行为。
    pub fn sine(axis: (f32, f32), amplitude: f32, frequency: f32) -> Self {
        Self::Sine(SineConfig {
            axis,
            amplitude,
            frequency,
            phase: 0.0,
        })
    }

    /// Create a tween behavior.
    ///
    /// 创建补间行为。
    pub fn tween(target: DanmakuTweenTarget, duration: f32, ease: Easing, to: f32) -> Self {
        Self::Tween(TweenConfig {
            target,
            duration,
            ease,
            to,
            ..Default::default()
        })
    }

    /// Create a delayed tween behavior.
    ///
    /// 创建延迟补间行为。
    pub fn tween_delayed(
        target: DanmakuTweenTarget,
        duration: f32,
        ease: Easing,
        to: f32,
        delay: f32,
    ) -> Self {
        Self::Tween(TweenConfig {
            target,
            duration,
            ease,
            to,
            delay,
            ..Default::default()
        })
    }

    /// Create a custom behavior with numeric property bindings.
    ///
    /// 创建带数值属性绑定的自定义行为。
    pub fn custom(
        id: impl Into<String>,
        props: impl IntoIterator<Item = (impl Into<String>, f32)>,
    ) -> Self {
        Self::Custom {
            id: id.into(),
            props: props
                .into_iter()
                .map(|(key, value)| (key.into(), value))
                .collect(),
        }
    }
}

/// Linear motion configuration.
///
/// 线性运动配置。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct LinearConfig {
    /// Direction vector (X, Y).
    ///
    /// 方向向量 (X, Y)。
    pub dir: (f32, f32),
    /// Speed in world units per second.
    ///
    /// 移动速度（世界单位/秒）。
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
///
/// 轨道运动配置。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct OrbitalConfig {
    /// Angular velocity in radians per second.
    ///
    /// 角速度（弧度/秒）。
    pub angular_velocity: f32,
    /// Radial velocity (expansion/contraction speed).
    ///
    /// 径向速度（扩张/收缩速度）。
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
///
/// 正弦波配置。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SineConfig {
    /// Reference axis for oscillation.
    ///
    /// 振荡的参考轴。
    pub axis: (f32, f32),
    /// Maximum displacement from the axis.
    ///
    /// 偏离轴线的最大位移。
    pub amplitude: f32,
    /// Oscillations per second.
    ///
    /// 每秒振荡次数（频率）。
    pub frequency: f32,
    /// Initial phase offset in radians.
    ///
    /// 初始相位偏移（弧度）。
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
///
/// 自机狙配置。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AimedConfig {
    /// Speed in world units per second.
    ///
    /// 移动速度（世界单位/秒）。
    pub speed: f32,
    /// Angular offset from the player direction (radians).
    ///
    /// 偏离玩家方向的角度偏移（弧度）。
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
///
/// 补间动画配置。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TweenConfig {
    /// Target property to animate.
    ///
    /// 要播放动画的目标属性。
    pub target: DanmakuTweenTarget,
    /// Duration of the animation in seconds.
    ///
    /// 动画持续时间（秒）。
    pub duration: f32,
    /// Easing function for interpolation.
    ///
    /// 用于插值的缓动函数。
    #[serde(default)]
    pub ease: Easing,
    /// Initial value.
    ///
    /// 初始值。
    #[serde(default)]
    pub from: f32,
    /// Target value.
    ///
    /// 目标值。
    pub to: f32,
    /// Delay before the animation starts (seconds).
    ///
    /// 动画开始前的延迟（秒）。
    #[serde(default)]
    pub delay: f32,
    /// Interpretation mode for `from` and `to` values.
    ///
    /// `from` 和 `to` 值的解释模式。
    #[serde(default)]
    pub mode: TweenMode,
}

impl Default for TweenConfig {
    fn default() -> Self {
        Self {
            target: DanmakuTweenTarget::default(),
            duration: 0.0,
            ease: Easing::default(),
            from: 0.0,
            to: 0.0,
            delay: 0.0,
            mode: TweenMode::default(),
        }
    }
}

/// Tween target properties (danmaku-specific).
///
/// 补间目标属性（弹幕专用）。
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub enum DanmakuTweenTarget {
    /// Visual opacity (alpha channel).
    ///
    /// 视觉不透明度（Alpha 通道）。
    #[default]
    Opacity,
    /// Uniform scale.
    ///
    /// 统一缩放。
    Scale,
    /// X-axis scale.
    ///
    /// X 轴缩放。
    ScaleX,
    /// Y-axis scale.
    ///
    /// Y 轴缩放。
    ScaleY,
    /// Relative or absolute X position.
    ///
    /// 相对或绝对 X 坐标。
    PositionX,
    /// Relative or absolute Y position.
    ///
    /// 相对或绝对 Y 坐标。
    PositionY,
    /// Rotation angle.
    ///
    /// 旋转角度。
    Rotation,
}

/// Easing functions for interpolation.
///
/// 用于插值的缓动函数。
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
