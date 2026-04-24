//! # Danmaku Behavior Schema
//!
//! Bullet movement and tween behavior schema types.
//!
//! # 弹幕行为 Schema
//!
//! 弹幕移动与补间行为 Schema 类型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        #[serde(default, serialize_with = "crate::ordered_map::serialize_ordered_map")]
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
